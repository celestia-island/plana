use anyhow::{Result, anyhow};
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use std::{
    sync::{
        Arc,
        atomic::{AtomicU8, AtomicU32, Ordering},
    },
    time::Duration,
};
use tokio::{
    net::TcpStream,
    sync::{Mutex, watch},
};

pub use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{
        client::IntoClientRequest,
        protocol::{CloseFrame, Message as WsMessage, frame::coding::CloseCode},
    },
};
use tracing::{debug, error, info, instrument, warn};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_RECONNECT_MAX_RETRIES: u32 = 5;
const BACKOFF_BASE: Duration = Duration::from_millis(500);
const BACKOFF_MAX: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ConnectionState {
    Disconnected = 0,
    Connecting = 1,
    Connected = 2,
    Reconnecting = 3,
}

#[derive(Debug, Clone)]
pub struct WsTransportConfig {
    pub url: String,
    pub heartbeat_interval: Option<Duration>,
    pub reconnect_max_retries: Option<u32>,
    pub connect_timeout: Option<Duration>,
    pub host_header: Option<String>,
    pub auth_header: Option<String>,
}

impl WsTransportConfig {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            heartbeat_interval: None,
            reconnect_max_retries: None,
            connect_timeout: None,
            host_header: None,
            auth_header: None,
        }
    }

    pub fn with_heartbeat(mut self, interval: Duration) -> Self {
        self.heartbeat_interval = Some(interval);
        self
    }

    pub fn with_reconnect(mut self, max_retries: u32) -> Self {
        self.reconnect_max_retries = Some(max_retries);
        self
    }

    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    pub fn with_host_header(mut self, host: impl Into<String>) -> Self {
        self.host_header = Some(host.into());
        self
    }

    pub fn with_auth_header(mut self, value: impl Into<String>) -> Self {
        self.auth_header = Some(value.into());
        self
    }
}

struct SharedState {
    sender: Mutex<Option<futures::stream::SplitSink<WsStream, WsMessage>>>,
    connection_state: AtomicU8,
    reconnect_attempts: AtomicU32,
    shutdown: watch::Sender<bool>,
}

pub struct WsTransport {
    shared: Arc<SharedState>,
}

pub struct WsSender {
    shared: Arc<SharedState>,
}

pub struct WsReceiver {
    rx: tokio::sync::mpsc::Receiver<Result<WsMessage>>,
}

impl WsTransport {
    #[instrument(skip(config), fields(url = %config.url))]
    pub async fn connect(config: WsTransportConfig) -> Result<(WsSender, WsReceiver)> {
        let (sender, receiver) = Self::establish_connection(&config).await?;

        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let (msg_tx, msg_rx) = tokio::sync::mpsc::channel(256);

        let shared = Arc::new(SharedState {
            sender: Mutex::new(Some(sender)),
            connection_state: AtomicU8::new(ConnectionState::Connected as u8),
            reconnect_attempts: AtomicU32::new(0),
            shutdown: shutdown_tx,
        });

        let shared_clone = shared.clone();
        let config_clone = config.clone();
        let msg_tx_clone = msg_tx.clone();

        tokio::spawn(async move {
            Self::receive_loop(
                receiver,
                msg_tx_clone,
                shutdown_rx,
                shared_clone,
                config_clone,
            )
            .await;
        });

        if let Some(interval) = config.heartbeat_interval {
            let hb_shared = shared.clone();
            let hb_shutdown = shared.shutdown.subscribe();
            let hb_interval = interval;
            tokio::spawn(async move {
                Self::heartbeat_loop(hb_shared, hb_shutdown, hb_interval).await;
            });
        }

        let ws_sender = WsSender {
            shared: shared.clone(),
        };
        let ws_receiver = WsReceiver { rx: msg_rx };

        Ok((ws_sender, ws_receiver))
    }

    async fn establish_connection(
        config: &WsTransportConfig,
    ) -> Result<(
        futures::stream::SplitSink<WsStream, WsMessage>,
        futures::stream::SplitStream<WsStream>,
    )> {
        let timeout = config.connect_timeout.unwrap_or(DEFAULT_CONNECT_TIMEOUT);

        let connect_future = async {
            let need_custom_request = config.host_header.is_some() || config.auth_header.is_some();
            if need_custom_request {
                let mut request = config.url.as_str().into_client_request().map_err(
                    |e: tokio_tungstenite::tungstenite::Error| anyhow!("connection failed: {}", e),
                )?;
                if let Some(ref host) = config.host_header {
                    request.headers_mut().insert(
                        "Host",
                        host.parse().map_err(|e| {
                            anyhow!("connection failed: invalid host header value: {}", e)
                        })?,
                    );
                }
                if let Some(ref auth) = config.auth_header {
                    request.headers_mut().insert(
                        "Authorization",
                        auth.parse().map_err(|e| {
                            anyhow!("connection failed: invalid auth header value: {}", e)
                        })?,
                    );
                }
                connect_async(request)
                    .await
                    .map_err(|e| anyhow!("connection failed: {}", e))
            } else {
                connect_async(&config.url)
                    .await
                    .map_err(|e| anyhow!("connection failed: {}", e))
            }
        };

        let (ws, _) = tokio::time::timeout(timeout, connect_future)
            .await
            .map_err(|_| anyhow!("connection failed: timeout after {:?}", timeout))?
            .map_err(|e| anyhow!("connection failed: {}", e))?;

        Ok(ws.split())
    }

    fn set_state(shared: &SharedState, state: ConnectionState) {
        shared.connection_state.store(state as u8, Ordering::SeqCst);
    }

    pub fn connection_state(&self) -> ConnectionState {
        match self.shared.connection_state.load(Ordering::SeqCst) {
            0 => ConnectionState::Disconnected,
            1 => ConnectionState::Connecting,
            2 => ConnectionState::Connected,
            3 => ConnectionState::Reconnecting,
            _ => ConnectionState::Disconnected,
        }
    }

    pub async fn shutdown(&self) {
        let _ = self.shared.shutdown.send(true);
        let mut guard = self.shared.sender.lock().await;
        if let Some(sender) = guard.take() {
            let frame = CloseFrame {
                code: CloseCode::Normal,
                reason: "shutdown".into(),
            };
            let mut s = sender;
            let _ = s.send(WsMessage::Close(Some(frame))).await;
        }
    }

    fn calc_backoff(attempt: u32) -> Duration {
        let exp = BACKOFF_BASE
            .checked_mul(1 << attempt.min(6))
            .unwrap_or(BACKOFF_MAX);
        exp.min(BACKOFF_MAX)
    }

    async fn try_reconnect(
        shared: &Arc<SharedState>,
        config: &WsTransportConfig,
    ) -> Result<(
        futures::stream::SplitSink<WsStream, WsMessage>,
        futures::stream::SplitStream<WsStream>,
    )> {
        let max_retries = config
            .reconnect_max_retries
            .unwrap_or(DEFAULT_RECONNECT_MAX_RETRIES);

        Self::set_state(shared, ConnectionState::Reconnecting);

        for attempt in 0..max_retries {
            shared
                .reconnect_attempts
                .store(attempt + 1, Ordering::SeqCst);
            let backoff = Self::calc_backoff(attempt);
            info!(
                attempt = attempt + 1,
                max = max_retries,
                backoff_ms = backoff.as_millis(),
                "reconnect attempt"
            );
            tokio::time::sleep(backoff).await;

            match Self::establish_connection(config).await {
                Ok(result) => {
                    shared.reconnect_attempts.store(0, Ordering::SeqCst);
                    Self::set_state(shared, ConnectionState::Connected);
                    info!(attempt = attempt + 1, "reconnected");
                    return Ok(result);
                },
                Err(e) => {
                    warn!(attempt = attempt + 1, error = %e, "reconnect failed");
                },
            }
        }

        Self::set_state(shared, ConnectionState::Disconnected);
        Err(anyhow!("reconnect exhausted after {} retries", max_retries))
    }

    async fn receive_loop(
        mut receiver: futures::stream::SplitStream<WsStream>,
        msg_tx: tokio::sync::mpsc::Sender<Result<WsMessage>>,
        mut shutdown_rx: watch::Receiver<bool>,
        shared: Arc<SharedState>,
        config: WsTransportConfig,
    ) {
        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        info!("receive loop shutting down");
                        break;
                    }
                }
                msg = receiver.next() => {
                    match msg {
                        Some(Ok(WsMessage::Text(text))) => {
                            debug!(size = text.len(), "text message received");
                            if msg_tx.send(Ok(WsMessage::Text(text))).await.is_err() {
                                break;
                            }
                        }
                        Some(Ok(WsMessage::Binary(data))) => {
                            debug!(size = data.len(), "binary message received");
                            if msg_tx.send(Ok(WsMessage::Binary(data))).await.is_err() {
                                break;
                            }
                        }
                        Some(Ok(WsMessage::Ping(data))) => {
                            debug!("ping received");
                            let mut guard = shared.sender.lock().await;
                            if let Some(sender) = guard.as_mut() {
                                let _ = sender.send(WsMessage::Pong(data)).await;
                            }
                        }
                        Some(Ok(WsMessage::Close(reason))) => {
                            info!(reason = ?reason, "connection closed by remote");
                            let _ = msg_tx.send(Err(anyhow!("connection closed"))).await;
                            break;
                        }
                        Some(Err(e)) => {
                            error!(error = %e, "websocket error in receive loop");
                            let _ = msg_tx.send(Err(anyhow!("receive failed: {}", e))).await;
                            break;
                        }
                        Some(_) => {}
                        None => {
                            info!("websocket stream ended");
                            break;
                        }
                    }
                }
            }
        }

        if config.reconnect_max_retries.is_some_and(|r| r > 0) {
            match Self::try_reconnect(&shared, &config).await {
                Ok((new_sender, new_receiver)) => {
                    {
                        let mut guard = shared.sender.lock().await;
                        *guard = Some(new_sender);
                    }
                    Box::pin(Self::receive_loop(
                        new_receiver,
                        msg_tx,
                        shutdown_rx,
                        shared,
                        config,
                    ))
                    .await;
                },
                Err(e) => {
                    error!(error = %e, "reconnection failed permanently");
                    let _ = msg_tx.send(Err(e)).await;
                },
            }
        } else {
            Self::set_state(&shared, ConnectionState::Disconnected);
        }
    }

    async fn heartbeat_loop(
        shared: Arc<SharedState>,
        mut shutdown_rx: watch::Receiver<bool>,
        interval: Duration,
    ) {
        let mut tick = tokio::time::interval(interval);
        tick.tick().await;

        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        break;
                    }
                }
                _ = tick.tick() => {
                    let state = shared.connection_state.load(Ordering::SeqCst);
                    if state != ConnectionState::Connected as u8 {
                        continue;
                    }
                    let mut guard = shared.sender.lock().await;
                    if let Some(sender) = guard.as_mut() {
                        let payload = format!("{{\"heartbeat\":{}}}", Utc::now().timestamp());
                        if let Err(e) = sender.send(WsMessage::Ping(payload.into())).await {
                            warn!(error = %e, "heartbeat ping failed");
                        }
                    }
                }
            }
        }
    }
}

impl WsSender {
    pub async fn send_text(&self, text: String) -> Result<()> {
        let mut guard = self.shared.sender.lock().await;
        let sender = guard
            .as_mut()
            .ok_or_else(|| anyhow!("transport not connected"))?;
        sender
            .send(WsMessage::Text(text.into()))
            .await
            .map_err(|e| anyhow!("send failed: {}", e))
    }

    pub async fn send_binary(&self, data: Vec<u8>) -> Result<()> {
        let mut guard = self.shared.sender.lock().await;
        let sender = guard
            .as_mut()
            .ok_or_else(|| anyhow!("transport not connected"))?;
        sender
            .send(WsMessage::Binary(data.into()))
            .await
            .map_err(|e| anyhow!("send failed: {}", e))
    }

    pub fn connection_state(&self) -> ConnectionState {
        match self.shared.connection_state.load(Ordering::SeqCst) {
            0 => ConnectionState::Disconnected,
            1 => ConnectionState::Connecting,
            2 => ConnectionState::Connected,
            3 => ConnectionState::Reconnecting,
            _ => ConnectionState::Disconnected,
        }
    }
}

impl WsReceiver {
    pub async fn next_message(&mut self) -> Option<Result<WsMessage>> {
        self.rx.recv().await
    }
}
