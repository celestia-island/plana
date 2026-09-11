//! Deferred-operation conformance — the executable form of the latency
//! policy in the `plana-rpc-server` crate docs: a handler whose upstream can
//! outlive the dispatch stall limit answers immediately with an op ref, and
//! the outcome is collectable afterwards.
//!
//! Everything here goes through the real wire (WebSocket, plus one HTTP POST
//! case): the tests hold no reference to the worker's handle, they only see
//! the JSON-RPC surface a client sees.

use std::time::{Duration, Instant};

use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio_tungstenite::tungstenite::Message;

use plana::jsonrpc::deferred::{OPS_CANCEL_METHOD, OPS_RESULT_METHOD, OPS_SETTLED_METHOD};
use plana::jsonrpc::error_codes;
use plana_rpc_server::{DeferredOps, DeferredOpsConfig, RpcRequestCtx, RpcServer, RpcServerConfig};

/// The deferred method under test: it never touches its upstream until the
/// test releases it, which is what makes "answered immediately" observable.
const SLOW_METHOD: &str = "slow.upstream";

/// Bind a server on a random loopback port and return its ws/http base URL.
async fn spawn(server: RpcServer, path: &str) -> String {
    let app = server.mount_at(path);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("ws://{addr}{path}")
}

async fn connect(
    url: &str,
) -> tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>> {
    let (stream, _resp) = tokio_tungstenite::connect_async(url).await.unwrap();
    stream
}

async fn send_json<S>(stream: &mut S, value: Value)
where
    S: SinkExt<Message> + Unpin,
    S::Error: std::fmt::Debug,
{
    stream
        .send(Message::Text(value.to_string().into()))
        .await
        .unwrap();
}

/// Read the next JSON object frame off the wire (notifications included).
async fn recv_json<S>(stream: &mut S) -> Value
where
    S: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    loop {
        let msg = stream.next().await.unwrap().unwrap();
        if let Message::Text(text) = msg {
            return serde_json::from_str(&text).unwrap();
        }
    }
}

/// One request/response round trip on an open socket. Server-initiated
/// notifications (settle announcements, heartbeat acks) are skipped: they are
/// interleaved with responses on the data lane by design.
async fn call<S>(stream: &mut S, id: &str, method: &str, params: Value) -> Value
where
    S: SinkExt<Message>
        + StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>
        + Unpin,
    S::Error: std::fmt::Debug,
{
    send_json(
        stream,
        json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}),
    )
    .await;
    loop {
        let frame = recv_json(stream).await;
        if frame.get("id").is_none() {
            continue;
        }
        assert_eq!(frame["id"], id, "unexpected frame: {frame}");
        return frame;
    }
}

/// Poll `ops.result` until the outcome settles, bounded by `within`.
async fn collect_settled<S>(stream: &mut S, op_id: &str, within: Duration) -> Value
where
    S: SinkExt<Message>
        + StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>
        + Unpin,
    S::Error: std::fmt::Debug,
{
    let deadline = Instant::now() + within;
    loop {
        let frame = call(
            stream,
            "collect",
            OPS_RESULT_METHOD,
            json!({"op_id": op_id}),
        )
        .await;
        if frame["result"]["status"] != "pending" {
            return frame["result"].clone();
        }
        assert!(
            Instant::now() < deadline,
            "outcome never settled within {within:?}"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

/// A server whose `slow.upstream` defers every call, plus the release channel
/// the test uses to let the worker settle.
struct Fixture {
    server: RpcServer,
    ops: DeferredOps,
    release: watch::Sender<bool>,
}

impl Fixture {
    fn new(config: RpcServerConfig) -> Self {
        let (release, release_rx) = watch::channel(false);
        let server = RpcServer::builder()
            .config(config)
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
        let ops = server.deferred_ops().clone();
        Self {
            server,
            ops,
            release,
        }
    }
}

fn test_config() -> RpcServerConfig {
    RpcServerConfig {
        idle_timeout: Duration::from_secs(5),
        // Deliberately far below the upstream's real duration: a handler that
        // blocked on the upstream instead of deferring would be answered
        // `-32051` here.
        dispatch_stall_limit: Duration::from_millis(200),
        ..RpcServerConfig::default()
    }
}

#[tokio::test]
async fn deferred_handler_answers_immediately_with_an_id() {
    let Fixture {
        server, release, ..
    } = Fixture::new(test_config());
    let url = spawn(server, "/api/ws").await;
    let mut ws = connect(&url).await;

    let started = Instant::now();
    let answer = call(&mut ws, "d1", SLOW_METHOD, json!({})).await;
    let elapsed = started.elapsed();

    assert!(
        answer.get("error").is_none(),
        "a deferred handler must not stall: {answer}"
    );
    assert!(
        elapsed < Duration::from_millis(150),
        "answer took {elapsed:?}, the dispatch was not handed off"
    );
    let op_id = answer["result"]["op_id"]
        .as_str()
        .expect("deferred answer carries op_id")
        .to_string();
    assert_eq!(
        answer["result"]["expires_in"], 1800,
        "the advertised window is the 30-minute default"
    );
    assert_eq!(
        op_id.len(),
        36,
        "op ids are opaque UUIDs, not counters: {op_id}"
    );

    // Still running upstream: the collection surface reports it as pending.
    let outcome = call(&mut ws, "c1", OPS_RESULT_METHOD, json!({"op_id": op_id})).await;
    assert_eq!(outcome["result"]["status"], "pending");
    assert_eq!(outcome["result"]["method"], SLOW_METHOD);
    assert!(outcome["result"]["result"].is_null());
    assert!(outcome["result"]["error"].is_null());
    assert!(outcome["result"]["expires_in"].as_u64().unwrap() <= 1800);

    release.send_replace(true);
}

#[tokio::test]
async fn outcome_is_collectable_after_completion_and_stays_collectable() {
    let Fixture {
        server, release, ..
    } = Fixture::new(test_config());
    let url = spawn(server, "/api/ws").await;
    let mut ws = connect(&url).await;

    let answer = call(&mut ws, "d2", SLOW_METHOD, json!({})).await;
    let op_id = answer["result"]["op_id"].as_str().unwrap().to_string();
    release.send_replace(true);

    let outcome = collect_settled(&mut ws, &op_id, Duration::from_secs(3)).await;
    assert_eq!(outcome["status"], "completed");
    assert_eq!(outcome["result"]["completed"], true);

    // Consumption is non-destructive (documented choice): a second collection
    // still answers, so a lost frame cannot cost the caller the outcome.
    let again = call(&mut ws, "c2", OPS_RESULT_METHOD, json!({"op_id": op_id})).await;
    assert_eq!(again["result"]["status"], "completed");
    assert_eq!(again["result"]["result"]["completed"], true);
}

#[tokio::test]
async fn outcome_is_collectable_from_a_second_connection() {
    let Fixture {
        server, release, ..
    } = Fixture::new(test_config());
    let url = spawn(server, "/api/ws").await;

    let mut first = connect(&url).await;
    let answer = call(&mut first, "d3", SLOW_METHOD, json!({})).await;
    let op_id = answer["result"]["op_id"].as_str().unwrap().to_string();
    // The originating connection goes away entirely.
    first.close(None).await.unwrap();
    drop(first);
    tokio::time::sleep(Duration::from_millis(50)).await;

    release.send_replace(true);

    let mut second = connect(&url).await;
    let outcome = collect_settled(&mut second, &op_id, Duration::from_secs(3)).await;
    assert_eq!(
        outcome["status"], "completed",
        "a reconnect inside the window must still collect"
    );
    assert_eq!(outcome["result"]["completed"], true);
}

#[tokio::test]
async fn unknown_and_expired_ids_answer_structured_errors() {
    let Fixture {
        server, release, ..
    } = Fixture::new(RpcServerConfig {
        deferred_ops: DeferredOpsConfig {
            ttl: Duration::from_millis(150),
            ..DeferredOpsConfig::default()
        },
        ..test_config()
    });
    let url = spawn(server, "/api/ws").await;
    let mut ws = connect(&url).await;

    // Never issued → unknown, with the published code and a machine-readable
    // reason (no panic, no hang, no silent success).
    let unknown = call(
        &mut ws,
        "u1",
        OPS_RESULT_METHOD,
        json!({"op_id": "01912345-6789-7abc-8def-0123456789ab"}),
    )
    .await;
    assert_eq!(unknown["error"]["code"], error_codes::OPS_UNKNOWN);
    assert_eq!(unknown["error"]["data"]["reason"], "unknown_op_id");

    // Malformed params are a params error, not an unknown-op error.
    let bad = call(&mut ws, "u2", OPS_RESULT_METHOD, json!({})).await;
    assert_eq!(bad["error"]["code"], error_codes::INVALID_PARAMS);

    // Issued, then outlived its window → expired, and pruning makes the
    // second lookup unknown.
    let answer = call(&mut ws, "d4", SLOW_METHOD, json!({})).await;
    let op_id = answer["result"]["op_id"].as_str().unwrap().to_string();
    tokio::time::sleep(Duration::from_millis(260)).await;

    let expired = call(&mut ws, "u3", OPS_RESULT_METHOD, json!({"op_id": op_id})).await;
    assert_eq!(expired["error"]["code"], error_codes::OPS_EXPIRED);
    assert_eq!(expired["error"]["data"]["reason"], "op_id_expired");

    let pruned = call(&mut ws, "u4", OPS_RESULT_METHOD, json!({"op_id": op_id})).await;
    assert_eq!(pruned["error"]["code"], error_codes::OPS_UNKNOWN);

    release.send_replace(true);
}

#[tokio::test]
async fn settle_notification_reaches_the_originating_connection() {
    let Fixture {
        server, release, ..
    } = Fixture::new(test_config());
    let url = spawn(server, "/api/ws").await;
    let mut ws = connect(&url).await;

    let answer = call(&mut ws, "d5", SLOW_METHOD, json!({})).await;
    let op_id = answer["result"]["op_id"].as_str().unwrap().to_string();
    release.send_replace(true);

    let notification = tokio::time::timeout(Duration::from_secs(3), recv_json(&mut ws))
        .await
        .expect("the settle notification must arrive while the connection is open");
    assert_eq!(notification["method"], OPS_SETTLED_METHOD);
    assert!(
        notification.get("id").is_none(),
        "notifications carry no id"
    );
    assert_eq!(notification["params"]["op_id"], op_id.as_str());
    assert_eq!(notification["params"]["status"], "completed");
    assert!(
        notification["params"].get("result").is_none(),
        "the notification is advisory: collection stays authoritative"
    );
}

#[tokio::test]
async fn expiry_prunes_retained_entries() {
    let Fixture {
        server,
        ops,
        release,
    } = Fixture::new(RpcServerConfig {
        deferred_ops: DeferredOpsConfig {
            ttl: Duration::from_millis(150),
            ..DeferredOpsConfig::default()
        },
        ..test_config()
    });
    let url = spawn(server, "/api/ws").await;
    let mut ws = connect(&url).await;

    call(&mut ws, "d6", SLOW_METHOD, json!({})).await;
    call(&mut ws, "d7", SLOW_METHOD, json!({})).await;
    assert_eq!(ops.retained_len(), 2);

    tokio::time::sleep(Duration::from_millis(260)).await;
    // Memory cannot grow without limit: the next begin sweeps what expired.
    call(&mut ws, "d8", SLOW_METHOD, json!({})).await;
    assert_eq!(
        ops.retained_len(),
        1,
        "expired entries must be pruned, not retained forever"
    );

    release.send_replace(true);
}

#[tokio::test]
async fn pending_cap_refuses_structurally() {
    let Fixture {
        server, release, ..
    } = Fixture::new(RpcServerConfig {
        deferred_ops: DeferredOpsConfig {
            max_pending: 1,
            ..DeferredOpsConfig::default()
        },
        ..test_config()
    });
    let url = spawn(server, "/api/ws").await;
    let mut ws = connect(&url).await;

    let first = call(&mut ws, "d9", SLOW_METHOD, json!({})).await;
    assert!(first.get("error").is_none(), "{first}");

    let refused = call(&mut ws, "d10", SLOW_METHOD, json!({})).await;
    assert_eq!(
        refused["error"]["code"],
        error_codes::OPS_CAPACITY,
        "the caller sees backpressure instead of an uncollectable op"
    );
    assert_eq!(refused["error"]["data"]["pending"], 1);
    assert_eq!(refused["error"]["data"]["max_pending"], 1);

    // Once the outstanding op is collected, the cap frees up.
    release.send_replace(true);
    let op_id = first["result"]["op_id"].as_str().unwrap().to_string();
    let outcome = collect_settled(&mut ws, &op_id, Duration::from_secs(3)).await;
    assert_eq!(outcome["status"], "completed");
    let after = call(&mut ws, "d11", SLOW_METHOD, json!({})).await;
    assert!(after.get("error").is_none(), "{after}");
}

#[tokio::test]
async fn ops_cancel_records_a_flag_the_worker_observes() {
    let Fixture {
        server, release, ..
    } = Fixture::new(test_config());
    let url = spawn(server, "/api/ws").await;
    let mut ws = connect(&url).await;

    let answer = call(&mut ws, "d12", SLOW_METHOD, json!({})).await;
    let op_id = answer["result"]["op_id"].as_str().unwrap().to_string();

    let cancel = call(&mut ws, "x1", OPS_CANCEL_METHOD, json!({"op_id": op_id})).await;
    assert_eq!(cancel["result"]["status"], "pending");
    assert_eq!(cancel["result"]["cancel_requested"], true);

    // The worker sees the request and reports it in its own result.
    release.send_replace(true);
    let outcome = collect_settled(&mut ws, &op_id, Duration::from_secs(3)).await;
    assert_eq!(outcome["status"], "completed");
    assert_eq!(outcome["result"]["cancel_requested"], true);

    // Cancelling a settled op is a valid answer, not an error.
    let late = call(&mut ws, "x2", OPS_CANCEL_METHOD, json!({"op_id": op_id})).await;
    assert_eq!(late["result"]["status"], "completed");
    assert_eq!(late["result"]["cancel_requested"], false);

    // Cancelling an unknown op is the structured unknown-op error.
    let unknown = call(
        &mut ws,
        "x3",
        OPS_CANCEL_METHOD,
        json!({"op_id": "01912345-6789-7abc-8def-000000000000"}),
    )
    .await;
    assert_eq!(unknown["error"]["code"], error_codes::OPS_UNKNOWN);
}

#[tokio::test]
async fn a_service_may_override_the_builtin_collection_method() {
    // The built-in methods are mounted by the framework, but a service keeps
    // the last word on its own method map.
    let server = RpcServer::builder()
        .config(test_config())
        .method_ctx(OPS_RESULT_METHOD, |_ctx: RpcRequestCtx| async move {
            Ok(json!({"service_specific": true}))
        })
        .build();
    let url = spawn(server, "/api/ws").await;
    let mut ws = connect(&url).await;

    let frame = call(
        &mut ws,
        "o1",
        OPS_RESULT_METHOD,
        json!({"op_id": "whatever"}),
    )
    .await;
    assert_eq!(frame["result"]["service_specific"], true);
}

#[tokio::test]
async fn deferred_ops_are_collectable_over_the_http_fallback() {
    // The registry is server-global, not connection-bound: an op begun on a
    // WebSocket is collectable through the degraded HTTP POST transport.
    let _ = rustls::crypto::ring::default_provider().install_default();
    let Fixture {
        server, release, ..
    } = Fixture::new(test_config());
    let url = spawn(server, "/api/ws").await;
    let http_url = url.replace("ws://", "http://");

    let mut ws = connect(&url).await;
    let answer = call(&mut ws, "d13", SLOW_METHOD, json!({})).await;
    let op_id = answer["result"]["op_id"].as_str().unwrap().to_string();
    release.send_replace(true);
    collect_settled(&mut ws, &op_id, Duration::from_secs(3)).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(&http_url)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "http-1",
            "method": OPS_RESULT_METHOD,
            "params": {"op_id": op_id},
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["result"]["status"], "completed");
    assert_eq!(body["result"]["result"]["completed"], true);

    // The HTTP transport has no server-initiated direction, so an op begun
    // there can never be announced — it must still register and settle in the
    // same server-global registry.
    let resp = client
        .post(&http_url)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "http-2",
            "method": SLOW_METHOD,
            "params": {},
        }))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let http_op = body["result"]["op_id"].as_str().unwrap().to_string();
    let collected = client
        .post(&http_url)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "http-3",
            "method": OPS_RESULT_METHOD,
            "params": {"op_id": http_op},
        }))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(collected["result"]["status"], "completed");
    assert_eq!(collected["result"]["result"]["completed"], true);
}

/// A worker that panics must still leave the caller an authoritative answer:
/// the upstream may already have been paid for, so "pending until the window
/// elapses" would be the worst possible outcome.
#[tokio::test]
async fn a_panicking_worker_settles_the_operation_as_failed() {
    let server = RpcServer::builder()
        .config(test_config())
        .method_ctx("panic.upstream", |ctx: RpcRequestCtx| async move {
            ctx.defer(|_handle| async move {
                panic!("upstream exploded");
            })
        })
        .build();
    let url = spawn(server, "/api/ws").await;
    let mut ws = connect(&url).await;

    let answer = call(&mut ws, "p1", "panic.upstream", json!({})).await;
    assert!(answer.get("error").is_none(), "{answer}");
    let op_id = answer["result"]["op_id"].as_str().unwrap().to_string();

    let outcome = collect_settled(&mut ws, &op_id, Duration::from_secs(3)).await;
    assert_eq!(outcome["status"], "failed");
    assert_eq!(outcome["error"]["code"], error_codes::INTERNAL_ERROR);
    assert!(
        outcome["error"]["message"]
            .as_str()
            .unwrap()
            .contains("panicked"),
        "{outcome}"
    );
}

/// The policy in one negative control: the framework's own stall guard still
/// fires for a handler that blocks, which is exactly the failure mode
/// deferring exists to avoid.
#[tokio::test]
async fn a_blocking_handler_still_hits_the_stall_guard() {
    let server = RpcServer::builder()
        .config(test_config())
        .method_ctx("blocking.upstream", |_ctx: RpcRequestCtx| async move {
            // The upstream the handler should never have waited on.
            tokio::time::sleep(Duration::from_secs(5)).await;
            Ok(json!({"never": "reached"}))
        })
        .build();
    let url = spawn(server, "/api/ws").await;
    let mut ws = connect(&url).await;

    let started = Instant::now();
    let frame = call(&mut ws, "b1", "blocking.upstream", json!({})).await;
    assert!(started.elapsed() < Duration::from_secs(2));
    assert_eq!(frame["error"]["code"], error_codes::INTERNAL_ERROR);
    assert_eq!(frame["error"]["data"]["stalled"], true);
}
