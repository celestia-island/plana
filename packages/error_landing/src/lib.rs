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

#![forbid(unsafe_code)]

mod detect;
mod handoff;
mod layer;

pub use detect::{BodyKind, is_browser_navigation};
pub use handoff::{
    SERVER_ERROR_LANDING_ID, SERVER_ERROR_LANDING_MARKER, SERVER_ERROR_LANDING_VERSION,
    ServerErrorLanding, inject_island,
};
pub use layer::{ErrorLanding, ErrorLandingLayer, ErrorLandingService, MAX_ERROR_BODY_BYTES};
