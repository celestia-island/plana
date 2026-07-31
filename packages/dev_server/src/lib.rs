//! Dev-mode static file server with hot-reload.
//!
//! Provides `DevFileServer` — a self-contained axum service that serves
//! files from a directory and watches for changes using `notify`.
//! When vite rebuilds the frontend, the next request automatically picks
//! up the new files. No process restart needed.

mod cache;

use std::path::PathBuf;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use notify::Watcher;
use std::sync::Arc;

use cache::Cache;

pub struct DevFileServerConfig {
    pub root_dir: PathBuf,
    /// When true, unknown paths fall back to `index.html` (SPA mode).
    pub spa_fallback: bool,
}

pub struct DevFileServer {
    root: PathBuf,
    spa: bool,
    cache: Arc<Cache>,
}

impl DevFileServer {
    pub fn new(config: DevFileServerConfig) -> Self {
        let cache = Arc::new(Cache::new(&config.root_dir));
        let s = Self {
            root: config.root_dir,
            spa: config.spa_fallback,
            cache,
        };
        s.spawn_watcher();
        s
    }

    fn spawn_watcher(&self) {
        let root = self.root.clone();
        let cache = self.cache.clone();

        std::thread::spawn(move || {
            let (tx, rx) = std::sync::mpsc::channel();
            let mut watcher =
                notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
                    if let Ok(e) = res {
                        let _ = tx.send(e);
                    }
                })
                .ok();

            if let Some(ref mut w) = watcher {
                let _ = w.watch(&root, notify::RecursiveMode::Recursive);
            }

            for event in rx {
                for path in &event.paths {
                    if let Ok(rel) = path.strip_prefix(&root) {
                        let key = rel.to_string_lossy().replace('\\', "/");
                        if !key.is_empty() {
                            cache.invalidate(&key);
                        }
                    }
                }
            }
        });
    }

    pub fn into_router(self) -> Router {
        let cache = self.cache.clone();
        let spa = self.spa;

        Router::new().fallback(axum::routing::any(move |req: Request<Body>| {
            let cache = cache.clone();
            async move {
                let path = req.uri().path().trim_start_matches('/');
                serve_file(&cache, path, spa).await
            }
        }))
    }
}

async fn serve_file(cache: &Cache, path: &str, spa: bool) -> Response {
    let key = if path.is_empty() { "index.html" } else { path };

    if let Some(resp) = cache.get(key) {
        return resp;
    }

    // Try with trailing index.html for directory paths
    let index_key = format!("{}/index.html", key.trim_end_matches('/'));
    if let Some(resp) = cache.get(&index_key) {
        return resp;
    }

    // SPA fallback — serve index.html for unknown paths that look like routes
    if spa && !path.contains('.') {
        if let Some(resp) = cache.get("index.html") {
            return resp;
        }
    }

    let mut resp = Response::new(Body::from("Not Found"));
    *resp.status_mut() = StatusCode::NOT_FOUND;
    resp
}
