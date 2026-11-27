//! The handoff surface as a DOWNSTREAM uses it: a domain that knows its
//! failure is browser-facing, decides before a response exists, and must
//! still answer exactly what the layer would have answered.
//!
//! Everything here is driven through the crate's public API only — this file
//! sits outside the `plana_error_landing` crate, so a name that is not
//! exported does not compile here. That is the point: the wrapper below is
//! the `chest-app` shape (navigation judgment → bare-text predicate →
//! landing response), and it carries no rule of its own.
//!
//! MUT:mCL-narrowing-a-name-breaks-this-file — narrowing any widened name back
//! to `pub(crate)`/private fails THIS file's compile (not an assertion), which
//! is what makes the sentence above load-bearing rather than decorative.

use axum::body::Body;
use axum::http::header;
use axum::http::{HeaderMap, HeaderValue, Method, Request, StatusCode};
use axum::response::{Html, Response};
use axum::routing::get;
use axum::{Json, Router};
use http_body_util::BodyExt;
use plana_error_landing::{
    ErrorLanding, MAX_ERROR_BODY_BYTES, ServerErrorLanding, is_bare_text, is_browser_navigation,
};
use serde_json::json;
use tower::ServiceExt;

const SHELL: &str = "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>app</title>\n</head>\n<body>\n<div id=\"app\"></div>\n</body>\n</html>\n";

const OAUTH_MESSAGE: &str = "identity provider unreachable: answered 400 Bad Request";

fn navigation() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
    headers.insert(
        header::ACCEPT,
        HeaderValue::from_static("text/html,*/*;q=0.8"),
    );
    headers
}

fn api_call() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
    headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));
    headers
}

/// One header of every class in `handoff.rs`'s replaced-body set, carrying a
/// value that is TRUE of the wire JSON and FALSE of the landing document —
/// so a member the assembly forgot to drop shows up here as a stale value
/// rather than as a coincidentally-correct one.
///
/// Two members are asserted differently from the rest. `content-type` and
/// `content-length` are REPLACED rather than dropped by the assembly (it sets
/// its own type, and `Body::from` regenerates the length from the landing
/// document), so those two are asserted "not the stale value"; every other
/// member is asserted absent outright. A surviving `content-length: 2` would
/// truncate the client at two bytes, which is why its wire value has to be
/// impossible for the landing.
const REPLACED_BODY_METADATA: [(&str, &str); 13] = [
    ("content-type", "application/json"),
    ("content-length", "2"),
    ("content-encoding", "identity"),
    ("transfer-encoding", "chunked"),
    ("etag", "\"abc\""),
    ("last-modified", "Wed, 21 Oct 2015 07:28:00 GMT"),
    ("content-range", "bytes 0-1/2"),
    ("expires", "Wed, 21 Oct 2015 07:28:00 GMT"),
    ("age", "120"),
    ("pragma", "no-cache"),
    ("connection", "keep-alive"),
    ("te", "trailers"),
    ("trailer", "X-Checksum"),
];

/// What the service said to the CLIENT: true of the landing too, so it must
/// survive the replacement.
const CLIENT_FACING: [(&str, &str); 2] = [
    ("set-cookie", "session=; Max-Age=0; Path=/"),
    ("retry-after", "3"),
];

/// The JSON answer the service would have sent to an API caller, carrying one
/// header of every class — the response a domain-declared landing lane holds
/// in hand.
fn wire_response(status: StatusCode, body: &serde_json::Value) -> Response<Body> {
    let mut builder = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json");
    for (name, value) in REPLACED_BODY_METADATA.iter().chain(CLIENT_FACING.iter()) {
        builder = builder.header(*name, *value);
    }
    builder
        .body(Body::from(serde_json::to_vec(body).expect("json")))
        .expect("static response")
}

/// The downstream wrapper this surface exists for: a domain that decides a
/// failure is browser-facing before any response exists, composes the answer
/// the service would have sent, and answers the landing out of the same
/// shell — using only what `plana_error_landing` exports.
async fn domain_declared(
    method: &Method,
    request_headers: &HeaderMap,
    wire: Response<Body>,
    shell: Option<&str>,
) -> Response<Body> {
    // 1. Navigation judgment — the REQUEST half of the family's one rule.
    let Some(shell) = shell.filter(|_| is_browser_navigation(method, request_headers)) else {
        return wire;
    };
    // 2. The bare-text predicate — the RESPONSE half, on the response it
    //    actually composed. A caller that skips it answers a curated HTML
    //    error page with the landing.
    if !is_bare_text(&wire) {
        return wire;
    }
    let (parts, body) = wire.into_parts();
    let bytes = axum::body::to_bytes(body, MAX_ERROR_BODY_BYTES)
        .await
        .expect("an error body fits the cap");
    let landing = ServerErrorLanding::from_body(
        parts.status,
        parts.headers.get(header::CONTENT_TYPE),
        &bytes,
    );
    // 3. The landing response, out of the parts it replaces.
    landing.into_response(parts, shell)
}

async fn body_text(response: Response<Body>) -> String {
    String::from_utf8(
        response
            .into_body()
            .collect()
            .await
            .expect("infallible body")
            .to_bytes()
            .to_vec(),
    )
    .expect("utf-8 document")
}

#[tokio::test]
async fn a_domain_declared_landing_answers_what_the_layer_would() {
    let wire = wire_response(StatusCode::BAD_GATEWAY, &json!({ "error": OAUTH_MESSAGE }));
    let response = domain_declared(&Method::GET, &navigation(), wire, Some(SHELL)).await;

    // The status the service answered with, not one the wrapper invents.
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    // The landing's own framing.
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/html; charset=utf-8"
    );
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(
        response.headers()[header::X_CONTENT_TYPE_OPTIONS],
        "nosniff"
    );
    assert_eq!(response.headers()[header::VARY], "Sec-Fetch-Dest, Accept");
    // What the service said to the CLIENT survives the landing …
    for (name, value) in CLIENT_FACING {
        assert_eq!(
            response.headers().get(name).and_then(|v| v.to_str().ok()),
            Some(value),
            "{name} must survive the landing"
        );
    }
    // … and nothing that described the replaced body does. Every stale value
    // is asserted GONE. `content-type` and `content-length` are the two the
    // landing REPLACES rather than drops (the type assertion above is one; the
    // length is regenerated from the landing document by `Body::from`), so they
    // are asserted as "not the stale value" — a surviving `content-length: 2`
    // would truncate the client at two bytes.
    for (name, stale) in REPLACED_BODY_METADATA {
        let seen = response
            .headers()
            .get(name)
            .map(|value| value.to_str().unwrap_or_default());
        assert!(
            seen != Some(stale),
            "{name} survived the landing carrying the replaced body's value {stale:?} \
             (the landing answers its own document, not that body)"
        );
        if name == "content-length" || name == "content-type" {
            continue;
        }
        assert_eq!(
            seen, None,
            "{name} described the body the landing replaced and must not survive (stale {stale})"
        );
    }

    let document = body_text(response).await;
    // The service's own shell, with the island seated inside <head>, and the
    // error the service answered with.
    assert!(document.starts_with("<!doctype html>\n<html lang=\"en\">\n<head>\n"));
    let island_at = document
        .find("<script type=\"application/json\" id=\"hk-server-error\">")
        .expect("island");
    assert!(
        island_at < document.find("</head>").expect("head"),
        "the island is seated inside <head>"
    );
    assert!(document.contains(r#""status":502"#));
    assert!(document.contains(OAUTH_MESSAGE));
    assert!(document.contains("<div id=\"app\"></div>"));
}

#[tokio::test]
async fn the_wrapper_and_the_layer_answer_one_failure_one_way() {
    // The whole claim of this surface: a downstream that applies the rules
    // itself lands on the SAME answer the layer produces. Compare the two
    // byte for byte on the same shell and the same wire JSON.
    let wire_json = json!({ "error": "not_found", "message": "no such file" });
    let declared = domain_declared(
        &Method::GET,
        &navigation(),
        wire_response(StatusCode::NOT_FOUND, &wire_json),
        Some(SHELL),
    )
    .await;

    let routed = Router::new()
        .route(
            "/missing",
            get(move || {
                let wire_json = wire_json.clone();
                async move { (StatusCode::NOT_FOUND, Json(wire_json)) }
            }),
        )
        .layer(ErrorLanding::new(SHELL).layer());
    let layered = routed
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/missing")
                .header("sec-fetch-dest", "document")
                .header(header::ACCEPT, "text/html,*/*;q=0.8")
                .body(Body::empty())
                .expect("static request"),
        )
        .await
        .expect("infallible router");

    assert_eq!(declared.status(), layered.status());
    for name in [
        header::CONTENT_TYPE,
        header::CACHE_CONTROL,
        header::X_CONTENT_TYPE_OPTIONS,
        header::VARY,
    ] {
        assert_eq!(
            declared.headers().get(&name),
            layered.headers().get(&name),
            "{}",
            name.as_str()
        );
    }
    assert_eq!(
        body_text(declared).await,
        body_text(layered).await,
        "one failure, one document"
    );
}

#[tokio::test]
async fn a_fetch_caller_never_reaches_the_assembly() {
    let wire = wire_response(StatusCode::BAD_GATEWAY, &json!({ "error": OAUTH_MESSAGE }));
    let response = domain_declared(&Method::GET, &api_call(), wire, Some(SHELL)).await;
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
    assert_eq!(
        body_text(response).await,
        format!(r#"{{"error":"{OAUTH_MESSAGE}"}}"#)
    );
}

#[tokio::test]
async fn a_head_request_never_reaches_the_assembly() {
    let wire = wire_response(StatusCode::BAD_GATEWAY, &json!({ "error": OAUTH_MESSAGE }));
    let response = domain_declared(&Method::HEAD, &navigation(), wire, Some(SHELL)).await;
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
}

#[tokio::test]
async fn a_shell_less_build_never_reaches_the_assembly() {
    let wire = wire_response(StatusCode::BAD_GATEWAY, &json!({ "error": OAUTH_MESSAGE }));
    let response = domain_declared(&Method::GET, &navigation(), wire, None).await;
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
}

#[tokio::test]
async fn the_bare_text_predicate_declines_what_a_caller_must_not_replace() {
    let of = |status: StatusCode, headers: &[(&str, &str)]| {
        let mut builder = Response::builder().status(status);
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        builder.body(Body::empty()).expect("static response")
    };
    // An error whose body a browser would have shown as bare text.
    assert!(is_bare_text(&of(StatusCode::NOT_FOUND, &[])));
    assert!(is_bare_text(&of(
        StatusCode::BAD_GATEWAY,
        &[("content-type", "application/json")]
    )));
    assert!(is_bare_text(&of(
        StatusCode::FORBIDDEN,
        &[("content-type", "text/plain; charset=utf-8")]
    )));
    // Not an error at all.
    assert!(!is_bare_text(&of(
        StatusCode::OK,
        &[("content-type", "application/json")]
    )));
    // A body the visitor was never shown as text, an encoded one, or an
    // attachment: each is somebody else's answer. MUT:m3-predicate-accepts-html
    // turns `BodyKind::Other` into `true` and reds the `text/html` row here.
    assert!(!is_bare_text(&of(
        StatusCode::BAD_REQUEST,
        &[("content-type", "text/html; charset=utf-8")]
    )));
    assert!(!is_bare_text(&of(
        StatusCode::BAD_GATEWAY,
        &[
            ("content-type", "application/json"),
            ("content-encoding", "gzip")
        ]
    )));
    assert!(!is_bare_text(&of(
        StatusCode::NOT_FOUND,
        &[
            ("content-type", "application/json"),
            ("content-disposition", "attachment; filename=export.json")
        ]
    )));
}

#[tokio::test]
async fn a_curated_html_error_page_is_answered_as_it_is() {
    let curated = Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from("<p>curated</p>"))
        .expect("static response");
    let response = domain_declared(&Method::GET, &navigation(), curated, Some(SHELL)).await;
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/html; charset=utf-8"
    );
    assert_eq!(body_text(response).await, "<p>curated</p>");
}

#[tokio::test]
async fn empty_parts_carry_a_status_and_nothing_else() {
    let parts = ServerErrorLanding::empty_parts(StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(parts.status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(parts.headers.is_empty(), "nothing to carry over");

    // The assembly adds everything the landing needs regardless.
    let response = ServerErrorLanding::new(StatusCode::SERVICE_UNAVAILABLE, "busy", "try later")
        .into_response(parts, SHELL);
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/html; charset=utf-8"
    );
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(response.headers()[header::VARY], "Sec-Fetch-Dest, Accept");
    let document = body_text(response).await;
    assert!(document.contains(r#""status":503,"code":"busy","message":"try later""#));
}

#[tokio::test]
async fn a_success_body_passes_the_predicate_and_the_caller_still_chooses() {
    // The predicate answers for the BARE-TEXT half only. The `a_curated_html…`
    // test above is its other half (its own status family); this one pins
    // that a 2xx with a JSON body is refused, so a caller cannot land a
    // success just because its body would have been bare text.
    let ok = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{}"))
        .expect("static response");
    assert!(!is_bare_text(&ok));
}

#[test]
fn the_layer_and_the_assembly_exist_side_by_side() {
    // Both halves of the surface are reachable from outside the crate: the
    // layer for a service that mounts it, the handoff assembly for a domain
    // that decides before a body exists.
    let _layer = ErrorLanding::new(SHELL).layer();
    let _disabled = ErrorLanding::disabled();
    let _standalone = ErrorLanding::standalone();
    let _html: Html<&str> = Html(SHELL);
}

#[test]
fn wants_answers_navigation_and_shell_together() {
    // The mount-side predicate: "this request's error would be replaced",
    // which is BOTH halves of the rule. Public so a caller that composes the
    // landing itself asks the same combined question instead of re-deriving
    // "shell present AND navigating" from two separate exports.
    assert!(ErrorLanding::new(SHELL).wants(&Method::GET, &navigation()));
    assert!(!ErrorLanding::new(SHELL).wants(&Method::GET, &api_call()));
    // No shell: nothing is replaced, however the request looks.
    // MUT:mW-wants-drops-the-shell-half reds the assertion right below.
    assert!(!ErrorLanding::disabled().wants(&Method::GET, &navigation()));
    // A landing has a body, so HEAD never qualifies. MUT:m6-service-ignores-wants
    // (the service path skipping the request-side judgment) is pinned by
    // `landing::head_requests_are_untouched`, which reds on this same rule.
    assert!(!ErrorLanding::new(SHELL).wants(&Method::HEAD, &navigation()));
}
