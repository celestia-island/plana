//! The layer over a real axum router: what lands, what passes through.

use axum::body::Body;
use axum::http::header;
use axum::http::{Request, StatusCode};
use axum::response::{Html, Redirect, Response};
use axum::routing::get;
use axum::{Json, Router};
use http_body_util::BodyExt;
use plana_error_landing::{ErrorLanding, SERVER_ERROR_LANDING_ID, SERVER_ERROR_LANDING_MARKER};
use serde_json::json;
use tower::ServiceExt;

const SHELL: &str = "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>app</title>\n</head>\n<body>\n<div id=\"app\"></div>\n<script type=\"module\" src=\"/assets/main.js\"></script>\n</body>\n</html>\n";

const OAUTH_MESSAGE: &str =
    "identity provider unreachable: identity provider answered 400 Bad Request";

fn app(landing: ErrorLanding) -> Router {
    Router::new()
        .route(
            "/json-error",
            get(|| async {
                (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({ "error": OAUTH_MESSAGE })),
                )
            }),
        )
        .route(
            "/auth-error",
            get(|| async {
                (
                    StatusCode::UNAUTHORIZED,
                    [
                        (header::SET_COOKIE, "session=; Max-Age=0; Path=/"),
                        (header::WWW_AUTHENTICATE, "Bearer realm=\"app\""),
                        (header::RETRY_AFTER, "3"),
                    ],
                    Json(json!({ "error": "sign in first" })),
                )
            }),
        )
        .route(
            "/html-error",
            get(|| async { (StatusCode::BAD_REQUEST, Html("<p>curated</p>")) }),
        )
        .route(
            "/validator-error",
            get(|| async {
                (
                    StatusCode::NOT_FOUND,
                    [
                        (header::ETAG, "\"abc\""),
                        (header::LAST_MODIFIED, "Wed, 21 Oct 2015 07:28:00 GMT"),
                        (header::CONTENT_RANGE, "bytes 0-10/11"),
                        (header::EXPIRES, "Wed, 21 Oct 2015 07:28:00 GMT"),
                        (header::SET_COOKIE, "sid=1; Path=/"),
                    ],
                    Json(json!({ "error": "range not satisfiable" })),
                )
            }),
        )
        .route(
            "/inner-compressed",
            get(|| async {
                (
                    StatusCode::NOT_FOUND,
                    [
                        (header::CONTENT_TYPE, "application/json"),
                        (header::CONTENT_ENCODING, " gzip "),
                    ],
                    // Bytes an inner gzip layer could have produced: not UTF-8.
                    Body::from(vec![0x1f, 0x8b, 0x00, b'n', b'o', b't']),
                )
            }),
        )
        .route("/ok", get(|| async { Html(SHELL) }))
        .route("/redirect", get(|| async { Redirect::to("/login") }))
        .route("/bare", get(|| async { StatusCode::NOT_FOUND }))
        .route(
            "/huge",
            get(|| async {
                (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({ "error": "x".repeat(200_000) })),
                )
            }),
        )
        .route(
            "/download",
            get(|| async {
                (
                    StatusCode::NOT_FOUND,
                    [(
                        header::CONTENT_DISPOSITION,
                        "attachment; filename=\"export.json\"",
                    )],
                    Json(json!({ "error": "gone" })),
                )
            }),
        )
        .route(
            "/stream",
            get(|| async {
                Response::new(Body::from_stream(futures::stream::iter([
                    Ok::<_, std::convert::Infallible>(axum::body::Bytes::from_static(
                        &[0x3a; 70_000],
                    )),
                    Ok::<_, std::convert::Infallible>(axum::body::Bytes::from_static(b"tail")),
                ])))
            }),
        )
        .layer(landing.layer())
}

async fn send(
    app: Router,
    method: &str,
    uri: &str,
    headers: &[(&'static str, &str)],
) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
    let mut request = Request::builder().method(method).uri(uri);
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let response = app
        .oneshot(request.body(Body::empty()).expect("static request"))
        .await
        .expect("infallible router");
    let (parts, body) = response.into_parts();
    let bytes = body.collect().await.expect("infallible body").to_bytes();
    (parts.status, parts.headers, bytes.to_vec())
}

async fn navigation(app: Router, uri: &str) -> (StatusCode, axum::http::HeaderMap, String) {
    let (status, headers, body) = send(
        app,
        "GET",
        uri,
        &[
            ("sec-fetch-dest", "document"),
            ("accept", "text/html,*/*;q=0.8"),
        ],
    )
    .await;
    (
        status,
        headers,
        String::from_utf8(body).expect("utf-8 document"),
    )
}

#[tokio::test]
async fn a_tab_navigation_to_a_json_error_lands_on_the_shell() {
    let (status, headers, document) =
        navigation(app(ErrorLanding::new(SHELL)), "/json-error").await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert_eq!(headers[header::CONTENT_TYPE], "text/html; charset=utf-8");
    assert_eq!(headers[header::CACHE_CONTROL], "no-store");
    assert_eq!(headers[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
    assert_eq!(headers[header::VARY], "Sec-Fetch-Dest, Accept");
    assert_eq!(
        headers.get(header::CONTENT_ENCODING),
        None,
        "nothing was compressed here"
    );

    // The document is the shell the service serves on `/`, island and all.
    assert!(document.starts_with("<!doctype html>\n<html lang=\"en\">\n<head>\n"));
    let island_at = document
        .find("<script type=\"application/json\" id=\"hk-server-error\">")
        .expect("island");
    let head_close = document.find("</head>").expect("shell head");
    assert!(island_at < head_close, "the island is seated inside <head>");
    let rest = &document[island_at..];
    let island = &rest[..rest.find("</script>").expect("island end") + "</script>".len()];
    let payload: serde_json::Value = serde_json::from_str(
        &island["<script type=\"application/json\" id=\"hk-server-error\">".len()
            ..island.len() - "</script>".len()],
    )
    .expect("the island parses as JSON");
    assert_eq!(
        payload,
        json!({ "v": 1, "status": 502, "code": "", "message": OAUTH_MESSAGE, "raw": { "error": OAUTH_MESSAGE } })
    );
    assert!(document.contains("<div id=\"app\"></div>"));
}

#[tokio::test]
async fn a_fetch_caller_keeps_the_json_byte_for_byte() {
    let wire = format!(r#"{{"error":"{OAUTH_MESSAGE}"}}"#);
    for headers in [
        vec![("sec-fetch-dest", "empty"), ("accept", "application/json")],
        vec![("sec-fetch-dest", "empty")],
        vec![("accept", "*/*")],
        vec![("accept", "application/json, text/plain, */*")],
        vec![],
    ] {
        let (status, response_headers, body) = send(
            app(ErrorLanding::new(SHELL)),
            "GET",
            "/json-error",
            &headers,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_GATEWAY, "headers {headers:?}");
        assert_eq!(
            response_headers[header::CONTENT_TYPE],
            "application/json",
            "headers {headers:?}"
        );
        assert_eq!(
            String::from_utf8(body).expect("utf-8"),
            wire,
            "headers {headers:?}"
        );
    }
}

#[tokio::test]
async fn a_client_without_fetch_metadata_is_judged_by_its_accept() {
    let (_, headers, document) = navigation_with(
        app(ErrorLanding::new(SHELL)),
        &[("accept", "text/html,application/xhtml+xml,*/*;q=0.8")],
    )
    .await;
    assert_eq!(headers[header::CONTENT_TYPE], "text/html; charset=utf-8");
    assert!(document.contains("id=\"hk-server-error\""));
}

async fn navigation_with(
    app: Router,
    headers: &[(&'static str, &str)],
) -> (StatusCode, axum::http::HeaderMap, String) {
    let (status, response_headers, body) = send(app, "GET", "/json-error", headers).await;
    (
        status,
        response_headers,
        String::from_utf8(body).expect("utf-8 document"),
    )
}

#[tokio::test]
async fn an_html_error_page_a_service_already_renders_is_left_alone() {
    let (status, headers, body) = send(
        app(ErrorLanding::new(SHELL)),
        "GET",
        "/html-error",
        &[("sec-fetch-dest", "document")],
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(headers[header::CONTENT_TYPE], "text/html; charset=utf-8");
    assert_eq!(String::from_utf8(body).expect("utf-8"), "<p>curated</p>");
}

#[tokio::test]
async fn success_redirects_streams_and_attachments_pass_through() {
    let app = app(ErrorLanding::new(SHELL));
    let navigation_headers = [("sec-fetch-dest", "document")];

    let (status, _, body) = send(app.clone(), "GET", "/ok", &navigation_headers).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        String::from_utf8(body).expect("utf-8"),
        SHELL,
        "the SPA shell carries no island"
    );

    let (status, headers, _) = send(app.clone(), "GET", "/redirect", &navigation_headers).await;
    // `Redirect::to` answers 303 See Other; a 3xx is followed, never landed.
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert_eq!(headers[header::LOCATION], "/login");

    let (status, _, body) = send(app.clone(), "GET", "/stream", &navigation_headers).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body.len(),
        70_004,
        "the stream was not buffered, replaced or truncated"
    );

    let (status, headers, body) = send(app.clone(), "GET", "/download", &navigation_headers).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(headers[header::CONTENT_TYPE], "application/json");
    assert_eq!(
        String::from_utf8(body).expect("utf-8"),
        r#"{"error":"gone"}"#
    );
}

#[tokio::test]
async fn the_landing_keeps_the_services_other_headers() {
    let (status, headers, document) =
        navigation(app(ErrorLanding::new(SHELL)), "/auth-error").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(headers[header::SET_COOKIE], "session=; Max-Age=0; Path=/");
    assert_eq!(headers[header::WWW_AUTHENTICATE], "Bearer realm=\"app\"");
    assert_eq!(headers[header::RETRY_AFTER], "3");
    assert_eq!(
        headers[header::CONTENT_LENGTH].to_str().unwrap(),
        document.len().to_string()
    );
    assert!(document.contains(r#""status":401"#));
    assert!(document.contains(r#""message":"sign in first""#));
}

#[tokio::test]
async fn validators_of_the_replaced_body_do_not_survive() {
    let (status, headers, document) =
        navigation(app(ErrorLanding::new(SHELL)), "/validator-error").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    // They described bytes that no longer exist; a revalidating or
    // range-aware client must never act on them against the landing.
    for gone in &[
        header::ETAG,
        header::LAST_MODIFIED,
        header::CONTENT_RANGE,
        header::EXPIRES,
    ] {
        assert_eq!(headers.get(gone), None, "{} survived", gone.as_str());
    }
    // What the service said to the CLIENT still does.
    assert_eq!(headers[header::SET_COOKIE], "sid=1; Path=/");
    assert_eq!(
        headers[header::CONTENT_LENGTH].to_str().unwrap(),
        document.len().to_string()
    );
}

#[tokio::test]
async fn a_body_an_inner_layer_already_encoded_is_left_alone() {
    let (status, headers, body) = send(
        app(ErrorLanding::new(SHELL)),
        "GET",
        "/inner-compressed",
        &[("sec-fetch-dest", "document")],
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    // The layer cannot read an encoded body, so it does not pretend to:
    // bytes and the header that explains them pass through together.
    assert_eq!(headers[header::CONTENT_ENCODING], " gzip ");
    assert_eq!(headers[header::CONTENT_TYPE], "application/json");
    assert_eq!(body, b"\x1f\x8b\x00not");
}

#[tokio::test]
async fn a_bare_status_lands_with_its_canonical_reason() {
    let (status, _, document) = navigation(app(ErrorLanding::new(SHELL)), "/bare").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(
        document.contains(r#""status":404,"code":"","message":"Not Found"}"#),
        "no raw when the service sent none"
    );
}

#[tokio::test]
async fn an_oversized_error_body_lands_without_its_content() {
    let (status, _, document) = navigation(app(ErrorLanding::new(SHELL)), "/huge").await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert!(
        document.len() < 8_000,
        "the 200 KB error body did not ride along: {} bytes",
        document.len()
    );
    assert!(document.contains(r#""status":502,"code":"","message":"Bad Gateway"}"#));
}

#[tokio::test]
async fn head_requests_are_untouched() {
    let (status, headers, _) = send(
        app(ErrorLanding::new(SHELL)),
        "HEAD",
        "/json-error",
        &[("sec-fetch-dest", "document")],
    )
    .await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert_eq!(headers[header::CONTENT_TYPE], "application/json");
}

#[tokio::test]
async fn without_a_shell_nothing_is_replaced() {
    let (status, headers, body) = send(
        app(ErrorLanding::disabled()),
        "GET",
        "/json-error",
        &[("sec-fetch-dest", "document")],
    )
    .await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert_eq!(headers[header::CONTENT_TYPE], "application/json");
    assert_eq!(
        String::from_utf8(body).expect("utf-8"),
        format!(r#"{{"error":"{OAUTH_MESSAGE}"}}"#)
    );
}

#[tokio::test]
async fn a_shell_with_a_marker_seats_the_island_there() {
    let shell = "<!doctype html><html><head><title>t</title><!--hk-server-error--></head><body>shell</body></html>";
    let (_, _, document) = navigation(app(ErrorLanding::new(shell)), "/json-error").await;
    assert!(
        document
            .contains("<title>t</title><script type=\"application/json\" id=\"hk-server-error\">"),
        "{document}"
    );
    assert!(
        !document.contains(SERVER_ERROR_LANDING_MARKER),
        "the marker comment is replaced, not kept"
    );
}

#[test]
fn the_public_constants_name_the_island_hikari_reads() {
    assert_eq!(SERVER_ERROR_LANDING_ID, "hk-server-error");
    assert_eq!(SERVER_ERROR_LANDING_MARKER, "<!--hk-server-error-->");
}
