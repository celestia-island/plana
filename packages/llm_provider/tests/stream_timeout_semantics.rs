//! Streaming HTTP-client timeout semantics.
//!
//! `request_timeout_secs` used to apply as a TOTAL deadline to streaming
//! requests as well, aborting any generation that ran longer than the
//! window mid-flight (2026-11-17 newapi incident review: local engines can
//! take minutes before the first token on compaction-sized prompts). The
//! streaming client must instead be bounded per read (idle), with no total
//! deadline. Both gates below go red if that semantic regresses.

use std::time::Duration;

use plana_llm_provider::{LlmProvider, OpenAiCompatibleProvider, ProviderConfig};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Serve one raw HTTP response: immediate headers + a first body chunk,
/// then a quiet gap of `quiet_ms`, then a final chunk and EOF.
async fn serve_chunk_gap_chunk(listener: tokio::net::TcpListener, quiet_ms: u64) {
    let (mut sock, _) = listener.accept().await.unwrap();
    let mut buf = vec![0u8; 4096];
    let _ = sock.read(&mut buf).await; // the request head
    sock.write_all(
        b"HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\nconnection: close\r\n\r\n",
    )
    .await
    .unwrap();
    sock.write_all(b"data: first\n\n").await.unwrap();
    sock.flush().await.unwrap();
    tokio::time::sleep(Duration::from_millis(quiet_ms)).await;
    sock.write_all(b"data: second\n\n").await.unwrap();
    sock.shutdown().await.unwrap();
}

/// A stream that runs LONGER than the configured total timeout but never
/// idles past the streaming idle window must complete end-to-end. With the
/// old shape (total timeout on the streaming client) this dies at ~1s.
#[tokio::test]
async fn a_stream_outliving_the_total_timeout_still_completes() {
    let config = ProviderConfig::new("k")
        .with_request_timeout(1)
        .with_stream_idle_timeout(5);
    let client = OpenAiCompatibleProvider::new()
        .create_stream_http_client(&config)
        .unwrap();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(serve_chunk_gap_chunk(listener, 2200));

    let body = client
        .get(format!("http://{addr}/"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .expect("a stream with sub-idle gaps must survive a 1s total timeout");
    assert!(
        body.contains("first") && body.contains("second"),
        "the whole body must arrive, got: {body}"
    );
    server.await.unwrap();
}

/// A stream that STALLS past the idle window must still be cut — no total
/// deadline does not mean never timing out.
#[tokio::test]
async fn a_stalled_stream_is_cut_at_the_idle_window() {
    let config = ProviderConfig::new("k").with_stream_idle_timeout(1);
    let client = OpenAiCompatibleProvider::new()
        .create_stream_http_client(&config)
        .unwrap();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (mut sock, _) = listener.accept().await.unwrap();
        let mut buf = vec![0u8; 4096];
        let _ = sock.read(&mut buf).await;
        sock.write_all(
            b"HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\nconnection: close\r\n\r\n",
        )
        .await
        .unwrap();
        sock.write_all(b"data: first\n\n").await.unwrap();
        sock.flush().await.unwrap();
        // Stall far past the 1s idle window; the client must give up first.
        tokio::time::sleep(Duration::from_secs(6)).await;
    });

    let started = std::time::Instant::now();
    let result = client
        .get(format!("http://{addr}/"))
        .send()
        .await
        .unwrap()
        .text()
        .await;
    let elapsed = started.elapsed();
    assert!(result.is_err(), "an idling upstream must be cut");
    assert!(
        elapsed < Duration::from_secs(5),
        "cut near the idle window, not never; took {elapsed:?}"
    );
    server.abort();
}
