//! The axum layer: a failed navigation in, the landing out.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use axum::body::Body;
use axum::http::header;
use axum::http::{Method, Request, Response};
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
    fn wants(&self, method: &Method, headers: &axum::http::HeaderMap) -> bool {
        self.shell.is_some() && is_browser_navigation(method, headers)
    }

    /// Replace the response when it is an error a browser would have shown
    /// as bare text; hand back anything else exactly as it was.
    async fn take_over(self: Arc<Self>, response: Response<Body>) -> Response<Body> {
        let Some(shell) = self.shell.clone() else {
            return response;
        };
        if !response_is_bare_text(&response) {
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
fn response_is_bare_text(response: &Response<Body>) -> bool {
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
        // Decided here, off the request, so `take_over` never has to guess
        // from the response whether anyone was navigating.
        let wants = self.landing.wants(req.method(), req.headers());
        let landing = Arc::clone(&self.landing);
        let future = self.inner.call(req);
        Box::pin(async move {
            let response = future.await?;
            if wants {
                Ok(landing.take_over(response).await)
            } else {
                Ok(response)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(response_is_bare_text(&response(
            axum::http::StatusCode::NOT_FOUND,
            &[]
        )));
        assert!(response_is_bare_text(&response(
            axum::http::StatusCode::BAD_GATEWAY,
            &[("content-type", "application/json")]
        )));
        assert!(response_is_bare_text(&response(
            axum::http::StatusCode::FORBIDDEN,
            &[("content-type", "text/plain; charset=utf-8")]
        )));
        // Not errors.
        assert!(!response_is_bare_text(&response(
            axum::http::StatusCode::OK,
            &[("content-type", "application/json")]
        )));
        assert!(!response_is_bare_text(&response(
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
                !response_is_bare_text(&response(
                    axum::http::StatusCode::NOT_FOUND,
                    &[("content-type", content_type)]
                )),
                "{content_type}"
            );
        }
        // Compressed by an inner layer: unreadable here, left alone.
        assert!(!response_is_bare_text(&response(
            axum::http::StatusCode::BAD_GATEWAY,
            &[
                ("content-type", "application/json"),
                ("content-encoding", "gzip"),
            ]
        )));
        assert!(response_is_bare_text(&response(
            axum::http::StatusCode::BAD_GATEWAY,
            &[
                ("content-type", "application/json"),
                ("content-encoding", "identity"),
            ]
        )));
        // An attachment the browser was about to save.
        assert!(!response_is_bare_text(&response(
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
        let out = Arc::new(landing).take_over(original).await;
        assert_eq!(out.status(), axum::http::StatusCode::NOT_FOUND);
        assert_eq!(
            out.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );
    }
}
