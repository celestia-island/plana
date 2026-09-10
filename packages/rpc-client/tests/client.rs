//! Integration tests: the Rust client against a real `plana-rpc-server`
//! endpoint — the two halves of the SDK dogfooding each other.

use std::time::Duration;

use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

use plana::jsonrpc::error_codes;
use plana_rpc_client::{post_rpc, ConnectionState, RpcClient, RpcClientConfig, RpcError};
use plana_rpc_server::{RpcServer, RpcServerConfig};

/// Bind on a random port and return `(base_url, port)`.
async fn spawn(server: RpcServer) -> (String, u16) {
    let app = server.mount_at("/api/ws");
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let port = addr.port();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("ws://{addr}/api/ws"), port)
}

fn echo_server() -> RpcServer {
    RpcServer::builder()
        .method("echo", |params: Value| async move { Ok(params) })
        .build()
}

async fn wait_for_state(
    client: &RpcClient,
    want: ConnectionState,
    within: Duration,
) -> ConnectionState {
    let deadline = tokio::time::Instant::now() + within;
    loop {
        if client.state() == want {
            return want;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "state did not reach {want:?}, now {:?}",
            client.state()
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test]
async fn calls_correlate_and_map_errors() {
    let (url, _port) = spawn(echo_server()).await;
    let client = RpcClient::builder().url(&url).build();

    let result = client
        .call("echo", json!({"n": 42}))
        .await
        .expect("echo should succeed");
    assert_eq!(result["n"], 42);

    let err = client.call("no.such.method", json!({})).await.unwrap_err();
    match &err {
        RpcError::Rpc { code, .. } => assert_eq!(*code, error_codes::METHOD_NOT_FOUND),
        other => panic!("expected Rpc error, got {other:?}"),
    }
}

#[tokio::test]
async fn concurrent_calls_correlate_their_own_ids() {
    let (url, _port) = spawn(echo_server()).await;
    let client = RpcClient::builder().url(&url).build();

    let mut tasks = tokio::task::JoinSet::new();
    for i in 0..20 {
        let client = client.clone();
        tasks.spawn(async move {
            let result = client.call("echo", json!({"i": i})).await.unwrap();
            assert_eq!(result["i"], i);
        });
    }
    while let Some(joined) = tasks.join_next().await {
        joined.unwrap();
    }
}

#[tokio::test]
async fn call_timeout_bounds_slow_handlers() {
    let server = RpcServer::builder()
        .config(RpcServerConfig {
            dispatch_stall_limit: Duration::from_secs(8),
            ..RpcServerConfig::default()
        })
        .method("slow", |_p: Value| async move {
            tokio::time::sleep(Duration::from_secs(5)).await;
            Ok(json!("late"))
        })
        .build();
    let (url, _port) = spawn(server).await;

    let client = RpcClient::builder()
        .url(&url)
        .config(RpcClientConfig {
            call_timeout: Duration::from_millis(150),
            ..RpcClientConfig::default()
        })
        .build();

    match client.call("slow", json!({})).await.unwrap_err() {
        RpcError::Timeout(d) => assert_eq!(d, Duration::from_millis(150)),
        other => panic!("expected Timeout, got {other:?}"),
    }
}

#[tokio::test]
async fn server_notifications_reach_subscribers() {
    let server = RpcServer::builder()
        .method_ctx("watch", |ctx: plana_rpc_server::RpcRequestCtx| async move {
            ctx.outbound
                .notify("watch.event", json!({"tick": 7}))
                .await
                .map_err(|_| plana::jsonrpc::JsonRpcError::internal_error("client gone"))?;
            Ok(json!({"ok": true}))
        })
        .build();
    let (url, _port) = spawn(server).await;

    let client = RpcClient::builder().url(&url).build();
    let mut notifications = client.subscribe();

    client.call("watch", json!({})).await.unwrap();

    let notification = tokio::time::timeout(Duration::from_secs(2), notifications.recv())
        .await
        .expect("notification should arrive")
        .unwrap();
    assert_eq!(notification["method"], "watch.event");
    assert_eq!(notification["params"]["tick"], 7);
}

#[tokio::test]
async fn heartbeat_keeps_an_idle_server_from_closing() {
    // The server closes idle connections at 300ms; a client heartbeating
    // every 60ms must keep it alive indefinitely.
    let server = RpcServer::builder()
        .config(RpcServerConfig {
            idle_timeout: Duration::from_millis(300),
            ..RpcServerConfig::default()
        })
        .method("echo", |p: Value| async move { Ok(p) })
        .build();
    let (url, _port) = spawn(server).await;

    let client = RpcClient::builder()
        .url(&url)
        .config(RpcClientConfig {
            heartbeat_interval: Duration::from_millis(60),
            heartbeat_timeout: Duration::from_secs(2),
            ..RpcClientConfig::default()
        })
        .build();

    tokio::time::sleep(Duration::from_millis(800)).await;
    let result = client.call("echo", json!({"alive": true})).await.unwrap();
    assert_eq!(result["alive"], true);
}

#[tokio::test]
async fn watchdog_cycles_a_silent_connection() {
    // Server with the heartbeat service disabled: it never acks, so the
    // client watchdog must declare the connection dead on its own.
    let server = RpcServer::builder()
        .config(RpcServerConfig {
            idle_timeout: Duration::from_secs(30),
            heartbeat: false,
            ..RpcServerConfig::default()
        })
        .method("echo", |p: Value| async move { Ok(p) })
        .build();
    let (url, _port) = spawn(server).await;

    let client = RpcClient::builder()
        .url(&url)
        .config(RpcClientConfig {
            heartbeat_interval: Duration::from_millis(50),
            heartbeat_timeout: Duration::from_millis(250),
            reconnect_initial: Duration::from_millis(40),
            ..RpcClientConfig::default()
        })
        .build();

    wait_for_state(
        &client,
        ConnectionState::Reconnecting,
        Duration::from_secs(5),
    )
    .await;
    // Calls during the outage fail fast with Closed, not a hang.
    match client.call("echo", json!({})).await.unwrap_err() {
        RpcError::Closed => {}
        other => panic!("expected Closed during outage, got {other:?}"),
    }
}

#[tokio::test]
async fn force_reconnect_reestablishes_and_rejects_pending() {
    let server = RpcServer::builder()
        .config(RpcServerConfig {
            dispatch_stall_limit: Duration::from_secs(8),
            ..RpcServerConfig::default()
        })
        .method("slow", |_p: Value| async move {
            tokio::time::sleep(Duration::from_millis(400)).await;
            Ok(json!("late"))
        })
        .method("echo", |p: Value| async move { Ok(p) })
        .build();
    let (url, _port) = spawn(server).await;

    let client = RpcClient::builder()
        .url(&url)
        .config(RpcClientConfig {
            call_timeout: Duration::from_secs(5),
            reconnect_initial: Duration::from_millis(40),
            ..RpcClientConfig::default()
        })
        .build();

    // Sanity: connected and working.
    client.call("echo", json!({"n": 1})).await.unwrap();

    // A slow in-flight call is rejected when the connection is forced over.
    let slow = {
        let client = client.clone();
        tokio::spawn(async move { client.call("slow", json!({})).await })
    };
    tokio::time::sleep(Duration::from_millis(50)).await;
    client.force_reconnect();

    match slow.await.unwrap().unwrap_err() {
        RpcError::Closed => {}
        other => panic!("expected Closed for pending call, got {other:?}"),
    }

    wait_for_state(&client, ConnectionState::Connected, Duration::from_secs(5)).await;
    let result = client.call("echo", json!({"n": 2})).await.unwrap();
    assert_eq!(result["n"], 2);
}

#[tokio::test]
async fn reconnect_budget_exhaustion_enters_failed_state() {
    // Reserve an ephemeral port and never serve on it.
    let probe = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = probe.local_addr().unwrap().port();
    drop(probe);

    let client = RpcClient::builder()
        .url(format!("ws://127.0.0.1:{port}/api/ws"))
        .config(RpcClientConfig {
            connect_timeout: Duration::from_millis(100),
            reconnect_initial: Duration::from_millis(30),
            max_reconnect_attempts: Some(2),
            ..RpcClientConfig::default()
        })
        .build();

    wait_for_state(&client, ConnectionState::Failed, Duration::from_secs(5)).await;
    match client.call("echo", json!({})).await.unwrap_err() {
        RpcError::Closed => {}
        other => panic!("expected Closed in Failed state, got {other:?}"),
    }
}

#[tokio::test]
async fn heartbeat_acks_do_not_surface_as_notifications() {
    let (url, _port) = spawn(echo_server()).await;
    let client = RpcClient::builder()
        .url(&url)
        .config(RpcClientConfig {
            heartbeat_interval: Duration::from_millis(50),
            ..RpcClientConfig::default()
        })
        .build();
    let mut notifications = client.subscribe();

    // Produce traffic so several heartbeat cycles elapse.
    client.call("echo", json!({})).await.unwrap();
    tokio::time::sleep(Duration::from_millis(400)).await;

    assert!(
        notifications.try_recv().is_err(),
        "heartbeat acks must stay internal"
    );
}

#[tokio::test]
async fn http_post_fallback_round_trip() {
    let (url, _port) = spawn(echo_server()).await;
    let http_url = url.replace("ws://", "http://");

    let result = post_rpc(&http_url, "echo", json!({"x": 5}), Duration::from_secs(2))
        .await
        .unwrap();
    assert_eq!(result["x"], 5);

    let err = post_rpc(&http_url, "missing", json!({}), Duration::from_secs(2))
        .await
        .unwrap_err();
    match err {
        RpcError::Rpc { code, .. } => assert_eq!(code, error_codes::METHOD_NOT_FOUND),
        other => panic!("expected Rpc error, got {other:?}"),
    }
}

#[tokio::test]
async fn raw_wire_shape_matches_the_profile() {
    // Independent wire-level witness: a hand-rolled socket client pins the
    // envelope shape both sides of this SDK agree on.
    let (url, _port) = spawn(echo_server()).await;

    let (mut ws, _resp) = tokio_tungstenite::connect_async(&url).await.unwrap();
    ws.send(Message::Text(
        json!({"jsonrpc": "2.0", "id": "w-1", "method": "echo", "params": {"w": 1}})
            .to_string()
            .into(),
    ))
    .await
    .unwrap();

    let reply = tokio::time::timeout(Duration::from_secs(2), ws.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let Message::Text(text) = reply else {
        panic!("expected text frame");
    };
    let value: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(value["jsonrpc"], "2.0");
    assert_eq!(value["id"], "w-1");
    assert_eq!(value["result"]["w"], 1);
    ws.close(None).await.unwrap();
}
