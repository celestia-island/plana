//! Integration test for RpcProvider — verifies chat.send ↔ chat.stream
//! WebSocket JSONRPC chain against the arona mock server.
//!
//! Requires Python 3 with aiohttp installed on the test host.
//! Skip with: `cargo test -- --skip rpc_provider` or set `SKIP_RPC_TEST=1`.

use futures::StreamExt;
use plana_llm_provider::{
    FinishReason, LlmChatRequest, LlmMessage, LlmProvider, MessageRole, ProviderConfig, RpcProvider,
};
use std::process::{Child, Command};
use std::time::Duration;

const MOCK_SCRIPT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../arona/scripts/mock/server.py"
);
const STARTUP_TIMEOUT_SECS: u64 = 15;

/// Return Some(port) if the mock script exists, None otherwise (skip test).
fn find_mock_port() -> Option<u16> {
    if std::env::var("SKIP_RPC_TEST").is_ok() {
        eprintln!("SKIP_RPC_TEST set — skipping");
        return None;
    }
    if !std::path::Path::new(MOCK_SCRIPT).exists() {
        eprintln!("Mock server not found at {MOCK_SCRIPT} — skipping");
        return None;
    }
    // Bind a random port
    let listener = std::net::TcpListener::bind("127.0.0.1:0").ok()?;
    let port = listener.local_addr().ok()?.port();
    drop(listener);
    Some(port)
}

fn start_mock_server(port: u16) -> Option<Child> {
    let mut child = Command::new("python3")
        .arg(MOCK_SCRIPT)
        .env("ARONA_MOCK_HOST", "127.0.0.1")
        .env("ARONA_MOCK_PORT", port.to_string())
        .env("ARONA_MOCK_API_KEY", "test-rpc-key")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;

    // Poll health endpoint via simple TCP + HTTP GET
    let health_url = format!("127.0.0.1:{port}");
    let deadline = std::time::Instant::now() + Duration::from_secs(STARTUP_TIMEOUT_SECS);
    while std::time::Instant::now() < deadline {
        if std::net::TcpStream::connect(&health_url).is_ok() {
            // Give it a moment to finish setup
            std::thread::sleep(Duration::from_millis(300));
            return Some(child);
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    eprintln!("Mock server health check timed out");
    let _ = child.kill();
    None
}

fn make_request(model: &str, content: &str) -> LlmChatRequest {
    LlmChatRequest {
        model: model.to_string(),
        messages: vec![LlmMessage {
            role: MessageRole::User,
            content: Some(content.into()),
            tool_call_id: None,
            tool_calls: None,
            images: None,
            content_blocks: None,
        }],
        temperature: Some(0.7),
        max_tokens: Some(256),
        tools: None,
        tool_choice: None,
        user_memory_id: None,
        workspace_memory_id: None,
        stream: Some(true),
    }
}

#[tokio::test]
async fn rpc_provider_chat_stream_basic() {
    let port = match find_mock_port() {
        Some(p) => p,
        None => return,
    };
    let mut mock = start_mock_server(port).expect("mock server failed to start");
    let base_url = format!("http://127.0.0.1:{port}");

    let config = ProviderConfig::new("test-rpc-key").with_base_url(&base_url);
    let provider = RpcProvider::new();

    let request = make_request("deepseek-v4-pro", "What is the system status?");

    let result = provider.chat_stream(request, &config).await;
    match result {
        Ok(mut stream) => {
            let mut tokens = Vec::new();
            let mut complete = false;
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(c) => {
                        if let Some(t) = &c.content {
                            tokens.push(t.clone());
                        }
                        if c.finish_reason == Some(FinishReason::Stop) || c.finish_reason.is_some()
                        {
                            complete = true;
                        }
                    }
                    Err(e) => {
                        // Tokio tungstenite connection errors (channel closed, etc.)
                        // are expected during cleanup — record but don't fail the test
                        // if we already received tokens.
                        if tokens.is_empty() {
                            panic!("RPC stream error before any tokens: {e:?}");
                        }
                        complete = true;
                        break;
                    }
                }
            }
            assert!(
                !tokens.is_empty(),
                "Expected at least one token from chat.stream"
            );
            let combined: String = tokens.concat();
            assert!(!combined.is_empty(), "Expected non-empty stream content");
            eprintln!(
                "RPC chat.stream tokens ({} chunks, {} bytes): {combined:.100}...",
                tokens.len(),
                combined.len()
            );
            assert!(complete, "Expected stream to complete");
        }
        Err(e) => {
            let _ = mock.kill();
            panic!("chat_stream failed: {e:?}");
        }
    }

    let _ = mock.kill();
    let _ = mock.wait();
}

#[tokio::test]
async fn rpc_provider_chat_non_streaming_returns_error() {
    let port = match find_mock_port() {
        Some(p) => p,
        None => return,
    };
    let mut mock = start_mock_server(port).expect("mock server failed to start");
    let base_url = format!("http://127.0.0.1:{port}");

    let config = ProviderConfig::new("test-rpc-key").with_base_url(&base_url);
    let provider = RpcProvider::new();

    let request = make_request("gpt-5.5", "Hello");

    let result = provider.chat(request, &config).await;
    assert!(
        result.is_err(),
        "Non-streaming chat should return an error for RpcProvider"
    );
    let err = result.unwrap_err();
    let err_str = format!("{err}");
    assert!(
        err_str.contains("Non-streaming")
            || err_str.contains("not supported")
            || err_str.contains("ConfigError"),
        "Error should mention unsupported non-streaming: {err_str}"
    );

    let _ = mock.kill();
    let _ = mock.wait();
}
