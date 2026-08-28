//! Transport abstraction and the Unix-socket IPC implementation.
//!
//! `IpcSocketTransport` mirrors the semantics of entelecheia's current
//! PoleMos bridge (spec fact F2): one connect → write → read → close
//! exchange per call over a newline-delimited JSON-RPC 2.0 Unix stream —
//! write a single line, drop the write half, read a single line, parse the
//! response, close. There is no session and no notification stream on IPC.
//!
//! One contract-mandated refinement: the read deadline must stay *above*
//! the server-side execution timeout (spec §4.1 read-back rule 4: "适配层 >
//! 服务端"), so the effective io timeout is derived from the request's
//! `timeout` param (see [`IpcSocketTransport::effective_io_timeout_secs`])
//! instead of a fixed 30 seconds.

use std::time::Duration;

use plana_jsonrpc::{JsonRpcRequest, JsonRpcResponse};
use serde_json::Value;

/// Evernight's documented default execution timeout for `Command.Exec`
/// (spec F6) — assumed when a request carries no explicit `timeout` param.
const EVERNIGHT_DEFAULT_EXEC_TIMEOUT_SECS: u64 = 60;

/// Margin added on top of the server-side timeout when deriving the client
/// read deadline, keeping the "adapter > server" ordering (spec §4.1).
const IO_TIMEOUT_MARGIN_SECS: u64 = 30;

/// Transport-layer failure reasons.
///
/// Every variant classifies as [`ErrorKind::PeerUnreachable`]
/// (`-32603` + `data.reason = "peer_unreachable"`) via
/// [`crate::classify_transport`]; the payload strings carry the underlying
/// source message for logs.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TransportError {
    /// Unix socket connect failed (refused, missing path, connect timeout).
    #[error("evernight IPC connect failed: {0}")]
    Connect(String),
    /// Writing the request line failed.
    #[error("evernight IPC write failed: {0}")]
    Write(String),
    /// Reading the response exceeded the derived io deadline.
    #[error("evernight IPC read timed out after {0}s")]
    ReadTimeout(u64),
    /// Reading the response failed with an io error.
    #[error("evernight IPC read failed: {0}")]
    Read(String),
    /// The response line was not a valid JSON-RPC response object.
    #[error("evernight IPC parse failed: {0}")]
    Parse(String),
    /// The peer closed the connection before sending a response line.
    #[error("evernight IPC: connection closed before a response arrived")]
    Closed,
}

/// One-shot request/response transport towards the evernight broker.
#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    /// Sends one JSON-RPC request and awaits its correlated response.
    async fn call(&self, req: JsonRpcRequest) -> Result<JsonRpcResponse, TransportError>;
}

/// Unix domain socket transport (newline-delimited JSON-RPC 2.0, one
/// short-lived connection per call — the contract's baseline form).
#[derive(Debug, Clone)]
pub struct IpcSocketTransport {
    socket_path: String,
    connect_timeout_secs: u64,
    io_timeout_secs: u64,
}

impl IpcSocketTransport {
    /// Default connect timeout (mirrors the existing PoleMos bridge, F2).
    pub const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 30;
    /// Default io timeout floor (mirrors the existing PoleMos bridge, F2).
    /// Requests with an explicit `timeout` param always derive a larger
    /// deadline; see [`Self::effective_io_timeout_secs`].
    pub const DEFAULT_IO_TIMEOUT_SECS: u64 = 30;

    /// Creates a transport for the broker socket path with default timeouts.
    pub fn new(socket_path: impl Into<String>) -> Self {
        Self::with_timeouts(
            socket_path,
            Self::DEFAULT_CONNECT_TIMEOUT_SECS,
            Self::DEFAULT_IO_TIMEOUT_SECS,
        )
    }

    /// Creates a transport with explicit connect/io timeout floors (seconds).
    pub fn with_timeouts(
        socket_path: impl Into<String>,
        connect_timeout_secs: u64,
        io_timeout_secs: u64,
    ) -> Self {
        Self {
            socket_path: socket_path.into(),
            connect_timeout_secs,
            io_timeout_secs,
        }
    }

    /// The broker socket path.
    pub fn socket_path(&self) -> &str {
        &self.socket_path
    }

    /// Configured connect timeout floor (seconds).
    pub fn connect_timeout_secs(&self) -> u64 {
        self.connect_timeout_secs
    }

    /// Configured io timeout floor (seconds).
    pub fn io_timeout_secs(&self) -> u64 {
        self.io_timeout_secs
    }

    /// Derives the read deadline for one exchange (seconds).
    ///
    /// `wire_timeout` is the request's `timeout` param, if present. The
    /// deadline is `max(configured floor, server timeout + 30s)` so the
    /// client never gives up before the server-side execution timeout has
    /// had a chance to fire (spec §4.1: adapter io timeout must exceed the
    /// server timeout). Requests without an explicit timeout are measured
    /// against evernight's documented default of 60s.
    pub fn effective_io_timeout_secs(&self, wire_timeout: Option<u64>) -> u64 {
        let server_timeout = wire_timeout.unwrap_or(EVERNIGHT_DEFAULT_EXEC_TIMEOUT_SECS);
        self.io_timeout_secs
            .max(server_timeout.saturating_add(IO_TIMEOUT_MARGIN_SECS))
    }

    /// Extracts the `timeout` param from a `Command.Exec` request.
    fn wire_timeout(req: &JsonRpcRequest) -> Option<u64> {
        req.params
            .as_ref()
            .and_then(|params| params.get("timeout"))
            .and_then(Value::as_u64)
    }
}

#[cfg(unix)]
#[async_trait::async_trait]
impl Transport for IpcSocketTransport {
    async fn call(&self, req: JsonRpcRequest) -> Result<JsonRpcResponse, TransportError> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        // Frame the request as a single newline-terminated JSON line.
        let mut framed = serde_json::to_string(&req)
            .map_err(|e| TransportError::Write(format!("serialize request: {e}")))?;
        framed.push('\n');

        // Connect (bounded).
        let stream = match tokio::time::timeout(
            Duration::from_secs(self.connect_timeout_secs),
            tokio::net::UnixStream::connect(&self.socket_path),
        )
        .await
        {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => {
                return Err(TransportError::Connect(format!(
                    "{} (path {})",
                    e, self.socket_path
                )));
            }
            Err(_) => {
                return Err(TransportError::Connect(format!(
                    "connect timeout after {}s (path {})",
                    self.connect_timeout_secs, self.socket_path
                )));
            }
        };

        // Write the single request line, then drop the write half — the
        // broker answers one line per connection (F2 semantics).
        let (read_half, mut write_half) = tokio::io::split(stream);
        write_half
            .write_all(framed.as_bytes())
            .await
            .map_err(|e| TransportError::Write(e.to_string()))?;
        drop(write_half);

        // Read exactly one response line, bounded by the derived deadline.
        let io_timeout = self.effective_io_timeout_secs(Self::wire_timeout(&req));
        let mut reader = BufReader::new(read_half);
        let mut line = String::with_capacity(64 * 1024);
        match tokio::time::timeout(Duration::from_secs(io_timeout), reader.read_line(&mut line))
            .await
        {
            Ok(Ok(0)) => return Err(TransportError::Closed),
            Ok(Ok(_)) => {}
            Ok(Err(e)) => return Err(TransportError::Read(e.to_string())),
            Err(_) => return Err(TransportError::ReadTimeout(io_timeout)),
        }

        serde_json::from_str::<JsonRpcResponse>(line.trim())
            .map_err(|e| TransportError::Parse(e.to_string()))
    }
}

#[cfg(not(unix))]
#[async_trait::async_trait]
impl Transport for IpcSocketTransport {
    async fn call(&self, req: JsonRpcRequest) -> Result<JsonRpcResponse, TransportError> {
        let _ = &self.socket_path;
        Err(TransportError::Connect(format!(
            "evernight IPC (method={}) only available on Unix",
            req.method
        )))
    }
}

/// Deferred remote-form transport skeleton (spec §5.1: the wss form is
/// explicitly postponed).
///
/// Exists so the adapter surface already names the future long-lived
/// transport; every call fails fast with [`TransportError::Connect`] until
/// the real tokio-tungstenite implementation lands behind the `ws` feature
/// (requires the broker-side dispatch-layer authentication gap, spec F8, to
/// be closed first). Do not enable for production traffic.
#[cfg(feature = "ws")]
#[derive(Debug, Clone)]
pub struct WsTransport {
    /// Remote endpoint (documentation examples use RFC 5737 addresses only).
    pub endpoint: String,
}

#[cfg(feature = "ws")]
#[async_trait::async_trait]
impl Transport for WsTransport {
    async fn call(&self, _req: JsonRpcRequest) -> Result<JsonRpcResponse, TransportError> {
        Err(TransportError::Connect(
            "evernight ws transport is a deferred skeleton; the remote wss form is postponed \
             by the routing contract until broker-side dispatch authentication lands"
                .to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_io_timeout_exceeds_server_timeout() {
        let transport = IpcSocketTransport::new("/tmp/broker.sock");
        // Explicit wire timeout always wins the max(): +30s margin.
        assert_eq!(transport.effective_io_timeout_secs(Some(60)), 90);
        assert_eq!(transport.effective_io_timeout_secs(Some(300)), 330);
        // Small explicit timeouts still clear the margin above the floor.
        assert_eq!(transport.effective_io_timeout_secs(Some(5)), 35);
        // No wire timeout: measured against evernight's default of 60s.
        assert_eq!(transport.effective_io_timeout_secs(None), 90);
    }

    #[test]
    fn configured_io_timeout_acts_as_a_floor() {
        let transport = IpcSocketTransport::with_timeouts("/tmp/broker.sock", 30, 120);
        assert_eq!(transport.io_timeout_secs(), 120);
        assert_eq!(transport.effective_io_timeout_secs(Some(60)), 120);
        assert_eq!(transport.effective_io_timeout_secs(Some(300)), 330);
    }

    #[test]
    fn socket_path_is_oversize_safe_and_reported() {
        let transport = IpcSocketTransport::with_timeouts("/tmp/broker.sock", 11, 22);
        assert_eq!(transport.socket_path(), "/tmp/broker.sock");
        assert_eq!(transport.connect_timeout_secs(), 11);
        assert_eq!(transport.io_timeout_secs(), 22);
    }

    #[test]
    fn transport_error_display_carries_source_message() {
        let error = TransportError::Connect("connection refused".to_string());
        assert!(error.to_string().contains("connection refused"));
        let error = TransportError::ReadTimeout(90);
        assert!(error.to_string().contains("90s"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn connect_to_missing_socket_yields_connect_error() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing.sock");
        let transport =
            IpcSocketTransport::with_timeouts(missing.to_string_lossy().to_string(), 2, 2);
        let request = JsonRpcRequest::new_raw("System.Ping", None);
        let error = transport.call(request).await.expect_err("must fail");
        assert!(
            matches!(error, TransportError::Connect(_)),
            "unexpected error: {error:?}"
        );
    }
}
