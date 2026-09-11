//! `RpcClient::await_op` — the client half of the deferred-operation
//! primitive, against a real `plana-rpc-server`.
//!
//! The interesting property is *how* the outcome is collected: the advisory
//! `ops.settled` notification when it arrives, `ops.result` polling when it
//! cannot (another transport began the op, the connection cycled), and never
//! a busy loop.

use std::time::{Duration, Instant};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio::sync::watch;

use plana::jsonrpc::deferred::{DeferredOpRef, OpsResultParams, OPS_RESULT_METHOD};
use plana::jsonrpc::{error_codes, JsonRpcError};
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
///
/// `count_collections` swaps in an instrumented `ops.result` that counts the
/// authoritative reads, which is how the "never a busy loop" claim is tested.
fn deferred_server(
    ttl: Duration,
    count_collections: Option<Arc<AtomicUsize>>,
) -> (RpcServer, watch::Sender<bool>) {
    let (release, release_rx) = watch::channel(false);
    let mut builder = RpcServer::builder()
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
        .method_ctx("failing.upstream", |ctx: RpcRequestCtx| async move {
            ctx.defer(|_handle| async move {
                Err(JsonRpcError::new(
                    error_codes::OPS_CANCELLED,
                    "upstream refused the request",
                ))
            })
        });
    if let Some(calls) = count_collections {
        builder = builder.method_ctx(OPS_RESULT_METHOD, move |ctx: RpcRequestCtx| {
            let calls = calls.clone();
            async move {
                calls.fetch_add(1, Ordering::SeqCst);
                let params: OpsResultParams = serde_json::from_value(ctx.params)
                    .map_err(|e| JsonRpcError::invalid_params(&format!("ops params: {e}")))?;
                let outcome = ctx.deferred.status(&params.op_id)?;
                serde_json::to_value(outcome)
                    .map_err(|e| JsonRpcError::internal_error(&e.to_string()))
            }
        });
    }
    (builder.build(), release)
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
    let (server, release) = deferred_server(Duration::from_secs(1800), None);
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

    // Drive `await_op` on its own task and release the worker only after the
    // first (immediate) `ops.result` read has certainly happened — otherwise a
    // slow start could observe `completed` on that first read and the test
    // would pass without ever exercising the notification path it names.
    let awaiting = {
        let client = client.clone();
        let op = op.clone();
        tokio::spawn(async move { client.await_op(&op, Duration::from_secs(3)).await })
    };
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert!(
        !awaiting.is_finished(),
        "the operation must still be pending while the worker is held"
    );
    release.send_replace(true);

    let started = Instant::now();
    let outcome = tokio::time::timeout(Duration::from_secs(5), awaiting)
        .await
        .expect("await_op must return before the outer guard")
        .expect("the awaiting task must not panic")
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
    let (server, release) = deferred_server(Duration::from_secs(1800), None);
    let (url, http) = spawn(server).await;

    let client = RpcClient::builder()
        .url(&url)
        .config(RpcClientConfig {
            op_poll_initial: Duration::from_millis(50),
            op_poll_max: Duration::from_millis(150),
            // Wide enough that 20ms state sampling cannot miss the
            // `Reconnecting` window on a loaded machine.
            reconnect_initial: Duration::from_millis(250),
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

    // The wait is already in flight when the connection cycles: the operation
    // is server-side state, so it survives, and the client must keep
    // collecting across the reconnect (the advisory notification dies with the
    // old connection).
    let awaiting = {
        let client = client.clone();
        let op = op.clone();
        tokio::spawn(async move { client.await_op(&op, Duration::from_secs(6)).await })
    };
    tokio::time::sleep(Duration::from_millis(100)).await;
    client.force_reconnect();
    // Wait for the cycle to actually happen: `Connected` can still be the
    // pre-reconnect state, so observing it alone would not prove the wait
    // spanned a reconnect.
    wait_for_state(
        &client,
        ConnectionState::Reconnecting,
        Duration::from_secs(5),
    )
    .await;
    wait_for_state(&client, ConnectionState::Connected, Duration::from_secs(5)).await;
    assert!(
        !awaiting.is_finished(),
        "the operation must still be pending across the reconnect"
    );
    release.send_replace(true);

    let outcome = tokio::time::timeout(Duration::from_secs(8), awaiting)
        .await
        .expect("await_op must return before the outer guard")
        .expect("the awaiting task must not panic")
        .expect("polling collects the outcome across a reconnect");

    assert_eq!(outcome.status, DeferredOpStatus::Completed);
    assert_eq!(outcome.result.unwrap()["completed"], true);
}

#[tokio::test]
async fn await_op_honours_the_deadline_without_losing_the_operation() {
    let (server, release) = deferred_server(Duration::from_secs(1800), None);
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
    let (server, _release) = deferred_server(Duration::from_millis(300), None);
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

/// The "no busy loop" promise must not rest on the caller configuring the
/// knobs sensibly: a zero initial delay, a zero ceiling and a shrinking factor
/// are all normalized — the delays to a ten-millisecond floor, the factor to
/// at least 1.0.
#[tokio::test]
async fn a_degenerate_poll_cadence_still_does_not_busy_loop() {
    let collected = Arc::new(AtomicUsize::new(0));
    let (server, _release) = deferred_server(Duration::from_secs(1800), Some(collected.clone()));
    let (url, _http) = spawn(server).await;

    let client = RpcClient::builder()
        .url(&url)
        .config(RpcClientConfig {
            // Every knob at its most degenerate value.
            op_poll_initial: Duration::ZERO,
            op_poll_factor: 0.0,
            op_poll_max: Duration::ZERO,
            ..RpcClientConfig::default()
        })
        .build();

    let answer = client.call(SLOW_METHOD, json!({})).await.unwrap();
    let op = op_from(&answer);

    let started = Instant::now();
    let err = tokio::time::timeout(
        Duration::from_secs(5),
        client.await_op(&op, Duration::from_millis(200)),
    )
    .await
    .expect("await_op must honour its own deadline")
    .expect_err("the never-released operation cannot settle");
    assert!(matches!(err, DeferredOpError::Deadline(_)), "got {err:?}");

    // With the 10ms floor this is ~20 polls; a scheduler-free spin over the
    // same 200ms is hundreds (a loopback round trip is well under a
    // millisecond).
    let calls = collected.load(Ordering::SeqCst);
    assert!(
        calls >= 2,
        "the fallback path must actually poll, saw {calls}"
    );
    assert!(
        calls <= 60,
        "polled ops.result {calls} times in {:?}: the cadence must be floored, not a busy loop",
        started.elapsed()
    );
}

/// A NaN growth factor would panic inside `Duration::mul_f64`; normalizing the
/// knobs must make it harmless.
#[tokio::test]
async fn a_nan_poll_factor_does_not_panic() {
    let (server, _release) = deferred_server(Duration::from_secs(1800), None);
    let (url, _http) = spawn(server).await;

    let client = RpcClient::builder()
        .url(&url)
        .config(RpcClientConfig {
            op_poll_initial: Duration::from_millis(10),
            op_poll_factor: f64::NAN,
            op_poll_max: Duration::from_millis(20),
            ..RpcClientConfig::default()
        })
        .build();

    let answer = client.call(SLOW_METHOD, json!({})).await.unwrap();
    let op = op_from(&answer);

    let err = tokio::time::timeout(
        Duration::from_secs(5),
        client.await_op(&op, Duration::from_millis(120)),
    )
    .await
    .expect("await_op must honour its own deadline")
    .expect_err("the never-released operation cannot settle");
    assert!(matches!(err, DeferredOpError::Deadline(_)), "got {err:?}");
}

/// A settled failure is a value, not a collection error: the caller gets the
/// worker's error inside the outcome.
#[tokio::test]
async fn await_op_returns_a_failed_outcome_as_a_value() {
    let (server, _release) = deferred_server(Duration::from_secs(1800), None);
    let (url, _http) = spawn(server).await;

    let client = RpcClient::builder()
        .url(&url)
        .config(RpcClientConfig {
            op_poll_initial: Duration::from_millis(30),
            op_poll_max: Duration::from_millis(60),
            ..RpcClientConfig::default()
        })
        .build();

    let answer = client.call("failing.upstream", json!({})).await.unwrap();
    let op = op_from(&answer);

    let outcome = tokio::time::timeout(
        Duration::from_secs(5),
        client.await_op(&op, Duration::from_secs(3)),
    )
    .await
    .expect("await_op must return before the outer guard")
    .expect("a failed operation is still a collected outcome");

    assert_eq!(outcome.status, DeferredOpStatus::Failed);
    assert!(outcome.result.is_none());
    let error = outcome.error.expect("the worker's error is carried");
    assert_eq!(error.code, error_codes::OPS_CANCELLED);
    assert_eq!(error.message, "upstream refused the request");
}

/// A deadline that the platform cannot represent (`Duration::MAX` is the
/// idiomatic "no timeout") must not panic the caller's task.
#[tokio::test]
async fn an_absurd_deadline_is_clamped_instead_of_panicking() {
    let (server, release) = deferred_server(Duration::from_secs(1800), None);
    let (url, _http) = spawn(server).await;

    let client = RpcClient::builder()
        .url(&url)
        .config(RpcClientConfig {
            op_poll_initial: Duration::from_millis(30),
            op_poll_max: Duration::from_millis(60),
            ..RpcClientConfig::default()
        })
        .build();

    let answer = client.call(SLOW_METHOD, json!({})).await.unwrap();
    let op = op_from(&answer);
    release_after(release, Duration::from_millis(120));

    let outcome = tokio::time::timeout(Duration::from_secs(5), client.await_op(&op, Duration::MAX))
        .await
        .expect("await_op must return before the outer guard")
        .expect("the operation settles");
    assert_eq!(outcome.status, DeferredOpStatus::Completed);
}

/// A payload the client cannot read is permanent: it must be reported, not
/// retried until the caller's deadline and then blamed on the deadline.
#[tokio::test]
async fn an_unreadable_ops_result_is_reported_as_a_protocol_error() {
    // A service that overrides the framework's collection method with an
    // incompatible shape (the profile allows the override).
    let server = RpcServer::builder()
        .config(RpcServerConfig::default())
        .method_ctx("slow.upstream", |ctx: RpcRequestCtx| async move {
            ctx.defer(|_handle| async {
                // Never settles: collection is what is under test here.
                std::future::pending::<()>().await;
                Ok(json!(null))
            })
        })
        .method_ctx(OPS_RESULT_METHOD, |_ctx: RpcRequestCtx| async move {
            Ok(json!({"status": "completed", "unexpected": true}))
        })
        .build();
    let (url, _http) = spawn(server).await;

    let client = RpcClient::builder()
        .url(&url)
        .config(RpcClientConfig {
            op_poll_initial: Duration::from_millis(30),
            op_poll_max: Duration::from_millis(60),
            ..RpcClientConfig::default()
        })
        .build();

    let answer = client.call("slow.upstream", json!({})).await.unwrap();
    let op = op_from(&answer);

    let started = Instant::now();
    let err = tokio::time::timeout(
        Duration::from_secs(5),
        client.await_op(&op, Duration::from_secs(30)),
    )
    .await
    .expect("a permanent protocol error must not wait for the deadline")
    .expect_err("an unreadable shape has no outcome");
    assert!(
        matches!(err, DeferredOpError::Protocol(_)),
        "expected a protocol error, got {err:?}"
    );
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "the caller must hear about version skew immediately, waited {:?}",
        started.elapsed()
    );
}

/// The other absurd end of the scale: a delay so large that growing it would
/// overflow `Duration::mul_f64`. The cadence is capped, so it is harmless.
#[tokio::test]
async fn an_absurd_poll_delay_does_not_panic() {
    let (server, _release) = deferred_server(Duration::from_secs(1800), None);
    let (url, _http) = spawn(server).await;

    let client = RpcClient::builder()
        .url(&url)
        .config(RpcClientConfig {
            op_poll_initial: Duration::MAX,
            op_poll_max: Duration::MAX,
            ..RpcClientConfig::default()
        })
        .build();

    let answer = client.call(SLOW_METHOD, json!({})).await.unwrap();
    let op = op_from(&answer);

    let err = tokio::time::timeout(
        Duration::from_secs(5),
        client.await_op(&op, Duration::from_millis(120)),
    )
    .await
    .expect("await_op must honour its own deadline")
    .expect_err("the never-released operation cannot settle");
    assert!(matches!(err, DeferredOpError::Deadline(_)), "got {err:?}");
}

/// A zero deadline means "do not wait": it answers immediately and does not
/// even issue a read.
#[tokio::test]
async fn a_zero_deadline_answers_without_reading() {
    let collected = Arc::new(AtomicUsize::new(0));
    let (server, _release) = deferred_server(Duration::from_secs(1800), Some(collected.clone()));
    let (url, _http) = spawn(server).await;

    let client = RpcClient::builder().url(&url).build();
    let answer = client.call(SLOW_METHOD, json!({})).await.unwrap();
    let op = op_from(&answer);

    let err = client
        .await_op(&op, Duration::ZERO)
        .await
        .expect_err("a zero deadline cannot collect anything");
    assert!(matches!(err, DeferredOpError::Deadline(_)), "got {err:?}");
    assert_eq!(
        collected.load(Ordering::SeqCst),
        0,
        "a zero deadline must not issue a read"
    );
}

/// A client that has exhausted its reconnect budget says so immediately
/// instead of waiting out a long deadline it can never satisfy.
#[tokio::test]
async fn a_dead_client_fails_fast_instead_of_waiting_out_the_deadline() {
    // Reserve an ephemeral port and never serve on it, so the client ends up
    // in `ConnectionState::Failed`.
    let probe = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = probe.local_addr().unwrap().port();
    drop(probe);

    let client = RpcClient::builder()
        .url(format!("ws://127.0.0.1:{port}/api/ws"))
        .config(RpcClientConfig {
            connect_timeout: Duration::from_millis(80),
            reconnect_initial: Duration::from_millis(20),
            max_reconnect_attempts: Some(2),
            op_poll_initial: Duration::from_millis(20),
            op_poll_max: Duration::from_millis(40),
            ..RpcClientConfig::default()
        })
        .build();
    wait_for_state(&client, ConnectionState::Failed, Duration::from_secs(5)).await;

    let started = Instant::now();
    let err = tokio::time::timeout(
        Duration::from_secs(5),
        client.await_op(&DeferredOpRef::new_random(), Duration::from_secs(300)),
    )
    .await
    .expect("a dead client must not wait out the deadline")
    .expect_err("a dead client cannot collect anything");
    assert!(
        matches!(err, DeferredOpError::Rpc(_)),
        "expected the transport error, got {err:?}"
    );
    assert!(
        started.elapsed() < Duration::from_secs(3),
        "took {:?} to report a dead client",
        started.elapsed()
    );
}
