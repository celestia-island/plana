//! Per-connection machinery: read loop, control-lane writer, outbound
//! notification handle.

use std::sync::Arc;

use futures::{SinkExt, StreamExt};
use tokio::sync::{mpsc, oneshot};

use axum::extract::ws::{CloseFrame, Message, WebSocket};
use plana::jsonrpc::{Id, JsonRpcError, JsonRpcNotification, JsonRpcResponse, JSONRPC_VERSION};

use crate::auth::AuthContext;
use crate::close_codes;
use crate::{RpcRequestCtx, RpcServer};

/// One serialized frame or a close request bound for the socket.
pub(crate) enum Wire {
    Text(String),
    Close(u16, &'static str),
}

/// The writer quit, so notifications can no longer be delivered.
#[derive(Debug)]
pub struct LaneClosed;

/// Handler-facing handle for pushing JSON-RPC notifications to this client
/// on the (bounded) data lane. Dropping or filling the lane surfaces as
/// [`LaneClosed`]; the connection tears down when the writer exits.
#[derive(Clone)]
pub struct OutboundHandle {
    data: mpsc::Sender<Wire>,
}

impl OutboundHandle {
    /// A handle whose notifications always fail. Used for the HTTP POST
    /// fallback transport, which has no server-initiated direction.
    pub fn closed() -> Self {
        let (tx, _rx) = mpsc::channel(1);
        drop(_rx);
        Self { data: tx }
    }

    /// Enqueue a notification frame for this client.
    pub async fn notify(&self, method: &str, params: serde_json::Value) -> Result<(), LaneClosed> {
        let frame = JsonRpcNotification {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: method.to_string(),
            params: Some(params),
        };
        let text = serde_json::to_string(&frame).map_err(|_| LaneClosed)?;
        self.data
            .send(Wire::Text(text))
            .await
            .map_err(|_| LaneClosed)
    }
}

/// Drive one accepted WebSocket connection to completion.
///
/// Structure mirrors the fleet reference implementations: the read loop
/// owns frame classification and dispatch, a dedicated writer task owns the
/// sink and drains the unbounded control lane with a biased `select!`
/// before the bounded data lane, and a close oneshot wakes the writer when
/// the read loop exits so idle connections cannot leak the receiver.
pub(crate) async fn run(
    socket: WebSocket,
    server: Arc<RpcServer>,
    conn_auth: AuthContext,
    _permit: tokio::sync::OwnedSemaphorePermit,
) {
    let (sink, mut stream) = socket.split();

    let (control_tx, control_rx) = mpsc::unbounded_channel::<Wire>();
    let (data_tx, data_rx) = mpsc::channel::<Wire>(server.config.data_lane_capacity);
    let (close_tx, close_rx) = oneshot::channel::<()>();

    tokio::spawn(run_writer(sink, control_rx, data_rx, close_rx));

    let outbound = OutboundHandle {
        data: data_tx.clone(),
    };

    while let Some(frame) = read_frame(&mut stream, server.config.idle_timeout).await {
        match frame {
            ReadFrame::Message(msg) => {
                match msg {
                    Message::Text(text) => {
                        handle_text(&text, &server, &conn_auth, &outbound, &data_tx, &control_tx)
                            .await;
                    }
                    Message::Binary(_) => {
                        // The profile is text-only; binary frames are a
                        // protocol violation, not a transient quirk.
                        let _ = control_tx.send(Wire::Close(
                            close_codes::UNSUPPORTED_DATA,
                            "binary frames are not part of this profile",
                        ));
                        break;
                    }
                    Message::Close(_) => break,
                    // Ping/Pong are answered by the WebSocket layer itself;
                    // receiving either still reset the idle window in
                    // `read_frame`.
                    Message::Ping(_) | Message::Pong(_) => {}
                }
            }
            ReadFrame::Idle => {
                let _ = control_tx.send(Wire::Close(
                    close_codes::IDLE_TIMEOUT,
                    "no inbound frame within the idle window",
                ));
                break;
            }
            ReadFrame::Gone => break,
        }
    }

    drop(data_tx);
    drop(outbound);
    let _ = close_tx.send(());
}

enum ReadFrame {
    Message(Message),
    Idle,
    Gone,
}

async fn read_frame<S>(stream: &mut S, idle: std::time::Duration) -> Option<ReadFrame>
where
    S: futures::Stream<Item = Result<Message, axum::Error>> + Unpin,
{
    match tokio::time::timeout(idle, stream.next()).await {
        // Elapsed without any inbound frame: heartbeat starvation.
        Err(_) => Some(ReadFrame::Idle),
        Ok(Some(Ok(msg))) => Some(ReadFrame::Message(msg)),
        Ok(Some(Err(_))) | Ok(None) => Some(ReadFrame::Gone),
    }
}

/// Classify and dispatch one text frame.
async fn handle_text(
    text: &str,
    server: &Arc<RpcServer>,
    conn_auth: &AuthContext,
    outbound: &OutboundHandle,
    data_tx: &mpsc::Sender<Wire>,
    control_tx: &mpsc::UnboundedSender<Wire>,
) {
    let value: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => {
            // Malformed JSON answers with id:null and keeps the connection,
            // matching the evernight reference behaviour.
            respond(
                data_tx,
                JsonRpcResponse::error(Id::Null, JsonRpcError::parse_error()),
            )
            .await;
            return;
        }
    };

    if value.is_array() {
        respond(
            data_tx,
            JsonRpcResponse::error(
                Id::Null,
                JsonRpcError::invalid_request()
                    .with_data(serde_json::json!({ "reason": "batch_not_supported" })),
            ),
        )
        .await;
        return;
    }

    match serde_json::from_value::<plana::jsonrpc::JsonRpcMessage>(value) {
        Ok(plana::jsonrpc::JsonRpcMessage::Request(req)) => {
            dispatch_request(req, server, conn_auth, outbound, data_tx).await;
        }
        Ok(plana::jsonrpc::JsonRpcMessage::Notification(notif)) => {
            if server.config.heartbeat && notif.method == close_codes::HEARTBEAT_METHOD {
                let ack = JsonRpcNotification {
                    jsonrpc: JSONRPC_VERSION.to_string(),
                    method: close_codes::HEARTBEAT_ACK_METHOD.to_string(),
                    params: None,
                };
                if let Ok(text) = serde_json::to_string(&ack) {
                    // Control lane: must overtake any backlog on the data lane.
                    let _ = control_tx.send(Wire::Text(text));
                }
            } else {
                tracing::debug!(method = %notif.method, "dropping client notification");
            }
        }
        Ok(plana::jsonrpc::JsonRpcMessage::Response(_)) => {
            tracing::warn!("client sent a JSON-RPC response frame; ignored");
        }
        Err(_) => {
            respond(
                data_tx,
                JsonRpcResponse::error(Id::Null, JsonRpcError::invalid_request()),
            )
            .await;
        }
    }
}

async fn dispatch_request(
    req: plana::jsonrpc::JsonRpcRequest,
    server: &Arc<RpcServer>,
    conn_auth: &AuthContext,
    outbound: &OutboundHandle,
    data_tx: &mpsc::Sender<Wire>,
) {
    let id = req.id.clone().unwrap_or(Id::Null);
    let params = req.params.unwrap_or(serde_json::Value::Null);

    if let Some(guard) = &server.guard {
        if let Err(err) = guard(conn_auth.clone(), &req.method, &params).await {
            respond(data_tx, JsonRpcResponse::error(id, err)).await;
            return;
        }
    }

    let Some(handler) = server.methods.get(&req.method).cloned() else {
        respond(
            data_tx,
            JsonRpcResponse::error(id, JsonRpcError::method_not_found(&req.method)),
        )
        .await;
        return;
    };

    let ctx = RpcRequestCtx {
        method: Arc::from(req.method.as_str()),
        params,
        auth: conn_auth.clone(),
        outbound: outbound.clone(),
    };

    match tokio::time::timeout(server.config.dispatch_stall_limit, handler(ctx)).await {
        Ok(Ok(result)) => respond(data_tx, JsonRpcResponse::success(id, result)).await,
        Ok(Err(err)) => respond(data_tx, JsonRpcResponse::error(id, err)).await,
        Err(_) => {
            // The handler future is dropped by the timeout; the cancelled
            // result never reaches the wire.
            tracing::warn!(method = %req.method, "dispatch stalled; frame dropped");
            respond(
                data_tx,
                JsonRpcResponse::error(
                    id,
                    JsonRpcError::internal_error("dispatch stalled").with_data(serde_json::json!({
                        "stalled": true,
                        "limit_ms": server.config.dispatch_stall_limit.as_millis() as u64,
                    })),
                ),
            )
            .await;
        }
    }
}

async fn respond(data_tx: &mpsc::Sender<Wire>, response: JsonRpcResponse) {
    if let Ok(text) = serde_json::to_string(&response) {
        // A send failure means the writer is gone; stop reading.
        if data_tx.send(Wire::Text(text)).await.is_err() {
            tracing::debug!("data lane closed while responding");
        }
    }
}

/// Drain the control lane (biased) before the data lane and write frames to
/// the socket. Exits when the read loop signals completion, either channel
/// closes, or the socket rejects a write.
async fn run_writer<S>(
    mut sink: S,
    mut control_rx: mpsc::UnboundedReceiver<Wire>,
    mut data_rx: mpsc::Receiver<Wire>,
    mut close_rx: oneshot::Receiver<()>,
) where
    S: futures::Sink<Message> + Unpin,
{
    loop {
        let wire = tokio::select! {
            biased;

            wire = control_rx.recv() => match wire {
                Some(w) => w,
                None => break,
            },
            wire = data_rx.recv() => match wire {
                Some(w) => w,
                // All response producers are gone; nothing left to write.
                None => break,
            },
            _ = &mut close_rx => break,
        };

        match wire {
            Wire::Text(text) => {
                if sink.send(Message::Text(text.into())).await.is_err() {
                    break;
                }
            }
            Wire::Close(code, reason) => {
                let _ = sink
                    .send(Message::Close(Some(CloseFrame {
                        code,
                        reason: reason.into(),
                    })))
                    .await;
                break;
            }
        }
    }
}
