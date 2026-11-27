//! The server half of the family error landing.
//!
//! A browser that reaches an error by NAVIGATION — the OAuth callback whose
//! token exchange failed, a revoked share link, a 502 from a restart window
//! — used to be answered with the service's JSON (`{"error": …}`) and
//! nothing else: a tab full of raw JSON, no way back, nothing to copy but
//! what the viewport holds (a 2026-11-26 production report).
//!
//! This crate answers that navigation the way the family answers every
//! other failure: with the service's own SPA document carrying one JSON
//! data island, which hikari's `createErrorReporting` reads on install and
//! turns into the unified [`HkErrorLanding`] card.
//!
//! ```html
//! <script type="application/json" id="hk-server-error">
//!   {"v":1,"status":502,"code":"identity_provider_unreachable",
//!    "message":"identity provider unreachable: …","raw":{"error":"…"}}
//! </script>
//! ```
//!
//! # What is converted, and what is never touched
//!
//! The layer ([`ErrorLanding`]) rewrites a response only when ALL hold:
//!
//! 1. the request is a browser navigation: `Sec-Fetch-Dest: document|iframe`,
//!    or (for clients without `Sec-Fetch-*`) an `Accept` naming `text/html`.
//!    `Accept: */*` (curl, MCP clients), WebSocket upgrades, `HEAD` and
//!    `OPTIONS` never qualify — a `fetch`/XHR caller keeps its JSON and the
//!    SPA keeps showing its own toast;
//! 2. the status is an error (4xx / 5xx);
//! 3. the body is something a browser would have shown as bare text: a JSON
//!    document, `text/plain`, or empty. An HTML error page a service
//!    already renders, a CSV export, an event stream, an attachment — each
//!    passes through byte for byte.
//!
//! The answer keeps the REAL status code and the response's other headers
//! (`Set-Cookie`, `WWW-Authenticate`, `Retry-After`, CORS …), replaces the
//! body with the shell document plus the island, and marks itself
//! `Cache-Control: no-store`.
//!
//! # The shell
//!
//! The service hands in its SPA `index.html` — the same bytes it serves on
//! `/` (the embedded `include_dir!` tree, the `rust-embed` asset, the file
//! read at startup). The island is injected where the shell asks for it
//! ([`SERVER_ERROR_LANDING_MARKER`]), else before `</head>`, else before
//! `</body>`, else appended — always in top-level markup: text inside an
//! HTML comment, a `<script>` or a `<style>` element is never taken for an
//! anchor. The card hikari paints is the family's one design: an HTTP
//! status chip, the wire code, the service's copy, the raw response under
//! a details pane — a 4xx in the warning tone, a 5xx in the error one.
//!
//! Without a shell the layer is inert: a headless build without a frontend
//! keeps answering JSON.
//!
//! # A domain that knows its answer is browser-facing
//!
//! Some failures are decided before a response exists: an OAuth lane picks
//! its redirect-or-JSON shape first, and by the time it has a response worth
//! handing to the layer it has already committed to one. Such a caller
//! assembles the landing itself, out of the same rule and the same assembly
//! the layer applies — [`is_browser_navigation`] on the request, then
//! [`is_bare_text`] on the response it composed, then
//! [`ServerErrorLanding::into_response`] out of the same shell. Nothing in
//! the downstream decides anything of its own: a second copy of these rules
//! is the thing this surface exists to remove.
//!
//! ```no_run
//! use axum::body::Body;
//! use axum::http::{HeaderMap, Method, StatusCode, header};
//! use axum::response::Response;
//! use plana_error_landing::{ServerErrorLanding, is_bare_text, is_browser_navigation};
//!
//! /// The answer for one failure: the landing for a browser navigation,
//! /// the wire JSON for everyone else.
//! fn answer(
//!     method: &Method,
//!     request: &HeaderMap,
//!     status: StatusCode,
//!     wire: Response<Body>,
//!     shell: Option<&str>,
//! ) -> Response<Body> {
//!     // The request half of the family's one rule, then the response half.
//!     let Some(shell) = shell.filter(|_| is_browser_navigation(method, request)) else {
//!         return wire;
//!     };
//!     if !is_bare_text(&wire) {
//!         return wire;
//!     }
//!     let (parts, body) = wire.into_parts();
//!     // A caller with no body to read (the answer is decided, not found)
//!     // builds the landing from its own copy instead of `from_body`.
//!     let _ = body;
//!     ServerErrorLanding::new(status, "upstream_down", "the upstream is down")
//!         .into_response(parts, shell)
//! }
//!
//! # let mut request = HeaderMap::new();
//! # request.insert("sec-fetch-dest", header::HeaderValue::from_static("document"));
//! # let wire = Response::builder()
//! #     .status(StatusCode::BAD_GATEWAY)
//! #     .header(header::CONTENT_TYPE, "application/json")
//! #     .body(Body::from(r#"{"error":"upstream down"}"#))
//! #     .expect("static response");
//! # let answer = answer(&Method::GET, &request, StatusCode::BAD_GATEWAY, wire, Some("<html></html>"));
//! # assert_eq!(answer.headers()[header::CONTENT_TYPE], "text/html; charset=utf-8");
//! ```
//!
//! # A service with no shell of its own
//!
//! A service that serves nothing but an API has no `index.html` to seat the
//! island in. [`ErrorLanding::standalone`] answers those with the family's
//! standalone landing document, which this crate ships: one file carrying
//! the family card, its copy in the family's locales, and the reader that
//! turns the island into it — so the failed navigation lands on the same
//! card a service with a shell shows. The document is hikari's
//! `packages/vue/src/errorLandingStandalone/error-landing.html`, vendored
//! byte for byte; its [`hk-error-landing`][standalone] meta names the hikari
//! version it was built from.
//!
//! That document carries one EXECUTABLE inline script. A service that sets
//! its own Content-Security-Policy therefore keeps its SPA shell — such a
//! service's `script-src` would refuse the landing's script, and a card that
//! never paints is worse than the shell it already has. Standalone is for
//! services with no CSP and no shell; [`ErrorLanding::disabled`] stays the
//! switch for a build that should replace nothing at all.
//!
//! ```no_run
//! use axum::Router;
//! use plana_error_landing::ErrorLanding;
//!
//! let app: Router = Router::new().layer(ErrorLanding::standalone().layer());
//! # let _ = app;
//! ```
//!
//! ```no_run
//! use std::sync::Arc;
//! use axum::{http::StatusCode, routing::get, Json, Router};
//! use plana_error_landing::ErrorLanding;
//!
//! # fn shell() -> String { String::new() }
//! let app: Router = Router::new()
//!     .route("/boom", get(|| async {
//!         (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": "upstream down" })))
//!     }))
//!     // Innermost: every route's error response passes through it.
//!     .layer(ErrorLanding::new(shell()).layer());
//! # let _ = app;
//! ```
//!
//! [`HkErrorLanding`]: https://github.com/celestia-island/hikari/blob/master/packages/vue/src/components/HkErrorLanding.tsx
//! [standalone]: https://github.com/celestia-island/hikari/blob/master/packages/vue/src/errorLandingStandalone/error-landing.html

#![forbid(unsafe_code)]

mod detect;
mod handoff;
mod layer;

pub use detect::{BodyKind, is_browser_navigation};
pub use handoff::{
    SERVER_ERROR_LANDING_ID, SERVER_ERROR_LANDING_MARKER, SERVER_ERROR_LANDING_VERSION,
    ServerErrorLanding, inject_island,
};
pub use layer::{
    ErrorLanding, ErrorLandingLayer, ErrorLandingService, MAX_ERROR_BODY_BYTES, is_bare_text,
};
