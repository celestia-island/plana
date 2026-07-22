use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use axum::{
    Router,
    extract::ws::Message,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json,
};
use serde_json::Value;

use crate::types::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, Id};

pub type RpcHandlerFn = Arc<
    dyn Fn(Value) -> Pin<Box<dyn Future<Output = Result<Value, JsonRpcError>> + Send>>
        + Send
        + Sync,
>;

#[derive(Clone)]
pub struct RpcMethodMap {
    methods: Arc<HashMap<String, RpcHandlerFn>>,
}

impl Default for RpcMethodMap {
    fn default() -> Self {
        Self {
            methods: Arc::new(HashMap::new()),
        }
    }
}

impl RpcMethodMap {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn method<F, Fut>(self, name: &str, handler: F) -> Self
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Value, JsonRpcError>> + Send + 'static,
    {
        let mut map = (*self.methods).clone();
        map.insert(name.to_string(), Arc::new(move |v| Box::pin(handler(v))));
        Self {
            methods: Arc::new(map),
        }
    }

    pub async fn dispatch(&self, params: Option<Value>, method: &str) -> JsonRpcResponse {
        let id = Id::String(method.to_string());
        if let Some(handler) = self.methods.get(method) {
            match handler(params.unwrap_or(Value::Null)).await {
                Ok(result) => JsonRpcResponse::success(id, result),
                Err(err) => JsonRpcResponse::error(id, err),
            }
        } else {
            JsonRpcResponse::error(id, JsonRpcError::method_not_found(method))
        }
    }
}

pub fn rpc_axum_router(methods: RpcMethodMap) -> Router {
    let map = Arc::new(methods);

    let map_post = map.clone();
    let map_ws = map;

    Router::new()
        .route(
            "/",
            post(move |Json(body): Json<Value>| {
                let map = map_post.clone();
                async move {
                    match handle_request(&map, body).await {
                        Ok(response) => Json(serde_json::to_value(response).unwrap()).into_response(),
                        Err((status, json)) => (status, json).into_response(),
                    }
                }
            })
            .get(move |ws: axum::extract::WebSocketUpgrade| {
                let map = map_ws.clone();
                async move { ws.on_upgrade(move |socket| handle_ws(socket, map)) }
            }),
        )
}

async fn handle_request(
    map: &RpcMethodMap,
    body: Value,
) -> Result<JsonRpcResponse, (StatusCode, Json<Value>)> {
    let request: JsonRpcRequest = serde_json::from_value(body).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(
                serde_json::to_value(
                    JsonRpcError::parse_error()
                        .with_data(Value::String(e.to_string())),
                )
                .unwrap(),
            ),
        )
    })?;

    let id = request.id.clone().unwrap_or(Id::Null);
    let method = &request.method;
    let params = request.params;

    if let Some(handler) = map.methods.get(method) {
        match handler(params.unwrap_or(Value::Null)).await {
            Ok(result) => Ok(JsonRpcResponse::success(id, result)),
            Err(err) => Ok(JsonRpcResponse::error(id, err)),
        }
    } else {
        Ok(JsonRpcResponse::error(
            id,
            JsonRpcError::method_not_found(method),
        ))
    }
}

async fn handle_ws(
    mut socket: axum::extract::ws::WebSocket,
    methods: Arc<RpcMethodMap>,
) {
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                let body: Value = match serde_json::from_str(&text) {
                    Ok(v) => v,
                    Err(_) => {
                        let err = JsonRpcResponse::error(
                            Id::String("parse".into()),
                            JsonRpcError::parse_error(),
                        );
                        let _ = socket
                            .send(Message::Text(
                                serde_json::to_string(&err).unwrap_or_default().into(),
                            ))
                            .await;
                        continue;
                    }
                };

                let response = match handle_request(&methods, body).await {
                    Ok(resp) => serde_json::to_string(&resp).unwrap_or_default(),
                    Err((_, err_json)) => {
                        serde_json::to_string(&err_json.0).unwrap_or_default()
                    }
                };
                let _ = socket.send(Message::Text(response.into())).await;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}
