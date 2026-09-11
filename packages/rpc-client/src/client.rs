//! Persistent WebSocket JSON-RPC client with id correlation, call
//! timeouts, heartbeat keepalive, and exponential-backoff reconnect.

use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use futures::{SinkExt, StreamExt};
use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tokio_tungstenite::tungstenite::Message;

use plana::jsonrpc::deferred::{
    DeferredOpOutcome, DeferredOpRef, DeferredOpSettledParams, OpsResultParams, OPS_RESULT_METHOD,
    OPS_SETTLED_METHOD,
};
use plana::jsonrpc::{
    error_codes, Id, JsonRpcError, JsonRpcMessage, JsonRpcNotification, JsonRpcRequest,
    JSONRPC_VERSION,
};

use crate::close_codes::HEARTBEAT_METHOD;
use crate::{ConnectionState, DeferredOpError, RpcError};

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
    /// First delay between `ops.result` polls in [`RpcClient::await_op`]
    /// (default 500ms); grows by `op_poll_factor` up to `op_poll_max`.
    ///
    /// The cadence only bounds the **fallback** path: the helper wakes
    /// immediately on the advisory `ops.settled` notification, so polling is
    /// what happens when a notification was missed (connection cycled,
    /// op begun on another connection, HTTP transport).
    pub op_poll_initial: Duration,
    pub op_poll_factor: f64,
    pub op_poll_max: Duration,
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
            op_poll_initial: Duration::from_millis(500),
            op_poll_factor: 1.5,
            op_poll_max: Duration::from_secs(5),
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
    /// Endpoint URL, e.g. `ws://127.0.0.1:8092/api/ws` (or `wss://` under
    /// the `tls` feature). Credentials may ride the fleet-canonical
    /// `?token=` query parameter.
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }

    /// Install the ring CryptoProvider as the process-wide rustls default
    /// (`tls` feature). Idempotent; call once before the first `wss://`
    /// connection — rustls refuses to build a ClientConfig without a
    /// provider.
    #[cfg(feature = "tls")]
    pub fn install_ring_tls_provider() {
        let _ = rustls::crypto::ring::default_provider().install_default();
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

    /// Await the outcome of a deferred operation (`op_id` from a
    /// deferred-capable handler's immediate answer).
    ///
    /// Collection is notification-first with a polling fallback: the helper
    /// subscribes to `ops.settled` and wakes as soon as the server announces
    /// this id, and otherwise polls `ops.result` on the configured cadence
    /// (never a busy loop). Every poll is the **authoritative** read — the
    /// notification only says an operation settled, never what it produced.
    ///
    /// - `deadline` bounds the whole wait; exceeding it answers
    ///   [`DeferredOpError::Deadline`]. A poll that would overrun the
    ///   deadline is cut short (its late response, if any, is discarded by
    ///   the client), so the caller is never kept waiting past it.
    /// - Transport failures (a reconnect cycling the connection, a dropped
    ///   call) are retried until the deadline: a client that reconnects
    ///   inside the op's validity window still collects the outcome, which is
    ///   the whole point of a deferred operation.
    /// - `-32052`/`-32053` (unknown / expired id) are terminal and answer
    ///   [`DeferredOpError::Unknown`] / [`DeferredOpError::Expired`].
    /// - A pending operation is never returned: the call resolves only with
    ///   an outcome that has settled (or fails).
    pub async fn await_op(
        &self,
        op_id: &DeferredOpRef,
        deadline: Duration,
    ) -> Result<DeferredOpOutcome, DeferredOpError> {
        let end = tokio::time::Instant::now() + deadline;
        let mut notifications = self.subscribe();
        let mut delay = self.inner.config.op_poll_initial;

        loop {
            let remaining = end.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return Err(DeferredOpError::Deadline(deadline));
            }

            match tokio::time::timeout(remaining, self.collect_op(op_id)).await {
                // The deadline elapsed while the call was in flight.
                Err(_) => return Err(DeferredOpError::Deadline(deadline)),
                Ok(Ok(outcome)) if outcome.is_settled() => return Ok(outcome),
                // Still pending: wait out the poll cadence (or wake on the
                // advisory notification for this id).
                Ok(Ok(_pending)) => {}
                Ok(Err(err)) => match err {
                    DeferredOpError::Unknown | DeferredOpError::Expired => return Err(err),
                    // Transport trouble: keep trying until the deadline, the
                    // op itself is still alive on the server.
                    DeferredOpError::Rpc(RpcError::Closed)
                    | DeferredOpError::Rpc(RpcError::Timeout(_))
                    | DeferredOpError::Rpc(RpcError::Transport(_)) => {}
                    // The server answered a real error (guard denial, method
                    // gone, …): retrying cannot help.
                    other => return Err(other),
                },
            }

            let remaining = end.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return Err(DeferredOpError::Deadline(deadline));
            }
            let wait = delay.min(remaining);
            let _ = tokio::time::timeout(wait, async {
                loop {
                    match notifications.recv().await {
                        Ok(value) if announces_settlement(&value, op_id) => break,
                        Ok(_) => continue,
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
            })
            .await;

            delay = scale_backoff(
                delay,
                self.inner.config.op_poll_factor,
                self.inner.config.op_poll_max,
            );
        }
    }

    /// One authoritative `ops.result` read.
    async fn collect_op(
        &self,
        op_id: &DeferredOpRef,
    ) -> Result<DeferredOpOutcome, DeferredOpError> {
        let params = serde_json::to_value(OpsResultParams {
            op_id: op_id.clone(),
        })
        .map_err(|e| DeferredOpError::Rpc(RpcError::Transport(e.to_string())))?;

        match self.call(OPS_RESULT_METHOD, params).await {
            Ok(value) => serde_json::from_value(value).map_err(|e| {
                DeferredOpError::Rpc(RpcError::Transport(format!(
                    "ops.result answered an unparseable outcome: {e}"
                )))
            }),
            Err(RpcError::Rpc { code, .. }) if code == error_codes::OPS_UNKNOWN => {
                Err(DeferredOpError::Unknown)
            }
            Err(RpcError::Rpc { code, .. }) if code == error_codes::OPS_EXPIRED => {
                Err(DeferredOpError::Expired)
            }
            Err(err) => Err(DeferredOpError::Rpc(err)),
        }
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

/// Whether a server-initiated notification is the advisory `ops.settled`
/// announcement for `op_id`.
fn announces_settlement(value: &serde_json::Value, op_id: &DeferredOpRef) -> bool {
    let Ok(notification) = serde_json::from_value::<JsonRpcNotification>(value.clone()) else {
        return false;
    };
    if notification.method != OPS_SETTLED_METHOD {
        return false;
    }
    notification
        .params
        .and_then(|params| serde_json::from_value::<DeferredOpSettledParams>(params).ok())
        .is_some_and(|params| params.op_id == *op_id)
}
