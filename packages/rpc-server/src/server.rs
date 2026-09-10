//! Server builder, axum mounting, and the request context handed to
//! handlers.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::Value;
use tokio::sync::Semaphore;

use plana::jsonrpc::{Id, JsonRpcError, JsonRpcMessage, JsonRpcResponse};

use crate::auth::{AuthContext, ConnectionAuthFn, RequestGuardFn};
use crate::config::RpcServerConfig;
use crate::connection::{self, OutboundHandle};

/// Extension error codes of the service profile (beyond JSON-RPC standard
/// and the plana fleet codes in `plana::jsonrpc::error_codes`).
pub mod ext_error_codes {
    /// Admission refused: connection cap reached (HTTP 429 body).
    pub const SERVICE_BUSY: i64 = -32050;
    /// Dispatch exceeded the stall limit and was cancelled.
    pub const DISPATCH_STALLED: i64 = -32051;
}

/// Full-context handler signature. Handlers receive the method name, the
/// request params, the connection's auth context, and a notification handle
/// bound to the calling client.
pub type ServerHandlerFn = Arc<
    dyn Fn(RpcRequestCtx) -> Pin<Box<dyn Future<Output = Result<Value, JsonRpcError>> + Send>>
        + Send
        + Sync,
>;

/// Everything a handler sees about one incoming request.
pub struct RpcRequestCtx {
    /// The dispatched method name.
    pub method: Arc<str>,
    /// The request params (`null` when absent).
    pub params: Value,
    pub(crate) auth: AuthContext,
    /// Push notifications to this client on the data lane. Always fails on
    /// the HTTP POST fallback transport.
    pub outbound: OutboundHandle,
}

impl RpcRequestCtx {
    /// Downcast the connection auth context to the concrete type the
    /// server's auth hook produced.
    pub fn auth<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.auth.downcast_ref::<T>()
    }
}

/// Builder for [`RpcServer`].
pub struct RpcServerBuilder {
    config: RpcServerConfig,
    auth: Option<ConnectionAuthFn>,
    guard: Option<RequestGuardFn>,
    methods: HashMap<String, ServerHandlerFn>,
}

impl Default for RpcServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RpcServerBuilder {
    pub fn new() -> Self {
        Self {
            config: RpcServerConfig::default(),
            auth: None,
            guard: None,
            methods: HashMap::new(),
        }
    }

    /// Replace the whole config.
    pub fn config(mut self, config: RpcServerConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the per-connection auth hook. Runs on every WebSocket upgrade
    /// and every HTTP POST request; denial aborts with HTTP 401 carrying
    /// the returned JSON-RPC error. The returned future must be `'static`
    /// — extract owned data from the arguments before awaiting.
    pub fn auth<F, Fut>(mut self, check: F) -> Self
    where
        F: Fn(&HeaderMap, &Uri) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<AuthContext, JsonRpcError>> + Send + 'static,
    {
        self.auth = Some(Arc::new(move |h, u| Box::pin(check(h, u))));
        self
    }

    /// Set the per-request guard. Runs before every dispatch; denial
    /// answers the request with the returned error. The returned future
    /// must be `'static`.
    pub fn guard<F, Fut>(mut self, check: F) -> Self
    where
        F: Fn(AuthContext, &str, &Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), JsonRpcError>> + Send + 'static,
    {
        self.guard = Some(Arc::new(move |ctx, m, p| Box::pin(check(ctx, m, p))));
        self
    }

    /// Register a simple handler that only sees the params.
    pub fn method<F, Fut>(self, name: &str, handler: F) -> Self
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Value, JsonRpcError>> + Send + 'static,
    {
        self.method_ctx(name, move |ctx: RpcRequestCtx| handler(ctx.params))
    }

    /// Register a full-context handler.
    pub fn method_ctx<F, Fut>(mut self, name: &str, handler: F) -> Self
    where
        F: Fn(RpcRequestCtx) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Value, JsonRpcError>> + Send + 'static,
    {
        self.methods.insert(
            name.to_string(),
            Arc::new(move |ctx| Box::pin(handler(ctx))),
        );
        self
    }

    pub fn build(self) -> RpcServer {
        let max = self.config.max_connections.max(1);
        RpcServer {
            methods: Arc::new(self.methods),
            auth: self.auth,
            guard: self.guard,
            config: self.config,
            conn_permits: Arc::new(Semaphore::new(max)),
        }
    }
}

/// A configured strict WS JSON-RPC server. Cheap to clone behind an `Arc`;
/// mount with [`RpcServer::mount_at`] or [`RpcServer::into_router`].
pub struct RpcServer {
    pub(crate) methods: Arc<HashMap<String, ServerHandlerFn>>,
    pub(crate) auth: Option<ConnectionAuthFn>,
    pub(crate) guard: Option<RequestGuardFn>,
    pub(crate) config: RpcServerConfig,
    pub(crate) conn_permits: Arc<Semaphore>,
}

impl RpcServer {
    pub fn builder() -> RpcServerBuilder {
        RpcServerBuilder::new()
    }

    /// Mount the WS upgrade and HTTP POST fallback at `path` (e.g.
    /// `"/api/ws"`).
    pub fn mount_at(self, path: &str) -> Router {
        Router::new()
            .route(path, get(ws_endpoint).post(post_endpoint))
            .with_state(Arc::new(self))
    }

    /// Mount at `/` — nest the result under the desired prefix.
    pub fn into_router(self) -> Router {
        self.mount_at("/")
    }
}

fn error_response(status: StatusCode, err: JsonRpcError) -> Response {
    (status, Json(serde_json::to_value(err).unwrap_or_default())).into_response()
}

async fn ws_endpoint(
    State(server): State<Arc<RpcServer>>,
    ws: axum::extract::WebSocketUpgrade,
    headers: HeaderMap,
    uri: Uri,
) -> Response {
    let permit = match server.conn_permits.clone().try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => {
            return error_response(
                StatusCode::TOO_MANY_REQUESTS,
                JsonRpcError::new(ext_error_codes::SERVICE_BUSY, "connection cap reached"),
            );
        }
    };

    let conn_auth = match &server.auth {
        Some(check) => match check(&headers, &uri).await {
            Ok(ctx) => ctx,
            Err(err) => return error_response(StatusCode::UNAUTHORIZED, err),
        },
        None => Arc::new(()),
    };

    let config = server.config.clone();
    ws.max_message_size(config.max_message_bytes)
        .max_frame_size(config.max_frame_bytes)
        .on_upgrade(move |socket| connection::run(socket, server, conn_auth, permit))
}

/// HTTP POST fallback on the same method map. Parity with
/// `plana::jsonrpc::rpc_router`: app-level failures are HTTP 200 with a
/// JSON-RPC error body; only unparseable bodies yield HTTP 400.
async fn post_endpoint(
    State(server): State<Arc<RpcServer>>,
    headers: HeaderMap,
    uri: Uri,
    body: axum::body::Bytes,
) -> Response {
    let value: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                JsonRpcError::parse_error().with_data(Value::String(e.to_string())),
            );
        }
    };

    let message = match serde_json::from_value::<JsonRpcMessage>(value) {
        Ok(m) => m,
        Err(e) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                JsonRpcError::invalid_request().with_data(Value::String(e.to_string())),
            );
        }
    };

    let JsonRpcMessage::Request(req) = message else {
        return error_response(
            StatusCode::BAD_REQUEST,
            JsonRpcError::invalid_request().with_data(Value::String(
                "only single request objects are accepted".into(),
            )),
        );
    };

    let id = req.id.clone().unwrap_or(Id::Null);
    let params = req.params.unwrap_or(Value::Null);

    let conn_auth = match &server.auth {
        Some(check) => match check(&headers, &uri).await {
            Ok(ctx) => ctx,
            Err(err) => return error_response(StatusCode::UNAUTHORIZED, err),
        },
        None => Arc::new(()),
    };

    let response = dispatch_http(&server, conn_auth, &req.method, params, id).await;
    Json(serde_json::to_value(response).unwrap_or_default()).into_response()
}

async fn dispatch_http(
    server: &Arc<RpcServer>,
    conn_auth: AuthContext,
    method: &str,
    params: Value,
    id: Id,
) -> JsonRpcResponse {
    if let Some(guard) = &server.guard {
        if let Err(err) = guard(conn_auth.clone(), method, &params).await {
            return JsonRpcResponse::error(id, err);
        }
    }

    let Some(handler) = server.methods.get(method).cloned() else {
        return JsonRpcResponse::error(id, JsonRpcError::method_not_found(method));
    };

    let ctx = RpcRequestCtx {
        method: Arc::from(method),
        params,
        auth: conn_auth,
        // No server-initiated direction on this transport.
        outbound: OutboundHandle::closed(),
    };

    match tokio::time::timeout(server.config.dispatch_stall_limit, handler(ctx)).await {
        Ok(Ok(result)) => JsonRpcResponse::success(id, result),
        Ok(Err(err)) => JsonRpcResponse::error(id, err),
        Err(_) => JsonRpcResponse::error(
            id,
            JsonRpcError::internal_error("dispatch stalled").with_data(serde_json::json!({
                "stalled": true,
                "limit_ms": server.config.dispatch_stall_limit.as_millis() as u64,
            })),
        ),
    }
}
