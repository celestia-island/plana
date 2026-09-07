//! Admission hooks: per-connection authentication and per-request guards.

use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::http::{HeaderMap, Uri};
use serde_json::Value;

use plana_jsonrpc::JsonRpcError;

/// Opaque per-connection identity established by the auth hook. Handlers
/// downcast it via [`crate::RpcRequestCtx::auth`].
pub type AuthContext = Arc<dyn Any + Send + Sync>;

/// Runs once per WebSocket upgrade (and once per HTTP POST fallback
/// request). A denial aborts the handshake with HTTP 401 carrying the
/// returned JSON-RPC error body.
///
/// The boxed future is `'static`: hooks must extract owned data (tokens,
/// claims) from the header map before awaiting.
pub type ConnectionAuthFn = Arc<
    dyn Fn(
            &HeaderMap,
            &Uri,
        ) -> Pin<Box<dyn Future<Output = Result<AuthContext, JsonRpcError>> + Send>>
        + Send
        + Sync,
>;

/// Runs before every method dispatch on an authenticated connection. A
/// denial answers the request with the returned JSON-RPC error (fleet
/// convention: code `-32005` for auth failures).
///
/// As with [`ConnectionAuthFn`], the boxed future is `'static`.
pub type RequestGuardFn = Arc<
    dyn Fn(
            AuthContext,
            &str,
            &Value,
        ) -> Pin<Box<dyn Future<Output = Result<(), JsonRpcError>> + Send>>
        + Send
        + Sync,
>;
