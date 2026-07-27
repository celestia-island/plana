use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::RwLock;

use axum::body::Body;
use axum::http::header;
use axum::response::Response;

const MIME_MAP: &[(&str, &str)] = &[
    ("html", "text/html; charset=utf-8"),
    ("css", "text/css; charset=utf-8"),
    ("js", "application/javascript; charset=utf-8"),
    ("mjs", "application/javascript; charset=utf-8"),
    ("json", "application/json; charset=utf-8"),
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("gif", "image/gif"),
    ("svg", "image/svg+xml"),
    ("ico", "image/x-icon"),
    ("webp", "image/webp"),
    ("glb", "model/gltf-binary"),
    ("wasm", "application/wasm"),
    ("woff2", "font/woff2"),
    ("woff", "font/woff"),
    ("ttf", "font/ttf"),
    ("txt", "text/plain; charset=utf-8"),
    ("xml", "application/xml"),
    ("webmanifest", "application/manifest+json"),
];

struct CachedFile {
    content: Vec<u8>,
    content_type: &'static str,
}

pub struct Cache {
    root: String,
    files: RwLock<Option<HashMap<String, CachedFile>>>,
}

impl Cache {
    pub fn new(root: &Path) -> Self {
        let root_str = root.to_string_lossy().to_string();
        let files = Self::scan(&root_str);
        tracing::info!("[dev-server] cached {} files from {}", files.len(), root_str);
        Self { root: root_str, files: RwLock::new(Some(files)) }
    }

    fn scan(root_str: &str) -> HashMap<String, CachedFile> {
        let root = Path::new(root_str);
        let mut map = HashMap::new();
        let mut stack = vec![root.to_path_buf()];

        while let Some(dir) = stack.pop() {
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push(path);
                    } else if let Ok(content) = fs::read(&path) {
                        if let Ok(rel) = path.strip_prefix(root) {
                            let key = rel.to_string_lossy().replace('\\', "/");
                            let ext = path.extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("");
                            let content_type = MIME_MAP.iter()
                                .find(|(e, _)| *e == ext)
                                .map(|(_, ct)| *ct)
                                .unwrap_or("application/octet-stream");

                            map.insert(key, CachedFile { content, content_type });
                        }
                    }
                }
            }
        }

        map
    }

    pub fn get(&self, key: &str) -> Option<Response> {
        let guard = self.files.read().unwrap();
        let map = guard.as_ref()?;
        let file = map.get(key)?;
        Some(file.to_response())
    }

    pub fn invalidate(&self, key: &str) {
        let mut guard = self.files.write().unwrap();
        if let Some(map) = guard.as_mut() {
            let path = Path::new(&self.root).join(key);
            if let Ok(content) = fs::read(&path) {
                let ext = Path::new(key).extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                let content_type = MIME_MAP.iter()
                    .find(|(e, _)| *e == ext)
                    .map(|(_, ct)| *ct)
                    .unwrap_or("application/octet-stream");

                map.insert(key.to_string(), CachedFile { content, content_type });
            } else {
                map.remove(key);
            }
        }
    }
}

impl CachedFile {
    pub fn to_response(&self) -> Response {
        let mut resp = Response::new(Body::from(self.content.clone()));

        // Set cache headers: short cache in dev mode
        resp.headers_mut().insert(
            header::CACHE_CONTROL,
            header::HeaderValue::from_static("public, max-age=1"),
        );
        resp.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static(self.content_type),
        );

        resp
    }
}
