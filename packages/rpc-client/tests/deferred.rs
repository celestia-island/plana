//! `RpcClient::await_op` — the client half of the deferred-operation
//! primitive, against a real `plana-rpc-server`.
//!
//! The interesting property is *how* the outcome is collected: the advisory
//! `ops.settled` notification when it arrives, `ops.result` polling when it
//! cannot (another transport began the op, the connection cycled), and never
//! a busy loop.

use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio::sync::watch;

use plana::jsonrpc::deferred::DeferredOpRef;
use plana_rpc_client::{
    post_rpc, ConnectionState, DeferredOpError, DeferredOpStatus, RpcClient, RpcClientConfig,
};
use plana_rpc_server::{DeferredOpsConfig, RpcRequestCtx, RpcServer, RpcServerConfig};

const SLOW_METHOD: &str = "slow.upstream";

/// Bind on a random port; returns `(ws_url, http_url)`.
async fn spawn(server: RpcServer) -> (String, String) {
    let app = server.mount_at("/api/ws");
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (
        format!("ws://{addr}/api/ws"),
        format!("http://{addr}/api/ws"),
    )
}

/// A server whose `slow.upstream` defers every call and settles only when the
/// test releases it. The worker reports the cancellation flag it observed.
fn deferred_server(ttl: Duration) -> (RpcServer, watch::Sender<bool>) {
    let (release, release_rx) = watch::channel(false);
    let server = RpcServer::builder()
        .config(RpcServerConfig {
            deferred_ops: DeferredOpsConfig {
                ttl,
                ..DeferredOpsConfig::default()
            },
            ..RpcServerConfig::default()
        })
        .method_ctx(SLOW_METHOD, move |ctx: RpcRequestCtx| {
            let release = release_rx.clone();
            async move {
                ctx.defer(move |handle| async move {
                    let mut release = release;
                    while !*release.borrow_and_update() {
                        if release.changed().await.is_err() {
                            break;
                        }
                    }
                    Ok(json!({
                        "completed": true,
                        "cancel_requested": handle.cancel_requested(),
                    }))
                })
            }
        })
        .build();
    (server, release)
}

async fn wait_for_state(client: &RpcClient, want: ConnectionState, within: Duration) {
    let deadline = tokio::time::Instant::now() + within;
    while client.state() != want {
        assert!(
            tokio::time::Instant::now() < deadline,
            "state did not reach {want:?}, now {:?}",
            client.state()
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

fn op_from(answer: &Value) -> DeferredOpRef {
    DeferredOpRef::from_wire(
        answer["op_id"]
            .as_str()
            .unwrap_or_else(|| panic!("no op_id in {answer}"))
            .to_string(),
    )
}

/// Release the worker after `delay` without blocking the test.
fn release_after(release: watch::Sender<bool>, delay: Duration) {
    tokio::spawn(async move {
        tokio::time::sleep(delay).await;
        release.send_replace(true);
    });
}

#[tokio::test]
async fn await_op_resolves_by_the_settle_notification_not_by_polling() {
    let (server, release) = deferred_server(Duration::from_secs(1800));
    let (url, _http) = spawn(server).await;

    // Poll cadence far longer than the deadline: only the advisory
    // notification can resolve this call in time.
    let client = RpcClient::builder()
        .url(&url)
        .config(RpcClientConfig {
            op_poll_initial: Duration::from_secs(60),
            op_poll_max: Duration::from_secs(60),
            ..RpcClientConfig::default()
        })
        .build();

    let answer = client.call(SLOW_METHOD, json!({})).await.unwrap();
    let op = op_from(&answer);
    release_after(release, Duration::from_millis(250));

    let started = Instant::now();
    let outcome = tokio::time::timeout(
        Duration::from_secs(5),
        client.await_op(&op, Duration::from_secs(3)),
    )
    .await
    .expect("await_op must return before the outer guard")
    .expect("the operation settles");
    let elapsed = started.elapsed();

    assert_eq!(outcome.status, DeferredOpStatus::Completed);
    assert_eq!(outcome.result.unwrap()["completed"], true);
    assert_eq!(outcome.op_id, op);
    assert!(
        elapsed < Duration::from_millis(1500),
        "took {elapsed:?}: with a 60s poll cadence the settle notification must be what woke us"
    );
}

#[tokio::test]
async fn await_op_polls_when_no_notification_can_arrive_and_survives_a_reconnect() {
    let (server, release) = deferred_server(Duration::from_secs(1800));
    let (url, http) = spawn(server).await;

    let client = RpcClient::builder()
        .url(&url)
        .config(RpcClientConfig {
            op_poll_initial: Duration::from_millis(50),
            op_poll_max: Duration::from_millis(150),
            reconnect_initial: Duration::from_millis(40),
            ..RpcClientConfig::default()
        })
        .build();

    // Begun over the HTTP POST fallback: that transport has no server->
    // client direction at all, so no settle notification can ever reach this
    // client — collection can only happen by polling.
    let answer = post_rpc(&http, SLOW_METHOD, json!({}), Duration::from_secs(2))
        .await
        .unwrap();
    let op = op_from(&answer);

    // Cycle the connection in the middle: the op is server-side state, so it
    // survives; only the (advisory) notification is lost.
    client.force_reconnect();
    wait_for_state(&client, ConnectionState::Connected, Duration::from_secs(5)).await;

    release_after(release, Duration::from_millis(300));

    let outcome = tokio::time::timeout(
        Duration::from_secs(8),
        client.await_op(&op, Duration::from_secs(6)),
    )
    .await
    .expect("await_op must return before the outer guard")
    .expect("polling collects the outcome");

    assert_eq!(outcome.status, DeferredOpStatus::Completed);
    assert_eq!(outcome.result.unwrap()["completed"], true);
}

#[tokio::test]
async fn await_op_honours_the_deadline_without_losing_the_operation() {
    let (server, release) = deferred_server(Duration::from_secs(1800));
    let (url, _http) = spawn(server).await;

    let client = RpcClient::builder()
        .url(&url)
        .config(RpcClientConfig {
            op_poll_initial: Duration::from_millis(40),
            op_poll_max: Duration::from_millis(80),
            ..RpcClientConfig::default()
        })
        .build();

    let answer = client.call(SLOW_METHOD, json!({})).await.unwrap();
    let op = op_from(&answer);

    // Never released: the caller's deadline is what ends the wait.
    let started = Instant::now();
    let err = tokio::time::timeout(
        Duration::from_secs(5),
        client.await_op(&op, Duration::from_millis(300)),
    )
    .await
    .expect("await_op must honour its own deadline")
    .expect_err("a pending operation is never returned as an outcome");
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "the deadline must bound the wait, took {:?}",
        started.elapsed()
    );
    match err {
        DeferredOpError::Deadline(d) => assert_eq!(d, Duration::from_millis(300)),
        other => panic!("expected Deadline, got {other:?}"),
    }

    // The operation itself is untouched: the same id still collects.
    release_after(release, Duration::from_millis(50));
    let outcome = tokio::time::timeout(
        Duration::from_secs(5),
        client.await_op(&op, Duration::from_secs(3)),
    )
    .await
    .expect("await_op must return before the outer guard")
    .expect("the operation is still collectable after a missed deadline");
    assert_eq!(outcome.status, DeferredOpStatus::Completed);
}

#[tokio::test]
async fn await_op_reports_unknown_and_expired_ids_terminally() {
    let (server, _release) = deferred_server(Duration::from_millis(300));
    let (url, _http) = spawn(server).await;

    let client = RpcClient::builder()
        .url(&url)
        .config(RpcClientConfig {
            op_poll_initial: Duration::from_millis(40),
            op_poll_max: Duration::from_millis(80),
            ..RpcClientConfig::default()
        })
        .build();

    // Never issued by this server, with a deadline far in the future: the
    // answer is terminal, not a wait until the deadline.
    let started = Instant::now();
    let err = tokio::time::timeout(
        Duration::from_secs(5),
        client.await_op(&DeferredOpRef::new_random(), Duration::from_secs(30)),
    )
    .await
    .expect("await_op must not wait for an unknown id")
    .expect_err("an unknown id has no outcome");
    assert!(matches!(err, DeferredOpError::Unknown), "got {err:?}");
    assert!(started.elapsed() < Duration::from_secs(2));

    // Issued, then outlived its window before settling.
    let answer = client.call(SLOW_METHOD, json!({})).await.unwrap();
    let op = op_from(&answer);
    let err = tokio::time::timeout(
        Duration::from_secs(5),
        client.await_op(&op, Duration::from_secs(20)),
    )
    .await
    .expect("await_op must return before the outer guard")
    .expect_err("an expired id has no outcome");
    assert!(matches!(err, DeferredOpError::Expired), "got {err:?}");
}
