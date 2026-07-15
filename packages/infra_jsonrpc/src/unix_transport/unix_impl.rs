use anyhow::{Context, Result, bail};
use std::{path::Path, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    net::UnixStream,
    time::Instant,
};

use tracing::{debug, info};

use crate::types::*;
use crate::unix_socket::platform;

#[derive(Debug, Clone, Copy)]
pub enum TimeoutPolicy {
    Default,
    Persistent,
    Indefinite,
    Deadline(Instant),
}

pub struct JsonRpcTransport {
    lines: Lines<BufReader<tokio::net::unix::OwnedReadHalf>>,
    write: tokio::net::unix::OwnedWriteHalf,
}

pub struct JsonRpcSender {
    write: tokio::net::unix::OwnedWriteHalf,
}

pub struct JsonRpcReceiver {
    lines: Lines<BufReader<tokio::net::unix::OwnedReadHalf>>,
}

impl JsonRpcTransport {
    pub fn new(stream: UnixStream) -> Self {
        let (read, write) = stream.into_split();
        let reader = BufReader::new(read);
        Self {
            lines: reader.lines(),
            write,
        }
    }

    pub async fn connect(socket_path: &Path) -> Result<Self> {
        let stream = platform::connect_tokio(socket_path).await?;
        Ok(Self::new(stream))
    }

    pub fn split(self) -> (JsonRpcSender, JsonRpcReceiver) {
        (
            JsonRpcSender { write: self.write },
            JsonRpcReceiver { lines: self.lines },
        )
    }

    pub async fn send(
        &mut self,
        request: &JsonRpcRequest,
        policy: TimeoutPolicy,
    ) -> Result<JsonRpcResponse> {
        match policy {
            TimeoutPolicy::Indefinite => {
                let started_at = Instant::now();
                debug!(started_at = ?started_at, policy = "Indefinite", "JSON-RPC send initiated");
                self.write_and_read_indefinite(request, started_at).await
            }
            TimeoutPolicy::Default => {
                let started_at = Instant::now();
                let deadline = started_at + Duration::from_secs(60);
                debug!(started_at = ?started_at, deadline = ?deadline, policy = "Default", "JSON-RPC send initiated");
                self.write_and_read_with_deadline(request, started_at, deadline)
                    .await
            }
            TimeoutPolicy::Persistent => {
                let started_at = Instant::now();
                let secs = std::env::var("JSONRPC_PERSISTENT_TIMEOUT_SECS")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(30);
                let deadline = started_at + Duration::from_secs(secs);
                debug!(started_at = ?started_at, deadline = ?deadline, policy = "Persistent", "JSON-RPC send initiated");
                self.write_and_read_with_deadline(request, started_at, deadline)
                    .await
            }
            TimeoutPolicy::Deadline(deadline) => {
                let started_at = Instant::now();
                debug!(started_at = ?started_at, deadline = ?deadline, policy = "Deadline", "JSON-RPC send initiated");
                self.write_and_read_with_deadline(request, started_at, deadline)
                    .await
            }
        }
    }

    async fn write_request(&mut self, request: &JsonRpcRequest) -> Result<()> {
        let mut json = serde_json::to_string(request)?;
        json.push('\n');
        self.write.write_all(json.as_bytes()).await?;
        self.write.flush().await?;
        Ok(())
    }

    fn check_id_match(request_id: &Option<Id>, response: &JsonRpcResponse) -> bool {
        Some(&response.id) == request_id.as_ref()
    }

    async fn write_and_read_indefinite(
        &mut self,
        request: &JsonRpcRequest,
        started_at: Instant,
    ) -> Result<JsonRpcResponse> {
        let request_id = request.id.clone();
        let method = request.method.clone();
        self.write_request(request).await?;
        debug!(method = %method, started_at = ?started_at, "Sent, waiting indefinitely");

        loop {
            match self.lines.next_line().await {
                Ok(Some(line)) => {
                    let value: serde_json::Value = serde_json::from_str(&line)
                        .with_context(|| format!("Failed to parse JSON-RPC message: {}", line))?;
                    if value.get("id").is_none() {
                        debug!(method = %method, "Skipping notification (no id)");
                        continue;
                    }
                    let response: JsonRpcResponse = serde_json::from_value(value)?;
                    if Self::check_id_match(&request_id, &response) {
                        let elapsed = started_at.elapsed();
                        debug!(method = %method, ?elapsed, "Received matching response");
                        return Ok(response);
                    }
                    debug!(method = %method, "Response ID mismatch, waiting");
                }
                Ok(None) => {
                    bail!(
                        "Unix socket closed while waiting for response to {}",
                        method
                    );
                }
                Err(e) => {
                    bail!("Unix socket read error waiting for {}: {}", method, e);
                }
            }
        }
    }

    async fn write_and_read_with_deadline(
        &mut self,
        request: &JsonRpcRequest,
        started_at: Instant,
        deadline: Instant,
    ) -> Result<JsonRpcResponse> {
        let request_id = request.id.clone();
        let method = request.method.clone();
        self.write_request(request).await?;
        debug!(method = %method, started_at = ?started_at, deadline = ?deadline, "Sent, waiting with deadline");

        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                let elapsed = started_at.elapsed();
                bail!(
                    "JSON-RPC request '{}' timed out: started={:?} deadline={:?} elapsed={:?}",
                    method,
                    started_at,
                    deadline,
                    elapsed
                );
            }

            match tokio::time::timeout(remaining, self.lines.next_line()).await {
                Ok(Ok(Some(line))) => {
                    let value: serde_json::Value = serde_json::from_str(&line)
                        .with_context(|| format!("Failed to parse JSON-RPC message: {}", line))?;
                    if value.get("id").is_none() {
                        debug!(method = %method, "Skipping notification (no id)");
                        continue;
                    }
                    let response: JsonRpcResponse = serde_json::from_value(value)?;
                    if Self::check_id_match(&request_id, &response) {
                        let elapsed = started_at.elapsed();
                        debug!(method = %method, ?elapsed, "Received matching response");
                        return Ok(response);
                    }
                    debug!(method = %method, "Response ID mismatch, waiting");
                }
                Ok(Ok(None)) => {
                    bail!(
                        "Unix socket closed while waiting for response to {}",
                        method
                    );
                }
                Ok(Err(e)) => {
                    bail!("Unix socket read error waiting for {}: {}", method, e);
                }
                Err(_) => {
                    let elapsed = started_at.elapsed();
                    bail!(
                        "JSON-RPC request '{}' timed out: started={:?} deadline={:?} elapsed={:?}",
                        method,
                        started_at,
                        deadline,
                        elapsed
                    );
                }
            }
        }
    }

    pub async fn send_notification(&mut self, notification: &JsonRpcNotification) -> Result<()> {
        let mut json = serde_json::to_string(notification)?;
        json.push('\n');
        self.write.write_all(json.as_bytes()).await?;
        self.write.flush().await?;
        debug!(method = %notification.method, "Sent JSON-RPC notification");
        Ok(())
    }

    pub async fn receive(&mut self) -> Result<Option<IncomingMessage>> {
        match self.lines.next_line().await? {
            Some(line) => {
                if line.trim().is_empty() {
                    return Ok(None);
                }
                let msg: JsonRpcMessage = serde_json::from_str(&line)
                    .with_context(|| format!("Failed to parse JSON-RPC message: {}", line))?;

                Ok(Some(match msg {
                    JsonRpcMessage::Request(req) => IncomingMessage::Request(req),
                    JsonRpcMessage::Notification(notif) => IncomingMessage::Notification(notif),
                    JsonRpcMessage::Response(resp) => IncomingMessage::Response(resp),
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn send_response(&mut self, response: &JsonRpcResponse) -> Result<()> {
        let mut json = serde_json::to_string(response)?;
        json.push('\n');
        self.write.write_all(json.as_bytes()).await?;
        self.write.flush().await?;
        debug!("Sent JSON-RPC response");
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        self.write.shutdown().await?;
        Ok(())
    }

    pub async fn read_line(&mut self) -> Result<Option<String>> {
        match self.lines.next_line().await {
            Ok(Some(line)) => {
                if line.trim().is_empty() {
                    return Ok(None);
                }
                Ok(Some(line))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn write_line(&mut self, line: &str) -> Result<()> {
        let mut buf = line.to_string();
        if !buf.ends_with('\n') {
            buf.push('\n');
        }
        self.write.write_all(buf.as_bytes()).await?;
        self.write.flush().await?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum IncomingMessage {
    Request(JsonRpcRequest),
    Notification(JsonRpcNotification),
    Response(JsonRpcResponse),
}

impl JsonRpcSender {
    pub async fn send_raw(&mut self, text: &str) -> Result<()> {
        let mut buf = text.to_string();
        buf.push('\n');
        self.write.write_all(buf.as_bytes()).await?;
        self.write.flush().await?;
        Ok(())
    }

    pub async fn send_response(&mut self, response: &JsonRpcResponse) -> Result<()> {
        let json = serde_json::to_string(response)?;
        self.send_raw(&json).await
    }

    pub async fn send_notification(&mut self, notification: &JsonRpcNotification) -> Result<()> {
        let json = serde_json::to_string(notification)?;
        self.send_raw(&json).await
    }

    pub async fn send_request(&mut self, request: &JsonRpcRequest) -> Result<()> {
        let json = serde_json::to_string(request)?;
        self.send_raw(&json).await
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        self.write.shutdown().await?;
        Ok(())
    }
}

impl JsonRpcReceiver {
    pub async fn receive(&mut self) -> Result<Option<IncomingMessage>> {
        match self.lines.next_line().await? {
            Some(line) => {
                if line.trim().is_empty() {
                    return Ok(None);
                }
                let msg: JsonRpcMessage = serde_json::from_str(&line)
                    .with_context(|| format!("Failed to parse JSON-RPC message: {}", line))?;
                Ok(Some(match msg {
                    JsonRpcMessage::Request(req) => IncomingMessage::Request(req),
                    JsonRpcMessage::Notification(notif) => IncomingMessage::Notification(notif),
                    JsonRpcMessage::Response(resp) => IncomingMessage::Response(resp),
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn read_line(&mut self) -> Result<Option<String>> {
        match self.lines.next_line().await? {
            Some(line) if !line.trim().is_empty() => Ok(Some(line)),
            _ => Ok(None),
        }
    }
}

pub struct JsonRpcServer {
    listener: tokio::net::UnixListener,
}

impl JsonRpcServer {
    pub async fn bind(socket_path: &Path) -> Result<Self> {
        let listener = platform::bind_tokio(socket_path).await?;
        Ok(Self { listener })
    }

    pub async fn accept(&self) -> Result<JsonRpcTransport> {
        let (stream, addr) = self.listener.accept().await?;
        info!(addr = ?addr, "Accepted Unix socket connection");
        Ok(JsonRpcTransport::new(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
    use anyhow::Error;
    use anyhow::Result;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_server_bind_and_connect() -> Result<()> {
        let dir = TempDir::new()?;
        let socket_path = dir.path().join("test.sock");

        let server = JsonRpcServer::bind(&socket_path).await?;
        assert!(socket_path.exists());

        let server_handle = tokio::spawn(async move {
            let mut transport = server.accept().await?;
            let msg = transport.receive().await?.context("expected message")?;
            if let IncomingMessage::Request(req) = msg {
                let resp = JsonRpcResponse::success(
                    req.id.clone().unwrap_or(Id::Null),
                    serde_json::json!({"echo": &req.method}),
                );
                transport.send_response(&resp).await?;
            }
            Ok::<_, Error>(())
        });

        let mut client = JsonRpcTransport::connect(&socket_path).await?;
        let request = JsonRpcRequest::new_raw("test.echo", Some(serde_json::json!({"data": 42})));
        let response = client.send(&request, TimeoutPolicy::Default).await?;

        assert!(response.result.is_some());
        assert!(response.error.is_none());

        server_handle.await??;
        Ok(())
    }

    #[tokio::test]
    async fn test_notification_no_response() -> Result<()> {
        let dir = TempDir::new()?;
        let socket_path = dir.path().join("notif.sock");

        let server = JsonRpcServer::bind(&socket_path).await?;

        let server_handle = tokio::spawn(async move {
            let mut transport = server.accept().await?;
            let msg = transport.receive().await?.context("expected message")?;
            match msg {
                IncomingMessage::Notification(notif) => {
                    assert_eq!(notif.method, "event.test");
                }
                IncomingMessage::Request(req) => {
                    assert_eq!(req.method, "event.test");
                }
                other => bail!("Expected request or notification, got {:?}", other),
            }
            Ok::<_, Error>(())
        });

        let mut client = JsonRpcTransport::connect(&socket_path).await?;
        let notif =
            JsonRpcNotification::new_raw("event.test", Some(serde_json::json!({"msg": "hello"})));
        client.send_notification(&notif).await?;

        server_handle.await??;
        Ok(())
    }
}
