//! One-shot HTTP POST fallback transport (`http://` only).
//!
//! Mirrors the degraded transport of the TS client: a single JSON-RPC
//! request object POSTed to the same path the WebSocket upgrades on, one
//! response object back. No connection reuse, no notifications. Intended
//! for probes and tooling; interactive consumers should use
//! [`crate::RpcClient`].

use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use plana::jsonrpc::{Id, JsonRpcRequest, JsonRpcResponse, JSONRPC_VERSION};

use crate::RpcError;

/// POST one JSON-RPC request and return its `result`.
///
/// Application-level failures (a JSON-RPC `error` object) map to
/// [`RpcError::Rpc`]; transport-level problems map to
/// [`RpcError::Transport`]. Only plaintext `http://` URLs are supported in
/// this release.
pub async fn post_rpc(
    url: &str,
    method: &str,
    params: serde_json::Value,
    timeout: Duration,
) -> Result<serde_json::Value, RpcError> {
    let response = post_rpc_raw(url, method, params, timeout).await?;
    match response.error {
        Some(err) => Err(err.into()),
        None => Ok(response.result.unwrap_or(serde_json::Value::Null)),
    }
}

/// Same as [`post_rpc`] but returns the whole response envelope.
pub async fn post_rpc_raw(
    url: &str,
    method: &str,
    params: serde_json::Value,
    timeout: Duration,
) -> Result<JsonRpcResponse, RpcError> {
    let (host, port, path) = parse_http_url(url)?;

    let request = JsonRpcRequest {
        jsonrpc: JSONRPC_VERSION.to_string(),
        id: Some(Id::new_uuid()),
        method: method.to_string(),
        params: Some(params),
    };
    let body = serde_json::to_string(&request).map_err(|e| RpcError::Transport(e.to_string()))?;

    let io = async {
        let mut stream = tokio::net::TcpStream::connect((host.as_str(), port))
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?;

        let head = format!(
            "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream
            .write_all(head.as_bytes())
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?;
        stream
            .write_all(body.as_bytes())
            .await
            .map_err(|e| RpcError::Transport(e.to_string()))?;

        // Read until the header terminator, then until Content-Length is
        // satisfied. Connection: close makes a short read the error signal.
        let mut raw: Vec<u8> = Vec::with_capacity(1024);
        let mut chunk = [0u8; 4096];
        loop {
            let header_end = find_header_end(&raw);
            if let Some(header_end) = header_end {
                let headers = String::from_utf8_lossy(&raw[..header_end]).to_string();
                let content_length = parse_content_length(&headers)
                    .ok_or_else(|| RpcError::Transport("response missing Content-Length".into()))?;
                if raw.len() >= header_end + 4 + content_length {
                    let body = raw[header_end + 4..header_end + 4 + content_length].to_vec();
                    return Ok((headers, body));
                }
            }
            let n = stream
                .read(&mut chunk)
                .await
                .map_err(|e| RpcError::Transport(e.to_string()))?;
            if n == 0 {
                return Err(RpcError::Transport("connection closed mid-response".into()));
            }
            raw.extend_from_slice(&chunk[..n]);
        }
    };

    let (headers, body) = tokio::time::timeout(timeout, io)
        .await
        .map_err(|_| RpcError::Timeout(timeout))??;

    let status = headers.lines().next().unwrap_or_default().to_string();
    if !status.contains("200") {
        return Err(RpcError::Transport(format!(
            "unexpected status line: {status}"
        )));
    }

    serde_json::from_slice::<JsonRpcResponse>(&body)
        .map_err(|e| RpcError::Transport(format!("malformed response body: {e}")))
}

fn parse_http_url(url: &str) -> Result<(String, u16, String), RpcError> {
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| RpcError::Transport("only http:// URLs are supported".into()))?;
    let (authority, path) = match rest.split_once('/') {
        Some((authority, path)) => (authority, format!("/{path}")),
        None => (rest, "/".to_string()),
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (
            host.to_string(),
            port.parse::<u16>()
                .map_err(|_| RpcError::Transport("invalid port".into()))?,
        ),
        None => (authority.to_string(), 80),
    };
    Ok((host, port, path))
}

fn find_header_end(raw: &[u8]) -> Option<usize> {
    raw.windows(4).position(|w| w == b"\r\n\r\n")
}

fn parse_content_length(headers: &str) -> Option<usize> {
    for line in headers.lines() {
        if let Some((name, value)) = line.split_once(':') {
            if name.trim().eq_ignore_ascii_case("content-length") {
                return value.trim().parse().ok();
            }
        }
    }
    None
}
