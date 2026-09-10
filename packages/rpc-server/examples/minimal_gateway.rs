//! Minimal gateway-shaped service on the PLANA service profile.
//!
//! Run with `cargo run -p plana-rpc-server --example minimal_gateway` and
//! point a client at `ws://127.0.0.1:8092/api/ws` (or POST the same JSON to
//! `http://127.0.0.1:8092/api/ws`). It exercises the pieces a hosted
//! gateway needs: connection-bound sessions minted from one-shot tickets,
//! per-connection auth, heartbeat, and progress notifications on a long
//! operation.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::sync::Mutex;

use plana::jsonrpc::JsonRpcError;
use plana_rpc_server::{RpcServer, RpcServerConfig};

/// Per-connection identity produced by the auth hook.
struct ConnIdentity {
    user: String,
}

#[derive(Default)]
struct AppState {
    /// One-shot tickets issued by a (fictional) OAuth callback. A ticket is
    /// consumed exactly once, at session mint.
    tickets: Mutex<HashMap<String, String>>,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState::default());

    // Seed a demo ticket as if an OAuth callback had just landed.
    state
        .tickets
        .lock()
        .await
        .insert("demo-ticket-0001".into(), "user-7f3a".into());

    let state_for_auth = state.clone();
    let state_for_open = state.clone();

    let app = RpcServer::builder()
        .config(RpcServerConfig {
            idle_timeout: Duration::from_secs(45),
            ..RpcServerConfig::default()
        })
        // Connection auth here is the ticket presentation; production
        // deployments authenticate the upgrade (bearer token, mTLS, …) and
        // mint sessions from `rescue.open_session` instead.
        .auth(move |_headers, _uri| {
            let state = state_for_auth.clone();
            async move {
                let _ = state;
                Ok(Arc::new(ConnIdentity {
                    user: "anonymous".into(),
                }) as plana_rpc_server::AuthContext)
            }
        })
        .method_ctx("whoami", |ctx| async move {
            // Downcast the per-connection identity established at upgrade.
            match ctx.auth::<ConnIdentity>() {
                Some(identity) => Ok(json!({"user": identity.user})),
                None => Err(JsonRpcError::internal_error("missing identity")),
            }
        })
        .method_ctx("rescue.open_session", move |ctx| {
            let state = state_for_open.clone();
            async move {
                let ticket = ctx
                    .params
                    .get("ticket")
                    .and_then(Value::as_str)
                    .ok_or_else(|| JsonRpcError::invalid_params("ticket required"))?;
                let mut tickets = state.tickets.lock().await;
                let user = tickets
                    .remove(ticket)
                    .ok_or_else(|| JsonRpcError::new(-32005, "unknown or consumed ticket"))?;
                // Sessions are connection-bound: the handle lives exactly
                // as long as this socket, never serialized to the client.
                Ok(json!({"session": {"user": user, "bound": "connection"}}))
            }
        })
        .method_ctx("rescue.diagnose", |ctx| async move {
            let bundle = ctx.params.get("bundle").cloned().unwrap_or(Value::Null);
            for step in 1..=3 {
                tokio::time::sleep(Duration::from_millis(200)).await;
                let _ = ctx
                    .outbound
                    .notify("rescue.diagnose.progress", json!({"step": step}))
                    .await;
            }
            Ok(json!({"diagnosis": {"root_cause": "demo", "bundle": bundle}}))
        })
        .build()
        .mount_at("/api/ws");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8092")
        .await
        .unwrap();
    println!("minimal gateway listening on ws://127.0.0.1:8092/api/ws");
    axum::serve(listener, app).await.unwrap();
}
