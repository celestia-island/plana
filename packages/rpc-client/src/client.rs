//! Persistent WebSocket JSON-RPC client with id correlation, call
//! timeouts, heartbeat keepalive, and exponential-backoff reconnect.

use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use futures::{SinkExt, StreamExt};
use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tokio_tungstenite::tungstenite::Message;

use plana_jsonrpc::{
    Id, JsonRpcError, JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, JSONRPC_VERSION,
};

use crate::close_codes::HEARTBEAT_METHOD;
use crate::{ConnectionState, RpcError};

/// Tuning knobs; defaults mirror the TS client (15s heartbeat, 30s call
/// timeout, 1.5x reconnect backoff capped at 30s).
#[derive(Debug, Clone)]
pub struct RpcClientConfig {
    /// Bound on a single dial attempt.
    pub connect_timeout: Duration,
    /// Bound on every [`RpcClient::call`], independent of transport state.
    pub call_timeout: Duration,
    /// Cadence of `Base.Heartbeat` notifications.
    pub heartbeat_interval: Duration,
    /// No inbound frame (ack or otherwise) within this window ⇒ the
    /// connection is declared dead and cycles.
    pub heartbeat_timeout: Duration,
    /// Serve the built-in heartbeat service.
    pub heartbeat: bool,
    /// First reconnect delay; grows by `reconnect_factor` per failure.
    pub reconnect_initial: Duration,
    pub reconnect_factor: f64,
    pub reconnect_max: Duration,
    /// None ⇒ reconnect forever. Some(n) ⇒ after n consecutive failed
    /// attempts the client enters [`ConnectionState::Failed`] and calls
    /// fail fast until a new client is built.
    pub max_reconnect_attempts: Option<u32>,
}

impl Default for RpcClientConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(5),
            call_timeout: Duration::from_secs(30),
            heartbeat_interval: Duration::from_secs(15),
            heartbeat_timeout: Duration::from_secs(10),
            heartbeat: true,
            reconnect_initial: Duration::from_millis(500),
            reconnect_factor: 1.5,
            reconnect_max: Duration::from_secs(30),
            max_reconnect_attempts: None,
        }
    }
}

type PendingResult = Result<serde_json::Value, PendingFail>;

#[derive(Debug)]
enum PendingFail {
    Closed,
    Rpc(JsonRpcError),
}

enum Cmd {
    /// Admission-checked frame send: the supervisor answers before the
    /// frame is considered accepted (Ok) or refused (Err).
    SendFrame {
        frame: String,
        ack: oneshot::Sender<Result<(), RpcError>>,
    },
    ForceReconnect,
}

struct Inner {
    config: RpcClientConfig,
    cmd_tx: mpsc::Sender<Cmd>,
    state: watch::Sender<ConnectionState>,
    notifications: broadcast::Sender<serde_json::Value>,
    pending: StdMutex<HashMap<Id, oneshot::Sender<PendingResult>>>,
}

/// Builder for [`RpcClient`].
pub struct RpcClientBuilder {
    url: String,
    config: RpcClientConfig,
}

impl RpcClientBuilder {
    /// Endpoint URL, e.g. `ws://127.0.0.1:8092/api/ws` (plaintext `ws://`
    /// in this release).
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }

    pub fn config(mut self, config: RpcClientConfig) -> Self {
        self.config = config;
        self
    }

    /// Spawn the supervisor task and return the client handle. Requires a
    /// running tokio runtime. Dropping every handle shuts the connection
    /// down.
    pub fn build(self) -> RpcClient {
        assert!(!self.url.is_empty(), "rpc client url must be set");

        let (cmd_tx, cmd_rx) = mpsc::channel::<Cmd>(64);
        let (state_tx, _state_rx) = watch::channel(ConnectionState::Connecting);
        let (notify_tx, _notify_rx) = broadcast::channel::<serde_json::Value>(64);

        let inner = Arc::new(Inner {
            config: self.config,
            cmd_tx: cmd_tx.clone(),
            state: state_tx,
            notifications: notify_tx,
            pending: StdMutex::new(HashMap::new()),
        });

        let supervisor = supervise(self.url, inner.clone(), cmd_rx);
        tokio::spawn(supervisor);

        RpcClient { inner }
    }
}

/// A persistent-profile JSON-RPC client. Clone cheaply; the connection
/// lives until every clone (and thus the supervisor) is dropped.
#[derive(Clone)]
pub struct RpcClient {
    inner: Arc<Inner>,
}

impl std::fmt::Debug for RpcClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RpcClient")
            .field("state", &self.state())
            .finish_non_exhaustive()
    }
}

impl RpcClient {
    pub fn builder() -> RpcClientBuilder {
        RpcClientBuilder {
            url: String::new(),
            config: RpcClientConfig::default(),
        }
    }

    /// Current connection state snapshot.
    pub fn state(&self) -> ConnectionState {
        *self.inner.state.borrow()
    }

    /// Subscribe to connection-state transitions.
    pub fn subscribe_state(&self) -> watch::Receiver<ConnectionState> {
        self.inner.state.subscribe()
    }

    /// Subscribe to server-initiated notifications (the full JSON-RPC
    /// notification object as a [`serde_json::Value`]). Heartbeat acks are
    /// consumed internally and not forwarded.
    pub fn subscribe(&self) -> broadcast::Receiver<serde_json::Value> {
        self.inner.notifications.subscribe()
    }

    /// Issue a request/response call. Fails with [`RpcError::Timeout`] if
    /// no answer arrives within `call_timeout`, and with
    /// [`RpcError::Closed`] if no connection admits the frame.
    pub async fn call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, RpcError> {
        let id = Id::new_uuid();
        let request = JsonRpcRequest {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: Some(id.clone()),
            method: method.to_string(),
            params: Some(params),
        };
        let frame =
            serde_json::to_string(&request).map_err(|e| RpcError::Transport(e.to_string()))?;

        self.send_and_wait(frame, id).await
    }

    /// Fire-and-forget notification to the server.
    pub async fn notify(&self, method: &str, params: serde_json::Value) -> Result<(), RpcError> {
        let notification = JsonRpcNotification {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: method.to_string(),
            params: Some(params),
        };
        let frame =
            serde_json::to_string(&notification).map_err(|e| RpcError::Transport(e.to_string()))?;
        self.admit(frame).await
    }

    /// Drop the current connection and dial again immediately (next call
    /// observes the reconnect cycle).
    pub fn force_reconnect(&self) {
        let _ = self.inner.cmd_tx.try_send(Cmd::ForceReconnect);
    }

    async fn send_and_wait(&self, frame: String, id: Id) -> Result<serde_json::Value, RpcError> {
        let (tx, rx) = oneshot::channel::<PendingResult>();
        self.inner.pending.lock().unwrap().insert(id.clone(), tx);

        if let Err(err) = self.admit(frame).await {
            self.inner.pending.lock().unwrap().remove(&id);
            return Err(err);
        }

        let timeout = self.inner.config.call_timeout;
        match tokio::time::timeout(timeout, rx).await {
            Err(_) => {
                self.inner.pending.lock().unwrap().remove(&id);
                Err(RpcError::Timeout(timeout))
            }
            Ok(Ok(result)) => result.map_err(|fail| match fail {
                PendingFail::Closed => RpcError::Closed,
                PendingFail::Rpc(err) => err.into(),
            }),
            Ok(Err(_cancelled)) => {
                // Supervisor dropped the completer without answering
                // (teardown raced us); treat as a dead connection.
                self.inner.pending.lock().unwrap().remove(&id);
                Err(RpcError::Closed)
            }
        }
    }

    async fn admit(&self, frame: String) -> Result<(), RpcError> {
        let (ack_tx, ack_rx) = oneshot::channel();
        self.inner
            .cmd_tx
            .send(Cmd::SendFrame { frame, ack: ack_tx })
            .await
            .map_err(|_| RpcError::Closed)?;
        match ack_rx.await {
            Ok(result) => result,
            Err(_) => Err(RpcError::Closed),
        }
    }
}

enum Outcome {
    /// The connection was lost or declared dead; cycle with backoff.
    Lost,
    /// The client asked for an immediate redial.
    Forced,
    /// Every client handle was dropped; shut down.
    Shutdown,
}

async fn supervise(url: String, inner: Arc<Inner>, mut cmd_rx: mpsc::Receiver<Cmd>) {
    let config = inner.config.clone();
    let mut backoff = config.reconnect_initial;
    let mut attempts: u32 = 0;

    loop {
        set_state(&inner, ConnectionState::Connecting);

        let dialed = tokio::time::timeout(
            config.connect_timeout,
            tokio_tungstenite::connect_async(url.clone()),
        )
        .await;

        let socket = match dialed {
            Ok(Ok((socket, _))) => Some(socket),
            Ok(Err(err)) => {
                tracing::debug!(error = %err, "rpc client dial failed");
                None
            }
            Err(_) => {
                tracing::debug!("rpc client dial timed out");
                None
            }
        };

        let outcome = match socket {
            Some(socket) => {
                set_state(&inner, ConnectionState::Connected);
                backoff = config.reconnect_initial;
                attempts = 0;
                connected_phase(socket, &inner, &mut cmd_rx).await
            }
            None => Outcome::Lost,
        };

        match outcome {
            Outcome::Shutdown => {
                reject_all_pending(&inner);
                return;
            }
            Outcome::Lost | Outcome::Forced => {
                reject_all_pending(&inner);
                set_state(&inner, ConnectionState::Reconnecting);
            }
        }

        attempts += 1;
        if let Some(max) = config.max_reconnect_attempts {
            if attempts >= max {
                tracing::warn!(attempts, "rpc client reconnect budget exhausted");
                set_state(&inner, ConnectionState::Failed);
                // Serve (and refuse) further calls until the last handle
                // drops, so queued callers fail fast instead of timing out.
                while let Some(cmd) = cmd_rx.recv().await {
                    if let Cmd::SendFrame { ack, .. } = cmd {
                        let _ = ack.send(Err(RpcError::Closed));
                    }
                }
                return;
            }
        }

        // Backoff window: refuse new frames but stay responsive to
        // shutdown and to force-reconnect.
        let deadline = tokio::time::Instant::now() + backoff;
        loop {
            tokio::select! {
                cmd = cmd_rx.recv() => match cmd {
                    None => {
                        reject_all_pending(&inner);
                        return;
                    }
                    Some(Cmd::SendFrame { ack, .. }) => {
                        let _ = ack.send(Err(RpcError::Closed));
                    }
                    Some(Cmd::ForceReconnect) => break,
                },
                _ = tokio::time::sleep_until(deadline) => break,
            }
        }

        backoff = scale_backoff(backoff, config.reconnect_factor, config.reconnect_max);
    }
}

async fn connected_phase(
    socket: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    inner: &Arc<Inner>,
    cmd_rx: &mut mpsc::Receiver<Cmd>,
) -> Outcome {
    let config = &inner.config;
    let (sink, mut stream) = futures::StreamExt::split(socket);
    let (out_tx, out_rx) = mpsc::channel::<String>(64);
    let writer = tokio::spawn(write_pump(sink, out_rx));

    let mut heartbeat = tokio::time::interval(config.heartbeat_interval);
    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut watchdog = tokio::time::interval(config.heartbeat_timeout);
    watchdog.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut last_inbound = std::time::Instant::now();

    let outcome = loop {
        tokio::select! {
            cmd = cmd_rx.recv() => match cmd {
                None => break Outcome::Shutdown,
                Some(Cmd::SendFrame { frame, ack }) => {
                    let admitted = out_tx
                        .send(frame)
                        .await
                        .map_err(|_| RpcError::Closed);
                    let failed = admitted.is_err();
                    let _ = ack.send(admitted);
                    if failed {
                        break Outcome::Lost;
                    }
                }
                Some(Cmd::ForceReconnect) => break Outcome::Forced,
            },

            msg = stream.next() => match msg {
                Some(Ok(Message::Text(text))) => {
                    last_inbound = std::time::Instant::now();
                    handle_inbound(inner, &text);
                }
                Some(Ok(Message::Close(_))) => break Outcome::Lost,
                // Ping/Pong are answered by the WebSocket layer itself;
                // they still prove liveness.
                Some(Ok(_)) => last_inbound = std::time::Instant::now(),
                Some(Err(_)) | None => break Outcome::Lost,
            },

            _ = heartbeat.tick(), if config.heartbeat => {
                let frame = JsonRpcNotification {
                    jsonrpc: JSONRPC_VERSION.to_string(),
                    method: HEARTBEAT_METHOD.to_string(),
                    params: None,
                };
                if let Ok(text) = serde_json::to_string(&frame) {
                    if out_tx.send(text).await.is_err() {
                        break Outcome::Lost;
                    }
                }
            },

            _ = watchdog.tick() => {
                if last_inbound.elapsed() >= config.heartbeat_timeout {
                    tracing::debug!("rpc client heartbeat watchdog fired");
                    break Outcome::Lost;
                }
            }
        }
    };

    drop(out_tx);
    // The writer may be blocked on a dead socket; do not wait for it.
    writer.abort();
    outcome
}

async fn write_pump<S>(mut sink: S, mut out_rx: mpsc::Receiver<String>)
where
    S: SinkExt<Message> + Unpin,
    S::Error: std::fmt::Debug,
{
    while let Some(text) = out_rx.recv().await {
        if sink.send(Message::Text(text.into())).await.is_err() {
            break;
        }
    }
}

fn handle_inbound(inner: &Arc<Inner>, text: &str) {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return;
    };
    let Ok(message) = serde_json::from_value::<JsonRpcMessage>(value) else {
        tracing::warn!("rpc client received an unclassifiable frame");
        return;
    };
    match message {
        JsonRpcMessage::Response(response) => {
            let completer = inner.pending.lock().unwrap().remove(&response.id);
            if let Some(tx) = completer {
                let result = match response.error {
                    Some(err) => Err(PendingFail::Rpc(err)),
                    None => Ok(response.result.unwrap_or(serde_json::Value::Null)),
                };
                let _ = tx.send(result);
            } else {
                tracing::debug!(id = ?response.id, "unsolicited response ignored");
            }
        }
        JsonRpcMessage::Notification(notification) => {
            // Heartbeat acks are liveness, not content: consumed by the
            // watchdog, never forwarded to subscribers.
            if notification.method == crate::close_codes::HEARTBEAT_ACK_METHOD {
                return;
            }
            let _ = inner
                .notifications
                .send(serde_json::to_value(&notification).unwrap_or_default());
        }
        JsonRpcMessage::Request(_) => {
            tracing::warn!("server sent a request frame; not supported by this client");
        }
    }
}

fn reject_all_pending(inner: &Arc<Inner>) {
    let mut pending = inner.pending.lock().unwrap();
    for (_, tx) in pending.drain() {
        let _ = tx.send(Err(PendingFail::Closed));
    }
}

fn set_state(inner: &Arc<Inner>, state: ConnectionState) {
    inner.state.send_replace(state);
}

fn scale_backoff(current: Duration, factor: f64, max: Duration) -> Duration {
    let next = current.mul_f64(factor);
    if next > max {
        max
    } else {
        next
    }
}
