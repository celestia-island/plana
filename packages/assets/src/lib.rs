//! Compile-time static asset embedding with axum serving.
//!
//! ## Quick start
//!
//! ```ignore
//! use include_dir::include_dir;
//! use plana_assets::AssetServer;
//!
//! static WEBUI: include_dir::Dir<'static> = include_dir!("dist/webui");
//!
//! let app = Router::new()
//!     .merge(AssetServer::new(&WEBUI).with_spa_fallback().into_router());
//! ```
//!
//! - `GET /`            → `dist/webui/index.html`
//! - `GET /assets/x.js` → `dist/webui/assets/x.js`
//! - `GET /app`         → `dist/webui/index.html` (SPA fallback)

use axum::{
    Router,
    body::Body,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use include_dir::Dir;

/// An axum-compatible static file server backed by an embedded directory.
pub struct AssetServer {
    root: &'static Dir<'static>,
    spa_fallback: bool,
    cache_control: Option<String>,
}

impl AssetServer {
    /// Create a new server for the given embedded directory tree.
    pub fn new(root: &'static Dir<'static>) -> Self {
        Self {
            root,
            spa_fallback: false,
            cache_control: None,
        }
    }

    /// Serve `index.html` for any path that does not match a real file.
    pub fn with_spa_fallback(mut self) -> Self {
        self.spa_fallback = true;
        self
    }

    /// Set a `Cache-Control` header value for all responses (e.g. `"no-store"`).
    pub fn with_cache_control(mut self, value: &str) -> Self {
        self.cache_control = Some(value.to_string());
        self
    }

    /// Convert into an axum [`Router`] that handles `/*path` and `/{*path}`.
    pub fn into_router(self) -> Router {
        let state = std::sync::Arc::new(self);
        Router::new()
            .route("/", get(root_handler))
            .route("/{*path}", get(path_handler))
            .with_state(state)
    }
}

/// Content-type for a file path.
fn mime_for(path: &str) -> &'static str {
    match std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
    {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css",
        "js" | "mjs" => "text/javascript",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "webp" => "image/webp",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "wasm" => "application/wasm",
        "txt" => "text/plain",
        "xml" => "application/xml",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => "application/octet-stream",
    }
}

fn insert_headers(resp: &mut Response, content_type: &str, cache_control: Option<&str>) {
    if let Ok(ct) = header::HeaderValue::from_str(content_type) {
        resp.headers_mut().insert(header::CONTENT_TYPE, ct);
    }
    if let Some(cc) = cache_control {
        resp.headers_mut().insert(
            header::CACHE_CONTROL,
            header::HeaderValue::from_str(cc).unwrap_or(header::HeaderValue::from_static("no-store")),
        );
    }
}

fn serve_file(dir: &'static Dir<'static>, path: &str) -> Option<Response> {
    let normalized = path.trim_start_matches('/');
    let file = dir.get_file(normalized)?;
    let content_type = mime_for(normalized);
    let body = Body::from(file.contents().to_vec());
    Some(Response::new(body))
}

async fn root_handler(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<AssetServer>>,
) -> Response {
    if let Some(mut resp) = serve_file(state.root, "/index.html") {
        insert_headers(&mut resp, "text/html; charset=utf-8", state.cache_control.as_deref());
        resp
    } else {
        (StatusCode::NOT_FOUND, "index.html not found").into_response()
    }
}

async fn path_handler(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<AssetServer>>,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response {
    if let Some(mut resp) = serve_file(state.root, &path) {
        let ct = mime_for(&path);
        insert_headers(&mut resp, ct, state.cache_control.as_deref());
        return resp;
    }
    if state.spa_fallback {
        if let Some(mut resp) = serve_file(state.root, "/index.html") {
            insert_headers(&mut resp, "text/html; charset=utf-8", state.cache_control.as_deref());
            return resp;
        }
    }
    (StatusCode::NOT_FOUND, "Not Found").into_response()
}
