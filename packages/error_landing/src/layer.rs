//! The axum layer: a failed navigation in, the landing out.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use axum::body::Body;
use axum::http::header;
use axum::http::{HeaderMap, Request, Response};
use http_body_util::{BodyExt, Limited};
use tower_layer::Layer;
use tower_service::Service;

use crate::detect::{BodyKind, is_browser_navigation};
use crate::handoff::ServerErrorLanding;

/// The largest error body read into the landing's `raw`. A body past it is
/// replaced WITHOUT its content (the landing still goes up, with the status
/// reason as its copy): an error body this size is not something a visitor
/// reads off a card, and an unbounded read would hand any route that
/// answers a huge error a way to allocate without limit.
pub const MAX_ERROR_BODY_BYTES: usize = 64 * 1024;

/// The standalone landing document [`ErrorLanding::standalone`] answers
/// with: hikari's `packages/vue/src/errorLandingStandalone/error-landing.html`,
/// vendored byte for byte (`res/error-landing.html`). Point every DOT of this
/// list at the first character of one line of that file — no comment line, no
/// blank line — and `cargo fmt --all -- --check` leaves the file alone; a
/// plain call here would be re-wrapped into mangled lines of its own.
#[rustfmt::skip]
const STANDALONE_DOCUMENT: &str = include_str!("../res/error-landing.html");

/// The landing's two inputs: the SPA shell document a failed navigation is
/// answered with, and (implicitly) that there is one at all.
///
/// Built with the same bytes the service serves on `/` — the embedded
/// `include_dir!` tree, the `rust-embed` asset, the file read at startup.
/// [`ErrorLanding::disabled`] is the headless posture: no frontend, so
/// every response passes through untouched.
#[derive(Clone, Debug)]
pub struct ErrorLanding {
    shell: Option<Arc<str>>,
}

impl ErrorLanding {
    /// Serve landings out of this shell document.
    #[must_use]
    pub fn new(shell: impl Into<Arc<str>>) -> Self {
        Self {
            shell: Some(shell.into()),
        }
    }

    /// Pass every response through: for a build without a frontend to embed.
    #[must_use]
    pub fn disabled() -> Self {
        Self { shell: None }
    }

    /// Serve landings out of the family's standalone landing document.
    ///
    /// For a service with no SPA shell to hand in: the failed navigation is
    /// answered with hikari's prebuilt `error-landing.html` — the family
    /// card, its locales and the island reader, in one file — instead of
    /// the service's own document. Every rule above still decides what
    /// qualifies; only the shell changes.
    ///
    /// The document carries an EXECUTABLE inline script, so it is for a
    /// service that sets no Content-Security-Policy of its own: a
    /// `script-src` an operator added would refuse that script and leave
    /// the visitor on a blank card. A service with its own CSP keeps its
    /// SPA shell. [`Self::disabled`] is unchanged — a build that should
    /// replace nothing keeps passing every response through.
    #[must_use]
    pub fn standalone() -> Self {
        Self::new(STANDALONE_DOCUMENT)
    }

    /// The layer over every route whose errors should land. Innermost, so it
    /// sees the response a handler produced before outer middleware
    /// (compression, security headers, CORS) touches it.
    #[must_use]
    pub fn layer(self) -> ErrorLandingLayer {
        ErrorLandingLayer {
            landing: Arc::new(self),
        }
    }

    /// Whether this request's error response would be replaced. Decided
    /// before the handler runs, so a non-navigation request pays nothing
    /// but one header read.
    pub fn wants(&self, method: &axum::http::Method, headers: &HeaderMap) -> bool {
        self.shell.is_some() && is_browser_navigation(method, headers)
    }

    /// Replace the response when it is an error a browser would have shown
    /// as bare text; hand back anything else exactly as it was.
    ///
    /// TEST-ONLY, and deliberately not public. In a real build the decision
    /// is [`ErrorLandingService::call`]'s alone: it asks the REQUEST once
    /// (navigation) and then this response-side predicate, in that order,
    /// with no second copy of the rule to drift. A downstream that reached
    /// for a `take_over`-like entry point would be re-deciding the half it
    /// cannot see; a caller that must decide before a body exists composes
    /// the landing out of [`crate::is_browser_navigation`],
    /// [`crate::is_bare_text`] and [`ServerErrorLanding::into_response`].
    #[cfg(test)]
    async fn take_over(self: Arc<Self>, response: Response<Body>) -> Response<Body> {
        let Some(shell) = self.shell.clone() else {
            return response;
        };
        if !is_bare_text(&response) {
            return response;
        }
        let (parts, body) = response.into_parts();
        // Too large to show: the landing goes up without the body, never
        // a half-read one.
        let bytes = Limited::new(body, MAX_ERROR_BODY_BYTES)
            .collect()
            .await
            .map(|collected| collected.to_bytes())
            .unwrap_or_default();
        ServerErrorLanding::from_body(
            parts.status,
            parts.headers.get(header::CONTENT_TYPE),
            &bytes,
        )
        .into_response(parts, &shell)
    }
}

/// Whether a response is one the landing may replace: an error status whose
/// body is JSON, plain text or absent, not compressed (the layer cannot read
/// what an inner layer already encoded), not an attachment.
///
/// The second half of the family's one landing rule — the first half is
/// [`is_browser_navigation`] on the REQUEST. Public because a caller that
/// assembles its own landing response (see
/// [`ServerErrorLanding::into_response`]) has to apply the same gate to the
/// response it holds; a restated copy in a downstream is a second rule that
/// drifts the day one side changes.
#[must_use]
pub fn is_bare_text(response: &Response<Body>) -> bool {
    let headers = &response.headers();
    if !(response.status().is_client_error() || response.status().is_server_error()) {
        return false;
    }
    let compressed = headers.get(header::CONTENT_ENCODING).is_some_and(|value| {
        value
            .to_str()
            .unwrap_or_default()
            .split(',')
            .any(|coding| !coding.trim().eq_ignore_ascii_case("identity"))
    });
    if compressed {
        return false;
    }
    if headers
        .get(header::CONTENT_DISPOSITION)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.to_ascii_lowercase().contains("attachment"))
    {
        return false;
    }
    match headers.get(header::CONTENT_TYPE).map(BodyKind::of) {
        // Absent and empty labels are the same thing: axum's bare 404/405.
        Some(BodyKind::Json | BodyKind::Text | BodyKind::None) | None => true,
        Some(BodyKind::Other) => false,
    }
}

/// [`Layer`] produced by [`ErrorLanding::layer`].
#[derive(Clone, Debug)]
pub struct ErrorLandingLayer {
    landing: Arc<ErrorLanding>,
}

impl<S> Layer<S> for ErrorLandingLayer {
    type Service = ErrorLandingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ErrorLandingService {
            inner,
            landing: Arc::clone(&self.landing),
        }
    }
}

/// The wrapped service: runs the inner one, then lets [`ErrorLanding`] take
/// over an error response for a navigation. `Clone` because a router clones
/// its service per request; the landing itself is shared behind the `Arc`.
#[derive(Clone)]
pub struct ErrorLandingService<S> {
    inner: S,
    landing: Arc<ErrorLanding>,
}

impl<S> Service<Request<Body>> for ErrorLandingService<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Send + 'static,
    S::Error: Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response<Body>, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        // Decided here, off the request and before the handler runs, so the
        // response side never has to guess whether anyone was navigating.
        let wants = self.landing.wants(req.method(), req.headers());
        let landing = Arc::clone(&self.landing);
        let future = self.inner.call(req);
        Box::pin(async move {
            let response = future.await?;
            if !wants || !is_bare_text(&response) {
                return Ok(response);
            }
            let Some(shell) = landing.shell.clone() else {
                return Ok(response);
            };
            let (parts, body) = response.into_parts();
            // Too large to show: the landing goes up without the body, never
            // a half-read one.
            let bytes = Limited::new(body, MAX_ERROR_BODY_BYTES)
                .collect()
                .await
                .map(|collected| collected.to_bytes())
                .unwrap_or_default();
            Ok(ServerErrorLanding::from_body(
                parts.status,
                parts.headers.get(header::CONTENT_TYPE),
                &bytes,
            )
            .into_response(parts, &shell))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handoff::inject_island;
    use axum::http::{HeaderValue, Method};

    fn response(
        status: axum::http::StatusCode,
        headers: &[(&'static str, &str)],
    ) -> Response<Body> {
        let mut builder = Response::builder().status(status);
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        builder.body(Body::empty()).expect("static response")
    }

    #[test]
    fn only_error_statuses_with_bare_text_bodies_are_replaced() {
        assert!(is_bare_text(&response(
            axum::http::StatusCode::NOT_FOUND,
            &[]
        )));
        assert!(is_bare_text(&response(
            axum::http::StatusCode::BAD_GATEWAY,
            &[("content-type", "application/json")]
        )));
        assert!(is_bare_text(&response(
            axum::http::StatusCode::FORBIDDEN,
            &[("content-type", "text/plain; charset=utf-8")]
        )));
        // Not errors.
        assert!(!is_bare_text(&response(
            axum::http::StatusCode::OK,
            &[("content-type", "application/json")]
        )));
        assert!(!is_bare_text(&response(
            axum::http::StatusCode::TEMPORARY_REDIRECT,
            &[("content-type", "text/html")]
        )));
        // A body that is not bare text a visitor would read.
        for content_type in [
            "text/html",
            "application/javascript",
            "text/csv",
            "text/event-stream",
            "image/webp",
        ] {
            assert!(
                !is_bare_text(&response(
                    axum::http::StatusCode::NOT_FOUND,
                    &[("content-type", content_type)]
                )),
                "{content_type}"
            );
        }
        // Compressed by an inner layer: unreadable here, left alone.
        assert!(!is_bare_text(&response(
            axum::http::StatusCode::BAD_GATEWAY,
            &[
                ("content-type", "application/json"),
                ("content-encoding", "gzip"),
            ]
        )));
        assert!(is_bare_text(&response(
            axum::http::StatusCode::BAD_GATEWAY,
            &[
                ("content-type", "application/json"),
                ("content-encoding", "identity"),
            ]
        )));
        // An attachment the browser was about to save.
        assert!(!is_bare_text(&response(
            axum::http::StatusCode::NOT_FOUND,
            &[
                ("content-type", "application/json"),
                ("content-disposition", "ATTACHMENT; filename=export.json"),
            ]
        )));
    }

    #[tokio::test]
    async fn a_disabled_landing_hands_the_response_back_untouched() {
        let landing = ErrorLanding::disabled();
        let original = response(
            axum::http::StatusCode::NOT_FOUND,
            &[("content-type", "application/json")],
        );
        assert!(!landing.wants(&Method::GET, &axum::http::HeaderMap::new()));
        // Not even a navigation gets a landing: `disabled` stays the switch
        // for a build that replaces nothing at all, whatever shell exists.
        // MUT:m1-standalone-disabled renders `standalone()` as `disabled()`;
        // the bite lands in the standalone test ABOVE, on its FIRST assertion
        // — `ErrorLanding::standalone().wants(&Method::GET, &navigation())`,
        // not on any document assertion. This test stays green under M1,
        // because `disabled` is exactly what it asserts.
        assert!(!landing.wants(&Method::GET, &navigation()));
        let out = Arc::new(landing).take_over(original).await;
        assert_eq!(out.status(), axum::http::StatusCode::NOT_FOUND);
        assert_eq!(
            out.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );
    }

    fn navigation() -> axum::http::HeaderMap {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
        headers
    }

    #[test]
    fn the_standalone_document_is_the_vendored_artifact() {
        // The provenance a vendored upstream copy is recognized by: hikari's
        // build stamps it, and the copy carries it unchanged.
        assert!(STANDALONE_DOCUMENT.starts_with("<!doctype html>"));
        assert!(
            STANDALONE_DOCUMENT
                .contains("<meta name=\"hk-error-landing\" content=\"v1 0.55.498 <dev>\">"),
            "the provenance meta survives the vendoring"
        );
        assert!(STANDALONE_DOCUMENT.trim_end().ends_with("</html>"));
    }

    #[tokio::test]
    async fn a_standalone_landing_seats_the_island_in_the_vendored_document() {
        // A real navigation wants it …
        assert!(ErrorLanding::standalone().wants(&Method::GET, &navigation()));
        let original = response(
            axum::http::StatusCode::BAD_GATEWAY,
            &[("content-type", "application/json")],
        );
        let out = Arc::new(ErrorLanding::standalone())
            .take_over(original)
            .await;
        assert_eq!(out.status(), axum::http::StatusCode::BAD_GATEWAY);
        assert_eq!(
            out.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );
        let (_, body) = out.into_parts();
        let document = String::from_utf8(
            http_body_util::BodyExt::collect(body)
                .await
                .expect("infallible body")
                .to_bytes()
                .to_vec(),
        )
        .expect("utf-8 document");
        // Both marks are in the answer: the island a reader consumes, and
        // the provenance meta that names the build it came from.
        assert!(document.contains("<script type=\"application/json\" id=\"hk-server-error\">"));
        assert!(
            document.contains("<meta name=\"hk-error-landing\" content=\"v1 0.55.498 <dev>\">")
        );
        // Nothing but the island moved: the artifact's own bytes around it
        // are whole. The layer seats the island off the response it took
        // over, so the comparison is the insertion shape rather than one
        // rebuilt island — this body's description is pinned elsewhere.
        let island_start = document
            .find("<script type=\"application/json\"")
            .expect("island");
        let island_end = island_start
            + document[island_start..]
                .find("</script>")
                .expect("island end")
            + "</script>".len();
        let mut without_island = document.clone();
        without_island.replace_range(island_start..island_end, "");
        assert_eq!(without_island, STANDALONE_DOCUMENT);
        // The island names the status the response actually carried.
        assert!(document[island_start..island_end].contains(r#""v":1,"status":502"#));
        // The markup is the artifact's own: one head to anchor at (the
        // bundled script mentions `</head>` in a string of its own, so the
        // count is the artifact's, not the document's), and one executable
        // script — the no-CSP constraint `standalone()` documents.
        assert_eq!(STANDALONE_DOCUMENT.matches("<head>").count(), 1);
        assert_eq!(STANDALONE_DOCUMENT.matches("</head>").count(), 1);
        assert_eq!(document.matches("<script type=\"module\">").count(), 1);
    }

    #[tokio::test]
    async fn the_standalone_document_takes_the_anchor_fallbacks_too() {
        // MUT:m2-inject-anchor-fallback drops the `</head>` / `</body>` scan
        // in `inject_island`; this test reds on its first assertion.
        // The artifact has both anchor levels, so pin them ON ITS BYTES:
        // the island lands before the FIRST `</head>`, at top level.
        let standalone = STANDALONE_DOCUMENT;
        let out = inject_island(standalone, "[i]");
        assert_eq!(
            out,
            standalone.replacen("</head>", "[i]</head>", 1),
            "before the first </head>, byte for byte around it"
        );
        // And with no `</head>` to take, on the next anchor down.
        let without_head = standalone.replace("</head>", "");
        assert_eq!(
            inject_island(&without_head, "[i]"),
            without_head.replacen("</body>", "[i]</body>", 1)
        );
        // Nothing at all: appended after the last byte.
        assert_eq!(
            inject_island("<!doctype html><p>fragment", "[i]"),
            "<!doctype html><p>fragment[i]"
        );
    }
}
