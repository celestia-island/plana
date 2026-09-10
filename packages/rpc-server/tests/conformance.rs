//! Conformance suite for the PLANA service profile.
//!
//! These tests are the executable form of `docs/en/rpc/service-profile.md`.
//! Any server claiming conformance — including third-party
//! implementations built on this crate — must pass an equivalent suite.

use std::time::Duration;

use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

use plana::jsonrpc::error_codes;
use plana_rpc_server::{RpcServer, RpcServerConfig};

use serde_json::{json, Value};

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

/// Read the next JSON object frame off the wire.
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

fn base_server() -> RpcServer {
    RpcServer::builder()
        .config(RpcServerConfig {
            // Keep the test cycle fast; production defaults are exercised
            // implicitly everywhere else.
            idle_timeout: Duration::from_secs(5),
            dispatch_stall_limit: Duration::from_secs(2),
            ..RpcServerConfig::default()
        })
        .method("echo", |params: Value| async move { Ok(params) })
        .method("fail.invalid_params", |_params: Value| async move {
            Err(plana::jsonrpc::JsonRpcError::invalid_params("bad input"))
        })
        .build()
}

#[tokio::test]
async fn request_response_round_trip_with_id_echo() {
    let url = spawn(base_server(), "/api/ws").await;
    let mut ws = connect(&url).await;

    send_json(
        &mut ws,
        json!({"jsonrpc": "2.0", "id": "req-1", "method": "echo", "params": {"hello": "world"}}),
    )
    .await;

    let resp = recv_json(&mut ws).await;
    assert_eq!(resp["id"], "req-1");
    assert_eq!(resp["result"]["hello"], "world");
    assert!(resp.get("error").is_none());
}

#[tokio::test]
async fn uuid_v7_string_ids_are_accepted_verbatim() {
    let url = spawn(base_server(), "/api/ws").await;
    let mut ws = connect(&url).await;

    // A canonical UUIDv7 (time-ordered) string id.
    let id = "01912345-6789-7abc-8def-0123456789ab";
    send_json(
        &mut ws,
        json!({"jsonrpc": "2.0", "id": id, "method": "echo", "params": null}),
    )
    .await;

    let resp = recv_json(&mut ws).await;
    assert_eq!(resp["id"], id);
}

#[tokio::test]
async fn unknown_method_answers_32601() {
    let url = spawn(base_server(), "/api/ws").await;
    let mut ws = connect(&url).await;

    send_json(
        &mut ws,
        json!({"jsonrpc": "2.0", "id": 7, "method": "no.such.method"}),
    )
    .await;

    let resp = recv_json(&mut ws).await;
    assert_eq!(resp["id"], 7);
    assert_eq!(resp["error"]["code"], error_codes::METHOD_NOT_FOUND);
}

#[tokio::test]
async fn handler_error_maps_to_jsonrpc_error_with_data() {
    let url = spawn(base_server(), "/api/ws").await;
    let mut ws = connect(&url).await;

    send_json(
        &mut ws,
        json!({"jsonrpc": "2.0", "id": "e1", "method": "fail.invalid_params", "params": {}}),
    )
    .await;

    let resp = recv_json(&mut ws).await;
    assert_eq!(resp["error"]["code"], error_codes::INVALID_PARAMS);
    assert!(resp["error"]["message"]
        .as_str()
        .unwrap()
        .contains("bad input"));
}

#[tokio::test]
async fn parse_error_answers_null_id_and_keeps_the_connection() {
    let url = spawn(base_server(), "/api/ws").await;
    let mut ws = connect(&url).await;

    // Not JSON at all.
    ws.send(Message::Text("{not json".into())).await.unwrap();
    let resp = recv_json(&mut ws).await;
    assert_eq!(resp["id"], Value::Null);
    assert_eq!(resp["error"]["code"], error_codes::PARSE_ERROR);

    // The connection must still work afterwards.
    send_json(
        &mut ws,
        json!({"jsonrpc": "2.0", "id": "after", "method": "echo", "params": 1}),
    )
    .await;
    let resp = recv_json(&mut ws).await;
    assert_eq!(resp["id"], "after");
    assert_eq!(resp["result"], 1);
}

#[tokio::test]
async fn batch_input_is_rejected_as_invalid_request() {
    let url = spawn(base_server(), "/api/ws").await;
    let mut ws = connect(&url).await;

    send_json(
        &mut ws,
        json!([{"jsonrpc": "2.0", "id": 1, "method": "echo"}, {"jsonrpc": "2.0", "id": 2, "method": "echo"}]),
    )
    .await;

    let resp = recv_json(&mut ws).await;
    assert_eq!(resp["id"], Value::Null);
    assert_eq!(resp["error"]["code"], error_codes::INVALID_REQUEST);
    assert_eq!(resp["error"]["data"]["reason"], "batch_not_supported");
}

#[tokio::test]
async fn heartbeat_notification_is_acked_on_the_control_lane() {
    let url = spawn(base_server(), "/api/ws").await;
    let mut ws = connect(&url).await;

    send_json(
        &mut ws,
        json!({"jsonrpc": "2.0", "method": "Base.Heartbeat"}),
    )
    .await;

    let resp = recv_json(&mut ws).await;
    assert_eq!(resp["method"], "Base.HeartbeatAck");
    assert!(resp.get("id").is_none());
}

#[tokio::test]
async fn heartbeat_ack_overtakes_a_saturated_data_lane() {
    // A slow handler holds the data lane busy; the heartbeat ack must
    // arrive before the slow response.
    let server = RpcServer::builder()
        .config(RpcServerConfig {
            idle_timeout: Duration::from_secs(5),
            dispatch_stall_limit: Duration::from_secs(5),
            ..RpcServerConfig::default()
        })
        .method("slow", |_params: Value| async move {
            tokio::time::sleep(Duration::from_millis(300)).await;
            Ok(json!({"done": true}))
        })
        .build();
    let url = spawn(server, "/api/ws").await;
    let mut ws = connect(&url).await;

    send_json(
        &mut ws,
        json!({"jsonrpc": "2.0", "id": "slow-1", "method": "slow"}),
    )
    .await;
    // Give the dispatcher a moment to enter the slow handler, then beat.
    tokio::time::sleep(Duration::from_millis(50)).await;
    send_json(
        &mut ws,
        json!({"jsonrpc": "2.0", "method": "Base.Heartbeat"}),
    )
    .await;

    let first = recv_json(&mut ws).await;
    assert_eq!(
        first["method"], "Base.HeartbeatAck",
        "control lane must deliver the ack before the slow data-lane response"
    );
    let second = recv_json(&mut ws).await;
    assert_eq!(second["id"], "slow-1");
    assert_eq!(second["result"]["done"], true);
}

#[tokio::test]
async fn stalled_dispatch_is_cancelled_with_a_structured_error() {
    let server = RpcServer::builder()
        .config(RpcServerConfig {
            idle_timeout: Duration::from_secs(5),
            dispatch_stall_limit: Duration::from_millis(150),
            ..RpcServerConfig::default()
        })
        .method("hang", |_params: Value| async move {
            tokio::time::sleep(Duration::from_secs(30)).await;
            Ok(json!("never"))
        })
        .build();
    let url = spawn(server, "/api/ws").await;
    let mut ws = connect(&url).await;

    let started = std::time::Instant::now();
    send_json(
        &mut ws,
        json!({"jsonrpc": "2.0", "id": "h1", "method": "hang"}),
    )
    .await;

    let resp = recv_json(&mut ws).await;
    assert!(started.elapsed() < Duration::from_secs(5), "must not hang");
    assert_eq!(resp["id"], "h1");
    assert_eq!(resp["error"]["code"], error_codes::INTERNAL_ERROR);
    assert_eq!(resp["error"]["data"]["stalled"], true);
}

#[tokio::test]
async fn idle_connection_is_closed_with_code_4000() {
    let server = RpcServer::builder()
        .config(RpcServerConfig {
            idle_timeout: Duration::from_millis(120),
            dispatch_stall_limit: Duration::from_secs(1),
            heartbeat: false,
            ..RpcServerConfig::default()
        })
        .method("echo", |p: Value| async move { Ok(p) })
        .build();
    let url = spawn(server, "/api/ws").await;
    let mut ws = connect(&url).await;

    let msg = tokio::time::timeout(Duration::from_secs(3), ws.next())
        .await
        .expect("server must close the idle connection")
        .unwrap()
        .unwrap();

    match msg {
        Message::Close(Some(frame)) => assert_eq!(u16::from(frame.code), 4000),
        other => panic!("expected close frame 4000, got {:?}", other),
    }
}

#[tokio::test]
async fn auth_denial_refuses_the_upgrade_with_http_401() {
    let server = RpcServer::builder()
        .auth(
            |_headers: &axum::http::HeaderMap, _uri: &axum::http::Uri| async move {
                Err(plana::jsonrpc::JsonRpcError::new(
                    error_codes::AUTH_ERROR,
                    "invalid connection token",
                ))
            },
        )
        .method("echo", |p: Value| async move { Ok(p) })
        .build();
    let url = spawn(server, "/api/ws").await;

    let err = tokio_tungstenite::connect_async(&url).await.unwrap_err();
    match err {
        tokio_tungstenite::tungstenite::Error::Http(resp) => {
            assert_eq!(resp.status(), 401);
        }
        other => panic!("expected HTTP rejection, got {:?}", other),
    }
}

#[tokio::test]
async fn request_guard_denial_answers_32005_without_dispatch() {
    let server = RpcServer::builder()
        .guard(|_ctx, method: &str, _params: &Value| {
            let method = method.to_string();
            async move {
                if method == "privileged.op" {
                    Err(plana::jsonrpc::JsonRpcError::new(
                        error_codes::AUTH_ERROR,
                        "guard denied",
                    ))
                } else {
                    Ok(())
                }
            }
        })
        .method("privileged.op", |_p: Value| async move {
            panic!("guarded method must never dispatch");
        })
        .build();
    let url = spawn(server, "/api/ws").await;
    let mut ws = connect(&url).await;

    send_json(
        &mut ws,
        json!({"jsonrpc": "2.0", "id": "g1", "method": "privileged.op"}),
    )
    .await;

    let resp = recv_json(&mut ws).await;
    assert_eq!(resp["error"]["code"], error_codes::AUTH_ERROR);
}

#[tokio::test]
async fn http_post_fallback_answers_on_the_same_method_map() {
    // The workspace reqwest ships rustls without a crypto provider.
    let _ = rustls::crypto::ring::default_provider().install_default();
    let url = spawn(base_server(), "/api/ws").await;
    let http_url = url.replace("ws://", "http://");

    let client = reqwest::Client::new();
    let resp = client
        .post(&http_url)
        .json(&json!({"jsonrpc": "2.0", "id": "post-1", "method": "echo", "params": {"x": 9}}))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["id"], "post-1");
    assert_eq!(body["result"]["x"], 9);
}

#[tokio::test]
async fn handler_notifications_reach_the_client() {
    let server = RpcServer::builder()
        .config(RpcServerConfig {
            idle_timeout: Duration::from_secs(5),
            dispatch_stall_limit: Duration::from_secs(2),
            ..RpcServerConfig::default()
        })
        .method_ctx("watch", |ctx: plana_rpc_server::RpcRequestCtx| async move {
            ctx.outbound
                .notify("watch.event", json!({"tick": 1}))
                .await
                .map_err(|_| plana::jsonrpc::JsonRpcError::internal_error("client gone"))?;
            Ok(json!({"watching": true}))
        })
        .build();
    let url = spawn(server, "/api/ws").await;
    let mut ws = connect(&url).await;

    send_json(
        &mut ws,
        json!({"jsonrpc": "2.0", "id": "w1", "method": "watch"}),
    )
    .await;

    // The notification is queued from inside the handler, so the response
    // and notification ordering is data-lane FIFO: notification enqueued
    // before the response send.
    let first = recv_json(&mut ws).await;
    assert_eq!(first["method"], "watch.event");
    let second = recv_json(&mut ws).await;
    assert_eq!(second["id"], "w1");
    assert_eq!(second["result"]["watching"], true);
}
