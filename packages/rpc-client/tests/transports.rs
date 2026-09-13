//! Integration tests for the transport matrix beyond plaintext TCP
//! WebSocket: Unix domain sockets (`ipc://`) and TLS WebSockets (`wss://`,
//! behind the `tls` feature).
//!
//! The `ipc://` suite reuses the real `plana-rpc-server` over a
//! `tokio::net::UnixListener` — axum serves the same router on a Unix
//! socket, so the client is exercised end-to-end against the other half of
//! the SDK, exactly like the TCP suite in `client.rs`. The `wss://` suite
//! stands up a hand-rolled TLS echo peer with a throwaway self-signed
//! certificate (rcgen + ring): what it pins is transport-level behaviour
//! (default-dial rejects untrusted certificates; the danger hook connects
//! and echoes), not JSON-RPC semantics those TCP tests already cover.

use std::time::Duration;

use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::Message;

use plana_rpc_client::{ConnectionState, RpcClient, RpcClientConfig, RpcError};
use plana_rpc_server::RpcServer;

fn echo_server() -> RpcServer {
    RpcServer::builder()
        .method("echo", |params: Value| async move { Ok(params) })
        .build()
}

/// Answer one JSON-RPC frame the way the client expects: requests get
/// their params echoed as the result, heartbeats get an ack. Used by the
/// hand-rolled peers below (the wss server, the replaceable ipc server) —
/// what those pin is transport behaviour, not rpc semantics.
fn echo_rpc_reply(frame: &str) -> Option<String> {
    let value: Value = serde_json::from_str(frame).ok()?;
    match (value.get("id"), value.get("method").and_then(Value::as_str)) {
        (Some(id), _) => Some(
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": value.get("params").cloned().unwrap_or(Value::Null),
            })
            .to_string(),
        ),
        (None, Some("Base.Heartbeat")) => {
            Some(json!({"jsonrpc": "2.0", "method": "Base.HeartbeatAck"}).to_string())
        }
        (None, _) => None,
    }
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

// ── ipc:// — Unix domain sockets ─────────────────────────────────────

mod ipc {
    #![cfg(unix)]

    use super::*;
    use plana::jsonrpc::error_codes;
    use plana_rpc_server::RpcServerConfig;
    use serde_json::json;
    use tokio::net::UnixListener;

    /// Serve a `RpcServer` on a Unix socket; the returned task ends when
    /// aborted or when the accept loop fails.
    async fn serve(server: RpcServer, path: &std::path::Path) -> tokio::task::JoinHandle<()> {
        let app = server.mount_at("/");
        let listener = UnixListener::bind(path).expect("bind unix socket");
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("unix ws server");
        })
    }

    #[tokio::test]
    async fn calls_correlate_and_map_errors() {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("rpc.sock");
        serve(echo_server(), &socket).await;

        let url = format!("ipc://{}", socket.display());
        let client = RpcClient::builder().url(&url).build();

        let result = client
            .call("echo", json!({"n": 42}))
            .await
            .expect("echo should succeed over ipc");
        assert_eq!(result["n"], 42);

        let err = client.call("no.such.method", json!({})).await.unwrap_err();
        match &err {
            RpcError::Rpc { code, .. } => assert_eq!(*code, error_codes::METHOD_NOT_FOUND),
            other => panic!("expected Rpc error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn watchdog_cycles_a_silent_connection() {
        // Server with the heartbeat service disabled: it never acks, so the
        // client watchdog must declare the connection dead on its own —
        // same contract as over TCP.
        let server = RpcServer::builder()
            .config(RpcServerConfig {
                idle_timeout: Duration::from_secs(30),
                heartbeat: false,
                ..RpcServerConfig::default()
            })
            .method("echo", |p: Value| async move { Ok(p) })
            .build();
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("rpc.sock");
        serve(server, &socket).await;

        let client = RpcClient::builder()
            .url(format!("ipc://{}", socket.display()))
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
        match client.call("echo", json!({})).await.unwrap_err() {
            RpcError::Closed => {}
            other => panic!("expected Closed during outage, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn watchdog_stays_disarmed_when_heartbeats_are_disabled() {
        // Regression (#339 semantics) on the ipc transport: with
        // `heartbeat: false` the client sends nothing the server should
        // ack, so inbound silence carries no signal. A quiet connection
        // several watchdog windows old must NOT cycle — only
        // transport-level failures may.
        let server = RpcServer::builder()
            .config(RpcServerConfig {
                idle_timeout: Duration::from_secs(30),
                heartbeat: false,
                ..RpcServerConfig::default()
            })
            .method("echo", |p: Value| async move { Ok(p) })
            .build();
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("rpc.sock");
        serve(server, &socket).await;

        let client = RpcClient::builder()
            .url(format!("ipc://{}", socket.display()))
            .config(RpcClientConfig {
                heartbeat: false,
                heartbeat_timeout: Duration::from_millis(200),
                ..RpcClientConfig::default()
            })
            .build();

        wait_for_state(&client, ConnectionState::Connected, Duration::from_secs(5)).await;
        tokio::time::sleep(Duration::from_millis(1000)).await;
        assert_eq!(
            client.state(),
            ConnectionState::Connected,
            "an idle ipc connection must survive with heartbeats disabled"
        );

        let result = client.call("echo", json!({"still": true})).await.unwrap();
        assert_eq!(result["still"], true);
    }

    #[tokio::test]
    async fn server_loss_cycles_and_reconnects_to_a_replacement() {
        // Kill the serving socket, watch the client cycle through the
        // reconnect state machine, then serve again on the same path and
        // prove recovery — the redial path over ipc must be the same loop
        // TCP uses. The peer below serves its one connection inline, so
        // aborting the task tears the live connection down with it (axum
        // hands accepted connections to independent tasks, which is why
        // the real RpcServer cannot stage this scenario deterministically).
        async fn spawn_replaceable_echo(path: &std::path::Path) -> tokio::task::JoinHandle<()> {
            let listener = UnixListener::bind(path).expect("bind unix socket");
            tokio::spawn(async move {
                loop {
                    let Ok((stream, _)) = listener.accept().await else {
                        return;
                    };
                    let Ok(mut ws) = tokio_tungstenite::accept_async(stream).await else {
                        continue;
                    };
                    while let Some(Ok(msg)) = ws.next().await {
                        let Message::Text(text) = msg else {
                            continue;
                        };
                        let Some(reply) = echo_rpc_reply(&text) else {
                            continue;
                        };
                        if ws.send(Message::Text(reply.into())).await.is_err() {
                            break;
                        }
                    }
                }
            })
        }

        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("rpc.sock");
        let url = format!("ipc://{}", socket.display());

        let server = spawn_replaceable_echo(&socket).await;
        let client = RpcClient::builder()
            .url(&url)
            .config(RpcClientConfig {
                reconnect_initial: Duration::from_millis(40),
                ..RpcClientConfig::default()
            })
            .build();

        let result = client.call("echo", json!({"n": 1})).await.unwrap();
        assert_eq!(result["n"], 1);

        server.abort();
        wait_for_state(
            &client,
            ConnectionState::Reconnecting,
            Duration::from_secs(5),
        )
        .await;
        match client.call("echo", json!({})).await.unwrap_err() {
            RpcError::Closed => {}
            other => panic!("expected Closed during outage, got {other:?}"),
        }

        std::fs::remove_file(&socket).expect("clear the dead socket file");
        spawn_replaceable_echo(&socket).await;
        wait_for_state(&client, ConnectionState::Connected, Duration::from_secs(5)).await;
        let result = client.call("echo", json!({"n": 2})).await.unwrap();
        assert_eq!(result["n"], 2);
    }

    #[tokio::test]
    async fn reconnect_budget_exhaustion_enters_failed_state() {
        // Dial a socket path that exists on no server: the budget must run
        // out and leave the client in Failed, failing calls fast.
        let dir = tempfile::tempdir().unwrap();
        let url = format!("ipc://{}/missing.sock", dir.path().display());

        let client = RpcClient::builder()
            .url(&url)
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
}

// ── wss:// — TLS WebSockets (`tls` feature) ──────────────────────────

mod wss {
    #![cfg(feature = "tls")]

    use std::sync::Arc;

    use super::*;
    use plana_rpc_client::RpcClientBuilder;
    use tokio::net::TcpListener;

    /// Serve a TLS WebSocket echo peer bound to a random port, with a
    /// throwaway self-signed certificate no default trust store contains.
    async fn spawn_self_signed_wss() -> String {
        let key_pair = rcgen::KeyPair::generate().unwrap();
        let params = rcgen::CertificateParams::new(vec!["127.0.0.1".to_string()]).unwrap();
        let cert = params.self_signed(&key_pair).unwrap();

        let server_config = rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(
            vec![cert.der().clone()],
            rustls::pki_types::PrivateKeyDer::try_from(key_pair.serialize_der()).unwrap(),
        )
        .unwrap();
        let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(server_config));

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            loop {
                let Ok((tcp, _)) = listener.accept().await else {
                    return;
                };
                let acceptor = acceptor.clone();
                tokio::spawn(async move {
                    let Ok(tls) = acceptor.accept(tcp).await else {
                        return;
                    };
                    let Ok(mut ws) = tokio_tungstenite::accept_async(tls).await else {
                        return;
                    };
                    while let Some(Ok(msg)) = ws.next().await {
                        let Message::Text(text) = msg else {
                            continue;
                        };
                        let Some(reply) = echo_rpc_reply(&text) else {
                            continue;
                        };
                        if ws.send(Message::Text(reply.into())).await.is_err() {
                            break;
                        }
                    }
                });
            }
        });
        format!("wss://{addr}/api/ws")
    }

    #[tokio::test]
    async fn default_mode_refuses_an_untrusted_certificate() {
        // Safe by default: a self-signed certificate is in no trust store,
        // so the default dial must fail — never silently connect. With the
        // reconnect budget exhausted the client lands in Failed.
        RpcClientBuilder::install_ring_tls_provider();
        let url = spawn_self_signed_wss().await;

        let client = RpcClient::builder()
            .url(&url)
            .config(RpcClientConfig {
                connect_timeout: Duration::from_secs(2),
                reconnect_initial: Duration::from_millis(30),
                max_reconnect_attempts: Some(3),
                ..RpcClientConfig::default()
            })
            .build();

        wait_for_state(&client, ConnectionState::Failed, Duration::from_secs(10)).await;
        assert_ne!(
            client.state(),
            ConnectionState::Connected,
            "an untrusted certificate must never yield a connection"
        );
        match client.call("echo", json!({})).await.unwrap_err() {
            RpcError::Closed => {}
            other => panic!("expected Closed in Failed state, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn danger_hook_connects_to_a_self_signed_server() {
        // The explicit test hook is the one way in: connect to the same
        // self-signed peer and echo over the TLS connection.
        RpcClientBuilder::install_ring_tls_provider();
        let url = spawn_self_signed_wss().await;

        let client = RpcClient::builder()
            .url(&url)
            .danger_accept_invalid_certs(true)
            .config(RpcClientConfig {
                connect_timeout: Duration::from_secs(5),
                ..RpcClientConfig::default()
            })
            .build();

        let result = client
            .call("echo", json!({"n": 7}))
            .await
            .expect("echo should succeed over wss with the danger hook");
        assert_eq!(result["n"], 7);
    }
}
