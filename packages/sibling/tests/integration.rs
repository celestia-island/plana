//! Integration test matrix for plana-sibling (T3-S1, design §6 S1):
//! failover order, retry semantics, per-class errors, no_proxy default,
//! layered timeouts, endpoint_provider callback, AllEndpointsDown
//! aggregation, bearer injection. All traffic is wiremock on 127.0.0.1.

use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use plana_sibling::{
    AttemptReason, EndpointProvider, Retry, SiblingBuildError, SiblingError, SiblingService,
};
use secrecy::SecretString;
use serde_json::json;
use wiremock::matchers::{any, body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// A port that (almost certainly) refuses connections: bind, read the port,
/// drop the listener. Standard trick for a deterministic "dead endpoint".
fn refused_endpoint() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    format!("http://127.0.0.1:{port}")
}

fn ok_mock(body: serde_json::Value) -> Mock {
    Mock::given(any()).respond_with(ResponseTemplate::new(200).set_body_json(body))
}

fn status_mock(code: u16, body: &str) -> Mock {
    Mock::given(any()).respond_with(ResponseTemplate::new(code).set_body_string(body.to_string()))
}

fn fixed(name: &str, urls: &[&str]) -> SiblingService {
    let mut builder = SiblingService::new(name);
    for url in urls {
        builder = builder.endpoint_fixed(*url);
    }
    builder.build().unwrap()
}

/// Process-global serialization for tests that mutate proxy-related env
/// (the only env all services implicitly agree on is the proxy namespace;
/// every other test uses a unique variable name and needs no lock).
static PROXY_ENV_LOCK: Mutex<()> = Mutex::new(());

fn set_env(key: &str, value: &str) {
    // edition 2024: env mutation is unsafe (process-global).
    unsafe { std::env::set_var(key, value) };
}

fn remove_env(key: &str) {
    unsafe { std::env::remove_var(key) };
}

/// Save-and-restore a set of environment variables for the test's lifetime.
/// `_guard = ...` bindings hold the restore-on-drop to scope end.
struct EnvGuard {
    saved: Vec<(String, Option<String>)>,
}

impl EnvGuard {
    fn set(vars: &[&str], value: &str) -> Self {
        let mut saved = Vec::with_capacity(vars.len());
        for var in vars {
            saved.push(((*var).to_string(), std::env::var(var).ok()));
            set_env(var, value);
        }
        EnvGuard { saved }
    }

    fn remove(vars: &[&str]) -> Self {
        let mut saved = Vec::with_capacity(vars.len());
        for var in vars {
            saved.push(((*var).to_string(), std::env::var(var).ok()));
            remove_env(var);
        }
        EnvGuard { saved }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, value) in &self.saved {
            match value {
                Some(v) => set_env(key, v),
                None => remove_env(key),
            }
        }
    }
}

/// EndpointProvider backed by a mutable list (dynamic per-call semantics).
struct ListProvider(Mutex<Vec<String>>);

impl ListProvider {
    fn new(list: Vec<String>) -> Self {
        ListProvider(Mutex::new(list))
    }

    fn set(&self, list: Vec<String>) {
        *self.0.lock().unwrap() = list;
    }
}

impl EndpointProvider for ListProvider {
    fn endpoints(&self) -> Vec<String> {
        self.0.lock().unwrap().clone()
    }
}

/// Assert the error is Timeout from `service` and fired well before a 3 s
/// mock delay would have completed (i.e. the configured layer actually cut
/// the attempt short, not the 10 s default total).
fn assert_timeout(err: SiblingError, service: &str) {
    match err {
        SiblingError::Timeout { service: s, after } => {
            assert_eq!(s, service);
            assert!(after >= Duration::from_millis(100), "too short: {after:?}");
            assert!(
                after < Duration::from_millis(1500),
                "timeout fired late (layer ignored?): {after:?}"
            );
        }
        other => panic!("expected Timeout, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// build-time fail-fast
// ---------------------------------------------------------------------------

#[test]
fn build_fails_without_any_endpoint_source() {
    let err = SiblingService::new("arona").build().unwrap_err();
    assert!(
        matches!(err, SiblingBuildError::NoEndpointSource(_)),
        "{err:?}"
    );
}

#[test]
fn build_fails_with_empty_name() {
    let err = SiblingService::new("")
        .endpoint_fixed("http://192.0.2.1")
        .build()
        .unwrap_err();
    assert!(matches!(err, SiblingBuildError::NameEmpty), "{err:?}");
}

#[test]
fn build_fails_when_endpoints_env_missing() {
    let _guard = EnvGuard::remove(&["T3SIB_URLS_MISSING"]);
    let err = SiblingService::new("arona")
        .endpoints_env("T3SIB_URLS_MISSING")
        .build()
        .unwrap_err();
    assert!(
        matches!(err, SiblingBuildError::EnvMissing { .. }),
        "{err:?}"
    );
}

#[test]
fn build_fails_when_endpoints_env_yields_nothing() {
    let _guard = EnvGuard::set(&["T3SIB_URLS_EMPTY"], "  ,  , ");
    let err = SiblingService::new("arona")
        .endpoints_env("T3SIB_URLS_EMPTY")
        .build()
        .unwrap_err();
    assert!(matches!(err, SiblingBuildError::EnvEmpty { .. }), "{err:?}");
}

#[test]
fn build_fails_on_invalid_fixed_endpoint() {
    let err = SiblingService::new("arona")
        .endpoint_fixed("ftp://192.0.2.1")
        .build()
        .unwrap_err();
    assert!(
        matches!(err, SiblingBuildError::InvalidEndpoint { .. }),
        "{err:?}"
    );
}

#[test]
fn build_fails_when_bearer_env_missing() {
    let _guard = EnvGuard::remove(&["T3SIB_TOKEN_MISSING"]);
    let err = SiblingService::new("arona")
        .endpoint_fixed("http://192.0.2.1")
        .bearer_env("T3SIB_TOKEN_MISSING")
        .build()
        .unwrap_err();
    assert!(
        matches!(err, SiblingBuildError::TokenMissing { .. }),
        "{err:?}"
    );
}

// ---------------------------------------------------------------------------
// failover order
// ---------------------------------------------------------------------------

#[tokio::test]
async fn failover_skips_dead_endpoint_in_order() {
    let server = MockServer::start().await;
    ok_mock(json!({"ok": true})).mount(&server).await;

    let _guard = EnvGuard::set(
        &["T3SIB_URLS_FAILOVER"],
        &format!("{},{}", refused_endpoint(), server.uri()),
    );
    let svc = SiblingService::new("arona")
        .endpoints_env("T3SIB_URLS_FAILOVER")
        .build()
        .unwrap();

    let out: serde_json::Value = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .retry(Retry::none())
        .send()
        .await
        .unwrap();
    assert_eq!(out, json!({"ok": true}));

    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn failover_tries_second_endpoint_after_5xx() {
    let first = MockServer::start().await;
    let second = MockServer::start().await;
    status_mock(500, "boom").mount(&first).await;
    ok_mock(json!({"second": true})).mount(&second).await;

    let _guard = EnvGuard::set(
        &["T3SIB_URLS_TWO"],
        &format!("{},{}", first.uri(), second.uri()),
    );
    let svc = SiblingService::new("arona")
        .endpoints_env("T3SIB_URLS_TWO")
        .build()
        .unwrap();

    let out: serde_json::Value = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .retry(Retry::none())
        .send()
        .await
        .unwrap();
    assert_eq!(out, json!({"second": true}));
    assert_eq!(first.received_requests().await.unwrap().len(), 1);
    assert_eq!(second.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn endpoint_declaration_order_is_respected() {
    // env source declared BEFORE fixed source: A then B.
    let a = MockServer::start().await;
    let b = MockServer::start().await;
    status_mock(500, "a down").mount(&a).await;
    status_mock(500, "b down").mount(&b).await;

    let _guard = EnvGuard::set(&["T3SIB_URLS_ORDER"], a.uri().as_str());
    let svc = SiblingService::new("arona")
        .endpoints_env("T3SIB_URLS_ORDER")
        .endpoint_fixed(b.uri())
        .build()
        .unwrap();

    let err = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .retry(Retry::none())
        .send::<serde_json::Value>()
        .await
        .unwrap_err();

    match err {
        SiblingError::AllEndpointsDown { service, tried } => {
            assert_eq!(service, "arona");
            assert_eq!(tried.len(), 2);
            assert_eq!(tried[0].endpoint, a.uri());
            assert_eq!(tried[1].endpoint, b.uri());
            assert_eq!(tried[0].reason, AttemptReason::Status(500));
            assert_eq!(tried[1].reason, AttemptReason::Status(500));
        }
        other => panic!("expected AllEndpointsDown, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// retry semantics
// ---------------------------------------------------------------------------

#[tokio::test]
async fn retry_none_is_a_single_pass_without_cycling() {
    let a = MockServer::start().await;
    let b = MockServer::start().await;
    status_mock(500, "a").mount(&a).await;
    status_mock(500, "b").mount(&b).await;

    let _guard = EnvGuard::set(&["T3SIB_URLS_RETRY0"], &format!("{},{}", a.uri(), b.uri()));
    let svc = SiblingService::new("arona")
        .endpoints_env("T3SIB_URLS_RETRY0")
        .build()
        .unwrap();

    let err = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .retry(Retry::none())
        .send::<serde_json::Value>()
        .await
        .unwrap_err();

    assert!(
        matches!(err, SiblingError::AllEndpointsDown { .. }),
        "{err:?}"
    );
    assert_eq!(a.received_requests().await.unwrap().len(), 1);
    assert_eq!(b.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn retry_idempotent_cycles_back_through_the_list() {
    let a = MockServer::start().await;
    let b = MockServer::start().await;
    status_mock(500, "a").mount(&a).await;
    status_mock(500, "b").mount(&b).await;

    let _guard = EnvGuard::set(&["T3SIB_URLS_RETRY2"], &format!("{},{}", a.uri(), b.uri()));
    let svc = SiblingService::new("arona")
        .endpoints_env("T3SIB_URLS_RETRY2")
        .build()
        .unwrap();

    let err = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .retry(Retry::idempotent(1))
        .send::<serde_json::Value>()
        .await
        .unwrap_err();

    match err {
        SiblingError::AllEndpointsDown { tried, .. } => {
            // A, B, then the cycling extra attempt back to A.
            assert_eq!(tried.len(), 3);
            assert_eq!(tried[0].endpoint, a.uri());
            assert_eq!(tried[1].endpoint, b.uri());
            assert_eq!(tried[2].endpoint, a.uri());
        }
        other => panic!("expected AllEndpointsDown, got: {other:?}"),
    }
    assert_eq!(a.received_requests().await.unwrap().len(), 2);
    assert_eq!(b.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn post_without_key_fails_over_across_never_delivered() {
    let alive = MockServer::start().await;
    ok_mock(json!({"ok": true})).mount(&alive).await;

    let _guard = EnvGuard::set(
        &["T3SIB_URLS_POST"],
        &format!("{},{}", refused_endpoint(), alive.uri()),
    );
    let svc = SiblingService::new("arona")
        .endpoints_env("T3SIB_URLS_POST")
        .build()
        .unwrap();

    // A connect-phase failure (refused) proves the request reached no
    // server, so even a POST without an idempotency key advances to the
    // next endpoint — replay cannot double-execute what nothing received.
    let out: serde_json::Value = svc
        .call(plana_sibling::Method::POST, "/api/x", &json!({"a": 1}))
        .retry(Retry::idempotent(3))
        .send()
        .await
        .unwrap();
    assert_eq!(out, json!({"ok": true}));
    // Exactly one hit: non-idempotent never cycles, the advance is only
    // ever to the NEXT endpoint after a never-delivered failure.
    assert_eq!(alive.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn post_read_timeout_does_not_replay_elsewhere() {
    // Endpoint 1 DELIVERS the request, then stalls past the read-timeout
    // layer: the server may have processed it, so a POST without an
    // idempotency key must stop there — the healthy endpoint 2 must never
    // see the request.
    let stalling = MockServer::start().await;
    Mock::given(any())
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"ok": true}))
                .set_delay(Duration::from_secs(3)),
        )
        .mount(&stalling)
        .await;
    let alive = MockServer::start().await;
    ok_mock(json!({"ok": true})).mount(&alive).await;

    let svc = SiblingService::new("arona")
        .endpoint_fixed(stalling.uri())
        .endpoint_fixed(alive.uri())
        .read_timeout(Duration::from_millis(300))
        .build()
        .unwrap();

    let err = svc
        .call(plana_sibling::Method::POST, "/api/x", &json!({"a": 1}))
        .send::<serde_json::Value>()
        .await
        .unwrap_err();
    // The delivered/unknown failure is terminal with its specific class.
    assert_timeout(err, "arona");
    // The safety pin: no replay against the second endpoint.
    assert_eq!(alive.received_requests().await.unwrap().len(), 0);
}

#[tokio::test]
async fn post_all_endpoints_refused_reports_all_down() {
    let a = refused_endpoint();
    let b = refused_endpoint();
    let svc = fixed("arona", &[a.as_str(), b.as_str()]);
    let err = svc
        .call(plana_sibling::Method::POST, "/api/x", &json!({"a": 1}))
        .send::<serde_json::Value>()
        .await
        .unwrap_err();
    match err {
        SiblingError::AllEndpointsDown { tried, .. } => {
            assert_eq!(tried.len(), 2);
            assert!(
                tried
                    .iter()
                    .all(|t| matches!(t.reason, AttemptReason::Unreachable(_)))
            );
        }
        other => panic!("expected AllEndpointsDown, got: {other:?}"),
    }
}

#[tokio::test]
async fn post_500_does_not_advance_without_key() {
    let first = MockServer::start().await;
    let alive = MockServer::start().await;
    status_mock(500, "boom").mount(&first).await;
    ok_mock(json!({"ok": true})).mount(&alive).await;

    let svc = fixed("arona", &[first.uri().as_str(), alive.uri().as_str()]);
    let err = svc
        .call(plana_sibling::Method::POST, "/api/x", &json!({"a": 1}))
        .send::<serde_json::Value>()
        .await
        .unwrap_err();

    match err {
        SiblingError::Status { code, body, .. } => {
            assert_eq!(code, 500);
            assert_eq!(body, "boom");
        }
        other => panic!("expected Status, got: {other:?}"),
    }
    assert_eq!(alive.received_requests().await.unwrap().len(), 0);
}

#[tokio::test]
async fn post_with_idempotency_key_fails_over_like_get() {
    let first = MockServer::start().await;
    let alive = MockServer::start().await;
    status_mock(500, "boom").mount(&first).await;
    // The key header itself must reach the endpoint (downstream dedup
    // depends on it), so the failover target only answers requests that
    // carry it.
    Mock::given(method("POST"))
        .and(header("idempotency-key", "t3sib-op-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .mount(&alive)
        .await;

    let svc = fixed("arona", &[first.uri().as_str(), alive.uri().as_str()]);
    let out: serde_json::Value = svc
        .call(plana_sibling::Method::POST, "/api/x", &json!({"a": 1}))
        .idempotency_key("t3sib-op-1")
        .retry(Retry::none())
        .send()
        .await
        .unwrap();
    assert_eq!(out, json!({"ok": true}));
    assert_eq!(first.received_requests().await.unwrap().len(), 1);
    assert_eq!(alive.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn four_xx_is_terminal_and_never_advances() {
    let first = MockServer::start().await;
    let alive = MockServer::start().await;
    status_mock(404, "nope").mount(&first).await;
    ok_mock(json!({"ok": true})).mount(&alive).await;

    let _guard = EnvGuard::set(
        &["T3SIB_URLS_4XX"],
        &format!("{},{}", first.uri(), alive.uri()),
    );
    let svc = SiblingService::new("arona")
        .endpoints_env("T3SIB_URLS_4XX")
        .build()
        .unwrap();

    let err = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .retry(Retry::idempotent(5))
        .send::<serde_json::Value>()
        .await
        .unwrap_err();

    match err {
        SiblingError::Status { code, body, .. } => {
            assert_eq!(code, 404);
            assert_eq!(body, "nope");
        }
        other => panic!("expected Status, got: {other:?}"),
    }
    assert_eq!(alive.received_requests().await.unwrap().len(), 0);
}

// ---------------------------------------------------------------------------
// error classes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn unreachable_is_reported_with_endpoint_context() {
    // A single refused endpoint on a POST: the connect-phase failure
    // advances (nothing was delivered), the list is exhausted, and the
    // aggregation carries the per-endpoint record.
    let dead = refused_endpoint();
    let svc = fixed("arona", &[dead.as_str()]);
    let err = svc
        .call(plana_sibling::Method::POST, "/api/x", &json!({"a": 1}))
        .send::<serde_json::Value>()
        .await
        .unwrap_err();
    match err {
        SiblingError::AllEndpointsDown { service, tried, .. } => {
            assert_eq!(service, "arona");
            assert_eq!(tried.len(), 1);
            assert_eq!(tried[0].endpoint, dead);
            match &tried[0].reason {
                AttemptReason::Unreachable(cause) => assert!(!cause.is_empty()),
                other => panic!("expected Unreachable reason, got: {other:?}"),
            }
        }
        other => panic!("expected AllEndpointsDown, got: {other:?}"),
    }
}

#[tokio::test]
async fn single_dead_endpoint_get_reports_all_endpoints_down_with_record() {
    let dead = refused_endpoint();
    let svc = fixed("arona", &[dead.as_str()]);
    let err = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .send::<serde_json::Value>()
        .await
        .unwrap_err();
    match err {
        SiblingError::AllEndpointsDown { tried, .. } => {
            assert_eq!(tried.len(), 1);
            assert_eq!(tried[0].endpoint, dead);
            assert!(matches!(tried[0].reason, AttemptReason::Unreachable(_)));
        }
        other => panic!("expected AllEndpointsDown, got: {other:?}"),
    }
}

#[tokio::test]
async fn all_endpoints_down_aggregates_every_attempt() {
    let a = MockServer::start().await;
    let b = MockServer::start().await;
    status_mock(500, "a").mount(&a).await;
    status_mock(503, "b").mount(&b).await;

    let _guard = EnvGuard::set(
        &["T3SIB_URLS_AGGR"],
        &format!("{},{},{}", refused_endpoint(), a.uri(), b.uri()),
    );
    let svc = SiblingService::new("arona")
        .endpoints_env("T3SIB_URLS_AGGR")
        .build()
        .unwrap();

    let err = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .retry(Retry::none())
        .send::<serde_json::Value>()
        .await
        .unwrap_err();

    match err {
        SiblingError::AllEndpointsDown { tried, .. } => {
            assert_eq!(tried.len(), 3);
            assert!(matches!(tried[0].reason, AttemptReason::Unreachable(_)));
            assert_eq!(tried[1].reason, AttemptReason::Status(500));
            assert_eq!(tried[2].reason, AttemptReason::Status(503));
            assert_eq!(a.received_requests().await.unwrap().len(), 1);
            assert_eq!(b.received_requests().await.unwrap().len(), 1);
        }
        other => panic!("expected AllEndpointsDown, got: {other:?}"),
    }
}

#[tokio::test]
async fn empty_endpoint_list_reports_all_endpoints_down() {
    let provider = Arc::new(ListProvider::new(Vec::new()));
    let svc = SiblingService::new("arona")
        .endpoint_provider(provider)
        .build()
        .unwrap();
    let err = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .send::<serde_json::Value>()
        .await
        .unwrap_err();
    match err {
        SiblingError::AllEndpointsDown { tried, .. } => assert!(tried.is_empty()),
        other => panic!("expected AllEndpointsDown, got: {other:?}"),
    }
}

#[tokio::test]
async fn decode_error_on_non_json_response() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200).set_body_string("not json at all"))
        .mount(&server)
        .await;

    let svc = fixed("arona", &[server.uri().as_str()]);
    let err = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .send::<serde_json::Value>()
        .await
        .unwrap_err();
    match err {
        SiblingError::Decode { cause, .. } => {
            assert!(cause.starts_with("response body:"), "{cause}");
        }
        other => panic!("expected Decode, got: {other:?}"),
    }
}

#[tokio::test]
async fn decode_error_on_unserializable_request_body() {
    let server = MockServer::start().await;
    ok_mock(json!({"ok": true})).mount(&server).await;

    let svc = fixed("arona", &[server.uri().as_str()]);
    // serde_json rejects non-string map keys — an unserializable request body.
    let body: std::collections::BTreeMap<Vec<u8>, u32> =
        std::collections::BTreeMap::from([(vec![1u8, 2u8], 3u32)]);
    let err = svc
        .call(plana_sibling::Method::POST, "/api/x", &body)
        .send::<serde_json::Value>()
        .await
        .unwrap_err();
    match err {
        SiblingError::Decode { cause, .. } => {
            assert!(cause.starts_with("request body:"), "{cause}");
        }
        other => panic!("expected Decode, got: {other:?}"),
    }
    // The endpoint must never have been contacted.
    assert_eq!(server.received_requests().await.unwrap().len(), 0);
}

#[tokio::test]
async fn status_error_carries_truncated_body() {
    let long_body = "x".repeat(4096);
    let server = MockServer::start().await;
    status_mock(418, &long_body).mount(&server).await;

    let svc = fixed("arona", &[server.uri().as_str()]);
    let err = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .send::<serde_json::Value>()
        .await
        .unwrap_err();
    match err {
        SiblingError::Status { code, body, .. } => {
            assert_eq!(code, 418);
            assert!(body.len() < long_body.len(), "body must be truncated");
            assert!(body.contains("bytes total"));
        }
        other => panic!("expected Status, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// layered timeouts
// ---------------------------------------------------------------------------

#[tokio::test]
async fn total_timeout_bounds_the_whole_call() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"ok": true}))
                .set_delay(Duration::from_secs(3)),
        )
        .mount(&server)
        .await;

    let svc = fixed("arona", &[server.uri().as_str()]);
    // Non-idempotent POST: the specific Timeout class surfaces directly
    // (idempotent exhaustion aggregates into AllEndpointsDown instead).
    let err = svc
        .call(plana_sibling::Method::POST, "/api/x", &json!({"a": 1}))
        .timeout(Duration::from_millis(300))
        .send::<serde_json::Value>()
        .await
        .unwrap_err();
    assert_timeout(err, "arona");
}

#[tokio::test]
async fn total_timeout_is_shared_across_failover_attempts() {
    // Both endpoints stall past the budget. The total deadline is a
    // call-level wall clock: it expires ONCE at 300 ms and reports Timeout —
    // NOT a per-attempt reset that would try endpoint B and come back as
    // AllEndpointsDown after ~600 ms.
    let a = MockServer::start().await;
    let b = MockServer::start().await;
    for server in [&a, &b] {
        Mock::given(any())
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"ok": true}))
                    .set_delay(Duration::from_secs(3)),
            )
            .mount(server)
            .await;
    }

    let _guard = EnvGuard::set(&["T3SIB_URLS_TOTAL"], &format!("{},{}", a.uri(), b.uri()));
    let svc = SiblingService::new("arona")
        .endpoints_env("T3SIB_URLS_TOTAL")
        .total_timeout(Duration::from_millis(300))
        .build()
        .unwrap();

    let err = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .retry(Retry::none())
        .send::<serde_json::Value>()
        .await
        .unwrap_err();
    match err {
        SiblingError::Timeout { service, after } => {
            assert_eq!(service, "arona");
            assert!(after >= Duration::from_millis(250), "{after:?}");
            assert!(after < Duration::from_millis(1500), "{after:?}");
        }
        other => panic!("expected call-level Timeout, got: {other:?}"),
    }
    // The budget died inside attempt 1 — endpoint B must never be touched.
    assert_eq!(a.received_requests().await.unwrap().len(), 1);
    assert_eq!(b.received_requests().await.unwrap().len(), 0);
}

#[tokio::test]
async fn read_timeout_layer_fires_at_build_level() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"ok": true}))
                .set_delay(Duration::from_secs(3)),
        )
        .mount(&server)
        .await;

    // total stays at the 10 s default; the 300 ms read layer must cut it.
    let svc = SiblingService::new("arona")
        .endpoint_fixed(server.uri())
        .read_timeout(Duration::from_millis(300))
        .build()
        .unwrap();
    let err = svc
        .call(plana_sibling::Method::POST, "/api/x", &json!({"a": 1}))
        .send::<serde_json::Value>()
        .await
        .unwrap_err();
    assert_timeout(err, "arona");
}

#[tokio::test]
async fn connect_timeout_config_builds_and_still_serves() {
    let server = MockServer::start().await;
    ok_mock(json!({"ok": true})).mount(&server).await;

    // A deterministic connect-timeout fire needs a SYN-blackhole network
    // partition; here we pin the layer to the config surface: the client
    // builds with the layer set and plain loopback traffic still flows.
    let svc = SiblingService::new("arona")
        .endpoint_fixed(server.uri())
        .connect_timeout(Duration::from_millis(100))
        .total_timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    let out: serde_json::Value = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .send()
        .await
        .unwrap();
    assert_eq!(out, json!({"ok": true}));
}

// ---------------------------------------------------------------------------
// no_proxy default / via_proxy opt-in
// ---------------------------------------------------------------------------

#[tokio::test]
async fn no_proxy_is_the_default_even_when_env_proxy_is_set() {
    let server = MockServer::start().await;
    ok_mock(json!({"ok": true})).mount(&server).await;

    // If the default `.no_proxy()` were missing, this dead proxy would
    // swallow the request and the call would fail. NO_PROXY/no_proxy must
    // go too — on runners where it contains 127.0.0.1 they would mask the
    // mutation and keep the test green (R2 S2-2). reqwest reads proxy env
    // at client build time, so the lock + env only need to cover `build`.
    let svc = {
        let _lock = PROXY_ENV_LOCK.lock().unwrap();
        let _no_proxy_guard = EnvGuard::remove(&["NO_PROXY", "no_proxy"]);
        let _guard = EnvGuard::set(
            &["HTTP_PROXY", "http_proxy", "ALL_PROXY", "all_proxy"],
            refused_endpoint().as_str(),
        );
        fixed("arona", &[server.uri().as_str()])
    };

    let out: serde_json::Value = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .send()
        .await
        .unwrap();
    assert_eq!(out, json!({"ok": true}));
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn via_proxy_opts_in_to_env_proxy() {
    let target = MockServer::start().await;
    ok_mock(json!({"ok": true})).mount(&target).await;
    let proxy = MockServer::start().await;
    ok_mock(json!({"via": "proxy"})).mount(&proxy).await;

    let svc = {
        let _lock = PROXY_ENV_LOCK.lock().unwrap();
        // Remove NO_PROXY so the proxy is actually honored (also at build).
        let _no_proxy_guard = EnvGuard::remove(&["NO_PROXY", "no_proxy"]);
        let _guard = EnvGuard::set(&["HTTP_PROXY", "http_proxy"], proxy.uri().as_str());
        SiblingService::new("arona")
            .endpoint_fixed(target.uri())
            .via_proxy()
            .build()
            .unwrap()
    };

    let out: serde_json::Value = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .send()
        .await
        .unwrap();
    // The answer comes from the proxy front, not the target.
    assert_eq!(out, json!({"via": "proxy"}));
    assert_eq!(target.received_requests().await.unwrap().len(), 0);

    let proxied = proxy.received_requests().await.unwrap();
    assert_eq!(proxied.len(), 1);
    assert!(
        proxied[0].url.as_str().contains(target.uri().as_str()),
        "proxy should have seen the absolute-form target URI, got: {}",
        proxied[0].url
    );
}

// ---------------------------------------------------------------------------
// credentials
// ---------------------------------------------------------------------------

#[tokio::test]
async fn bearer_env_injects_authorization_header() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(header("authorization", "Bearer fake-test-token-do-not-use"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .mount(&server)
        .await;

    let _guard = EnvGuard::set(&["T3SIB_TOKEN_BEARER"], "fake-test-token-do-not-use");
    let svc = SiblingService::new("arona")
        .endpoint_fixed(server.uri())
        .bearer_env("T3SIB_TOKEN_BEARER")
        .build()
        .unwrap();
    let out: serde_json::Value = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .send()
        .await
        .unwrap();
    assert_eq!(out, json!({"ok": true}));
}

#[tokio::test]
async fn no_bearer_means_no_authorization_header() {
    let server = MockServer::start().await;
    ok_mock(json!({"ok": true})).mount(&server).await;

    let svc = fixed("arona", &[server.uri().as_str()]);
    let out: serde_json::Value = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .send()
        .await
        .unwrap();
    assert_eq!(out, json!({"ok": true}));
    let reqs = server.received_requests().await.unwrap();
    assert_eq!(reqs.len(), 1);
    assert!(reqs[0].headers.get("authorization").is_none());
}

#[tokio::test]
async fn bearer_secret_direct_takes_precedence() {
    let server = MockServer::start().await;
    Mock::given(header("authorization", "Bearer direct-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .mount(&server)
        .await;

    let _guard = EnvGuard::set(&["T3SIB_TOKEN_PRECEDE"], "env-value");
    let svc = SiblingService::new("arona")
        .endpoint_fixed(server.uri())
        .bearer_env("T3SIB_TOKEN_PRECEDE")
        .bearer_secret(SecretString::new("direct-secret".to_string()))
        .build()
        .unwrap();
    let out: serde_json::Value = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .send()
        .await
        .unwrap();
    assert_eq!(out, json!({"ok": true}));
}

#[tokio::test]
async fn token_never_appears_in_error_or_debug_output() {
    let server = MockServer::start().await;
    status_mock(500, "boom").mount(&server).await;

    let svc = SiblingService::new("arona")
        .endpoint_fixed(server.uri())
        .bearer_secret(SecretString::new("fake-secret-must-not-leak".to_string()))
        .build()
        .unwrap();
    let err = svc
        .call(plana_sibling::Method::POST, "/api/x", &json!({"a": 1}))
        .send::<serde_json::Value>()
        .await
        .unwrap_err();
    assert!(
        !format!("{err}").contains("fake-secret-must-not-leak"),
        "token leaked in error display"
    );
    assert!(
        !format!("{svc:?}").contains("fake-secret-must-not-leak"),
        "token leaked in debug output"
    );
}

// ---------------------------------------------------------------------------
// endpoint_provider callback
// ---------------------------------------------------------------------------

#[tokio::test]
async fn provider_endpoints_are_used_with_failover() {
    let alive = MockServer::start().await;
    ok_mock(json!({"ok": true})).mount(&alive).await;

    let provider = Arc::new(ListProvider::new(vec![refused_endpoint(), alive.uri()]));
    let svc = SiblingService::new("arona")
        .endpoint_provider(provider)
        .build()
        .unwrap();

    let out: serde_json::Value = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .retry(Retry::none())
        .send()
        .await
        .unwrap();
    assert_eq!(out, json!({"ok": true}));
}

#[tokio::test]
async fn provider_is_queried_again_on_every_call() {
    let alive = MockServer::start().await;
    ok_mock(json!({"ok": true})).mount(&alive).await;

    let provider = Arc::new(ListProvider::new(vec![refused_endpoint()]));
    let svc = SiblingService::new("arona")
        .endpoint_provider(provider.clone())
        .build()
        .unwrap();

    // First call: provider still points at the dead endpoint.
    let err = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .send::<serde_json::Value>()
        .await
        .unwrap_err();
    assert!(
        matches!(err, SiblingError::AllEndpointsDown { .. }),
        "{err:?}"
    );

    // The operator re-points the provider (e.g. HA failover landed); the
    // very next call picks it up without a restart.
    provider.set(vec![alive.uri()]);
    let out: serde_json::Value = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .send()
        .await
        .unwrap();
    assert_eq!(out, json!({"ok": true}));
}

#[tokio::test]
async fn provider_slot_splices_in_at_declaration_position() {
    let a = MockServer::start().await;
    let b = MockServer::start().await;
    status_mock(500, "a").mount(&a).await;
    status_mock(500, "b").mount(&b).await;

    let provider = Arc::new(ListProvider::new(vec![b.uri()]));
    let svc = SiblingService::new("arona")
        .endpoint_fixed(a.uri())
        .endpoint_provider(provider)
        .build()
        .unwrap();

    let err = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .retry(Retry::none())
        .send::<serde_json::Value>()
        .await
        .unwrap_err();
    match err {
        SiblingError::AllEndpointsDown { tried, .. } => {
            assert_eq!(tried.len(), 2);
            assert_eq!(tried[0].endpoint, a.uri());
            assert_eq!(tried[1].endpoint, b.uri());
        }
        other => panic!("expected AllEndpointsDown, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// URL joining & body transport
// ---------------------------------------------------------------------------

#[tokio::test]
async fn path_join_handles_trailing_slash_and_relative_path() {
    let server = MockServer::start().await;
    // Only /api/x counts — proves the root "/"+relative "api/x" joined right.
    Mock::given(path("/api/x"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .mount(&server)
        .await;

    let svc = fixed("arona", &[format!("{}/", server.uri()).as_str()]);
    let out: serde_json::Value = svc
        .call(plana_sibling::Method::GET, "api/x", &())
        .send()
        .await
        .unwrap();
    assert_eq!(out, json!({"ok": true}));
}

#[tokio::test]
async fn post_body_roundtrips_as_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/rpc"))
        .and(body_json(json!({"id": 7, "op": "ping"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"pong": 7})))
        .mount(&server)
        .await;

    let svc = fixed("arona", &[server.uri().as_str()]);
    let out: serde_json::Value = svc
        .call(
            plana_sibling::Method::POST,
            "/api/rpc",
            &json!({"id": 7, "op": "ping"}),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(out, json!({"pong": 7}));
}

#[tokio::test]
async fn post_body_is_labeled_as_json() {
    // Regression pin (2026-10-30): axum-backed siblings (arona /api/rpc)
    // answer a JSON body without `Content-Type: application/json` with
    // 415 — which callers classify as "unreachable", the empty-roster
    // outage. wiremock's `body_json` matcher alone never enforced the
    // header, so the mock below gates on it explicitly: an unlabeled
    // body falls through to the unmatched 404 and this test fails.
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/rpc"))
        .and(header("content-type", "application/json"))
        .and(body_json(json!({"id": 7, "op": "ping"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"pong": 7})))
        .mount(&server)
        .await;

    let svc = fixed("arona", &[server.uri().as_str()]);
    let out: serde_json::Value = svc
        .call(
            plana_sibling::Method::POST,
            "/api/rpc",
            &json!({"id": 7, "op": "ping"}),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(out, json!({"pong": 7}));
    // Belt and braces: the header on the wire is exactly the JSON label.
    let reqs = server.received_requests().await.unwrap();
    assert_eq!(reqs.len(), 1);
    assert_eq!(
        reqs[0].headers.get("content-type").unwrap(),
        "application/json"
    );
}

#[tokio::test]
async fn bodyless_requests_carry_no_content_type() {
    // The JSON label is scoped to bodies: GET/HEAD must stay header-free
    // (endpoint caches/CDNs key on content-type and a bogus label would
    // poison them).
    let server = MockServer::start().await;
    ok_mock(json!({"ok": true})).mount(&server).await;

    let svc = fixed("arona", &[server.uri().as_str()]);
    let _: serde_json::Value = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .send()
        .await
        .unwrap();
    let reqs = server.received_requests().await.unwrap();
    assert_eq!(reqs.len(), 1);
    assert!(reqs[0].headers.get("content-type").is_none());
}

#[tokio::test]
async fn send_bytes_returns_raw_body() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200).set_body_string("RAW-01"))
        .mount(&server)
        .await;

    let svc = fixed("arona", &[server.uri().as_str()]);
    let bytes = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .send_bytes()
        .await
        .unwrap();
    assert_eq!(bytes, b"RAW-01");
}

// ---------------------------------------------------------------------------
// tracing span
// ---------------------------------------------------------------------------

struct BufWriter(Arc<Mutex<Vec<u8>>>);

impl io::Write for BufWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn sibling_call_span_carries_identity_endpoint_result_elapsed() {
    let buffer: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = buffer.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(move || BufWriter(sink.clone()))
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    let server = MockServer::start().await;
    ok_mock(json!({"ok": true})).mount(&server).await;

    let svc = fixed("span-sib", &[server.uri().as_str()]);
    let out: serde_json::Value = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .send()
        .await
        .unwrap();
    assert_eq!(out, json!({"ok": true}));

    let captured = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
    // tracing-subscriber fmt quotes string fields.
    assert!(captured.contains("sibling_call"), "{captured}");
    assert!(captured.contains("sibling=\"span-sib\""), "{captured}");
    assert!(
        captured.contains(&format!("endpoint={}", server.uri())),
        "{captured}"
    );
    assert!(captured.contains("method=GET"), "{captured}");
    assert!(captured.contains("path=\"/api/x\""), "{captured}");
    assert!(captured.contains("result=\"ok\""), "{captured}");
    assert!(captured.contains("elapsed_ms="), "{captured}");
}

#[tokio::test]
async fn sibling_call_span_records_failure_result() {
    let buffer: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = buffer.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(move || BufWriter(sink.clone()))
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    let server = MockServer::start().await;
    status_mock(500, "boom").mount(&server).await;

    let svc = fixed("span-sib-fail", &[server.uri().as_str()]);
    let err = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .send::<serde_json::Value>()
        .await
        .unwrap_err();
    assert!(
        matches!(err, SiblingError::AllEndpointsDown { .. }),
        "{err:?}"
    );

    let captured = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
    assert!(captured.contains("sibling=\"span-sib-fail\""), "{captured}");
    assert!(captured.contains("result=\"HTTP 500\""), "{captured}");
}

// ---------------------------------------------------------------------------
// BearerProvider (dynamic credential) & call_no_body
// ---------------------------------------------------------------------------

struct StaticBearer(Option<SecretString>);

impl plana_sibling::BearerProvider for StaticBearer {
    fn bearer(&self) -> Option<SecretString> {
        self.0.clone()
    }
}

#[tokio::test]
async fn bearer_provider_resolves_per_attempt() {
    let server = MockServer::start().await;
    Mock::given(header("authorization", "Bearer token-one"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"n": 1})))
        .mount(&server)
        .await;
    Mock::given(header("authorization", "Bearer token-two"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"n": 2})))
        .mount(&server)
        .await;

    // A provider whose answer flips between calls (rotation without restart).
    struct Flipping(std::sync::Mutex<Option<SecretString>>);
    impl plana_sibling::BearerProvider for Flipping {
        fn bearer(&self) -> Option<SecretString> {
            self.0.lock().unwrap().clone()
        }
    }
    let flipping = Arc::new(Flipping(std::sync::Mutex::new(Some(SecretString::new(
        "token-one".to_string(),
    )))));
    let provider: Arc<dyn plana_sibling::BearerProvider> = flipping.clone();

    let svc = SiblingService::new("arona")
        .endpoint_fixed(server.uri())
        .bearer_provider(Arc::clone(&provider))
        .build()
        .unwrap();
    let out: serde_json::Value = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .send()
        .await
        .unwrap();
    assert_eq!(out, json!({"n": 1}));

    // Rotate: the very next attempt carries the new token.
    *flipping.0.lock().unwrap() = Some(SecretString::new("token-two".to_string()));
    let out: serde_json::Value = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .send()
        .await
        .unwrap();
    assert_eq!(out, json!({"n": 2}));
}

#[tokio::test]
async fn bearer_provider_none_sends_no_authorization_header() {
    let server = MockServer::start().await;
    ok_mock(json!({"ok": true})).mount(&server).await;

    let provider = Arc::new(StaticBearer(None));
    let svc = SiblingService::new("arona")
        .endpoint_fixed(server.uri())
        .bearer_provider(provider)
        .build()
        .unwrap();
    let out: serde_json::Value = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .send()
        .await
        .unwrap();
    assert_eq!(out, json!({"ok": true}));
    let reqs = server.received_requests().await.unwrap();
    assert!(reqs[0].headers.get("authorization").is_none());
}

#[tokio::test]
async fn bearer_secret_takes_precedence_over_provider() {
    let server = MockServer::start().await;
    Mock::given(header("authorization", "Bearer direct-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .mount(&server)
        .await;

    let provider = Arc::new(StaticBearer(Some(SecretString::new(
        "provider-secret".to_string(),
    ))));
    let svc = SiblingService::new("arona")
        .endpoint_fixed(server.uri())
        .bearer_provider(provider)
        .bearer_secret(SecretString::new("direct-secret".to_string()))
        .build()
        .unwrap();
    let out: serde_json::Value = svc
        .call(plana_sibling::Method::GET, "/api/x", &())
        .send()
        .await
        .unwrap();
    assert_eq!(out, json!({"ok": true}));
}

#[tokio::test]
async fn call_no_body_sends_no_payload_at_all() {
    let server = MockServer::start().await;
    ok_mock(json!({"ok": true})).mount(&server).await;

    let svc = fixed("arona", &[server.uri().as_str()]);
    let out: serde_json::Value = svc
        .call_no_body(plana_sibling::Method::POST, "/api/rotate")
        .send()
        .await
        .unwrap();
    assert_eq!(out, json!({"ok": true}));
    let reqs = server.received_requests().await.unwrap();
    assert_eq!(reqs.len(), 1);
    // A `call(.., &())` would wire the JSON literal `null`; a bodyless POST
    // must wire neither body nor a JSON content-type.
    assert!(
        reqs[0].body.is_empty(),
        "bodyless POST must not carry a payload: {:?}",
        String::from_utf8_lossy(&reqs[0].body)
    );
    assert!(reqs[0].headers.get("content-type").is_none());
}

#[tokio::test]
async fn query_pair_lands_on_the_wire() {
    use wiremock::matchers::query_param;
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/prs"))
        .and(query_param("state", "open"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"prs": []})))
        .mount(&server)
        .await;

    let svc = fixed("noa", &[server.uri().as_str()]);
    let out: serde_json::Value = svc
        .call(plana_sibling::Method::GET, "/api/v1/prs", &())
        .query_pair("state", "open")
        .send()
        .await
        .unwrap();
    assert_eq!(out, json!({"prs": []}));
}
