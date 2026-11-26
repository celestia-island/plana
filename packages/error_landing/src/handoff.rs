//! The handoff payload, its data island, and how it is seated in a shell.

use axum::body::Body;
use axum::http::header;
use axum::http::{HeaderValue, StatusCode};
use serde::Serialize;
use serde_json::{Value, json};

use crate::detect::BodyKind;

/// Element id of the JSON island hikari's `createErrorReporting` reads.
pub const SERVER_ERROR_LANDING_ID: &str = "hk-server-error";

/// The only island shape this crate writes; hikari refuses any other `v`.
pub const SERVER_ERROR_LANDING_VERSION: u8 = 1;

/// An HTML comment a shell may carry to choose where the island lands. When
/// present it is REPLACED by the island; otherwise the island goes before
/// `</head>`, else before `</body>`, else at the very end.
pub const SERVER_ERROR_LANDING_MARKER: &str = "<!--hk-server-error-->";

/// Headers of the JSON answer that describe a body the landing replaces.
const REPLACED_BODY_HEADERS: [header::HeaderName; 4] = [
    header::CONTENT_TYPE,
    header::CONTENT_LENGTH,
    header::CONTENT_ENCODING,
    header::TRANSFER_ENCODING,
];

/// The request headers the landing's eligibility depends on, written for
/// `Vary` the way the HTTP grammar spells them.
const VARY_ON: [&str; 2] = ["Sec-Fetch-Dest", "Accept"];

/// A failed navigation as the landing shows it — the island's content.
///
/// Built by a service that knows the copy it wants ([`ServerErrorLanding::new`]),
/// or read off a wire error body the family already answers with
/// ([`ServerErrorLanding::from_body`]).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ServerErrorLanding {
    /// The island shape version hikari validates before rendering.
    #[serde(rename = "v")]
    version: u8,
    /// The HTTP status the service ANSWERED with — the card's `HTTP <n>` chip.
    pub status: u16,
    /// The wire error code chip. Empty: hikari falls back to `http_<status>`.
    pub code: String,
    /// The service's explanation — the card's description. Never left empty:
    /// an empty one becomes the status's canonical reason.
    pub message: String,
    /// The body the service would have sent, shown in the card's details pane.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<Value>,
}

impl ServerErrorLanding {
    /// A handoff with the service's own copy. `message` empty is replaced by
    /// the status's canonical reason, so the card never shows a blank line.
    #[must_use]
    pub fn new(status: StatusCode, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::compose(status, code.into(), message.into(), None)
    }

    /// Attach the body the card's details pane should show.
    #[must_use]
    pub fn raw(mut self, raw: Value) -> Self {
        self.raw = Some(raw);
        self
    }

    /// The one constructor: an empty message becomes the status's canonical
    /// reason, so the card never shows a blank line.
    fn compose(status: StatusCode, code: String, message: String, raw: Option<Value>) -> Self {
        let message = if message.trim().is_empty() {
            status.canonical_reason().unwrap_or("HTTP error").to_owned()
        } else {
            message
        };
        Self {
            version: SERVER_ERROR_LANDING_VERSION,
            status: status.as_u16(),
            code,
            message,
            raw,
        }
    }

    /// Read a wire error body the way the family writes them:
    ///
    /// | Shape | Where it comes from | `code` | `message` |
    /// |---|---|---|---|
    /// | `{"error": "<text>"}` | the family's most common error body | — | the text |
    /// | `{"error": "<code>", "message": "<text>"}` | a wire code beside its text | the code | the text |
    /// | `{"error": {"code", "message", "type"}}` | a nested error object; JSON-RPC error frames | `code`, else `type` | `message` |
    /// | `{"code", "message"}` / RFC 9457 `{"type", "title", "detail"}` | a flat code pair; problem details | `code`, else `type` | `message`, else `detail`, else `title` |
    /// | any `text/plain`, or a body declared JSON that does not parse | a gateway's API lane; axum's own defaults | — | the text |
    /// | empty | axum's bare 404 / 405 | — | the status's canonical reason |
    ///
    /// A body that parses to JSON but matches no row keeps the whole document
    /// as `raw`, so nothing the service said is lost. A heuristic, not a
    /// contract: a service whose copy deserves better calls [`Self::new`].
    #[must_use]
    pub fn from_body(status: StatusCode, content_type: Option<&HeaderValue>, body: &[u8]) -> Self {
        let text = || String::from_utf8_lossy(body).trim().to_owned();
        let (code, message, raw): (String, String, Option<Value>) =
            match content_type.map(BodyKind::of).unwrap_or(BodyKind::None) {
                BodyKind::Json => match serde_json::from_slice::<Value>(body) {
                    Ok(value) => {
                        // A body that is not an object (an array, a bare
                        // string) names nothing to describe: it rides as
                        // `raw` whole.
                        let (code, message) = match &value {
                            Value::Object(object) => describe(object),
                            _ => (String::new(), String::new()),
                        };
                        (code, message, Some(value))
                    }
                    // Declared JSON that does not parse: show it as the text it
                    // is — and an empty body is simply empty (no `raw`).
                    Err(_) => {
                        let text = text();
                        let raw = (!text.is_empty()).then(|| Value::String(text.clone()));
                        (String::new(), text, raw)
                    }
                },
                BodyKind::Text | BodyKind::None | BodyKind::Other => {
                    let text = text();
                    let raw = (!text.is_empty()).then(|| Value::String(text.clone()));
                    (String::new(), text, raw)
                }
            };
        Self::compose(status, code, message, raw)
    }

    /// The island element: a JSON data block, never executed, with the few
    /// characters that could close it or read as markup escaped.
    #[must_use]
    pub fn island(&self) -> String {
        // Serializing this struct cannot fail (strings and a `Value`); the
        // arm exists so a future field cannot turn a landing into a panic.
        let json = serde_json::to_string(self).unwrap_or_else(|_| {
            json!({ "v": SERVER_ERROR_LANDING_VERSION, "status": self.status, "code": self.code, "message": self.message })
                .to_string()
        });
        format!(
            "<script type=\"application/json\" id=\"{SERVER_ERROR_LANDING_ID}\">{}</script>",
            escape_island_json(&json)
        )
    }

    /// The shell document with the island seated in it (see
    /// [`inject_island`] for where).
    #[must_use]
    pub fn document(&self, shell: &str) -> String {
        inject_island(shell, &self.island())
    }

    /// The HTTP answer: the shell with the island, at the status the service
    /// answered with, uncacheable, with every header the service set except
    /// the four that described the replaced body.
    pub(crate) fn into_response(
        self,
        parts: axum::http::response::Parts,
        shell: &str,
    ) -> axum::response::Response {
        let vary = merged_vary(&parts.headers);
        let mut headers = axum::http::HeaderMap::new();
        for (name, value) in &parts.headers {
            if REPLACED_BODY_HEADERS.contains(name) || name == header::VARY {
                continue;
            }
            headers.append(name, value.clone());
        }
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        );
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
        headers.insert(header::VARY, vary);
        if !headers.contains_key(header::X_CONTENT_TYPE_OPTIONS) {
            headers.insert(
                header::X_CONTENT_TYPE_OPTIONS,
                HeaderValue::from_static("nosniff"),
            );
        }
        let mut parts = parts;
        parts.headers = headers;
        axum::response::Response::from_parts(parts, Body::from(self.document(shell)))
    }
}

/// Escape the characters that could let a payload close the `<script>`
/// element or be read as markup by a parser that never treats the island as
/// a data block. `JSON.parse` turns every one of them back.
#[must_use]
pub(crate) fn escape_island_json(json: &str) -> String {
    let mut out = String::with_capacity(json.len());
    for ch in json.chars() {
        match ch {
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            _ => out.push(ch),
        }
    }
    out
}

/// Seat the island in a shell: at [`SERVER_ERROR_LANDING_MARKER`] (which it
/// replaces — a marker comment is not something to keep), else right before
/// the first `</head>`, else the first `</body>`, else appended. Anchors
/// match case-insensitively; byte offsets found in the lowercased shell are
/// valid in the original, since ASCII lowercasing keeps every byte's length.
#[must_use]
pub fn inject_island(shell: &str, island: &str) -> String {
    let lower = shell.to_ascii_lowercase();
    if let Some(at) = lower.find(SERVER_ERROR_LANDING_MARKER) {
        let mut out = String::with_capacity(shell.len() + island.len());
        out.push_str(&shell[..at]);
        out.push_str(island);
        out.push_str(&shell[at + SERVER_ERROR_LANDING_MARKER.len()..]);
        return out;
    }
    for anchor in ["</head>", "</body>"] {
        if let Some(at) = lower.find(anchor) {
            let mut out = String::with_capacity(shell.len() + island.len());
            out.push_str(&shell[..at]);
            out.push_str(island);
            out.push_str(&shell[at..]);
            return out;
        }
    }
    let mut out = String::with_capacity(shell.len() + island.len());
    out.push_str(shell);
    out.push_str(island);
    out
}

/// `Vary` for the landing: what the service already varied on, plus the two
/// request headers this layer's own decision depends on. The same URL is
/// JSON to a `fetch` and HTML to a navigation, so any cache in between has
/// to know — even though the landing itself is `no-store`.
#[must_use]
fn merged_vary(original: &axum::http::HeaderMap) -> HeaderValue {
    // The service's own tokens keep the casing it wrote them in; the two
    // this layer adds are spelled canonically. Duplicates are case-insensitive.
    let mut names: Vec<String> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    let push = |name: String, names: &mut Vec<String>, seen: &mut Vec<String>| {
        let lowered = name.to_ascii_lowercase();
        if !lowered.is_empty() && !seen.contains(&lowered) {
            seen.push(lowered);
            names.push(name);
        }
    };
    for value in original.get_all(header::VARY) {
        for name in value.to_str().unwrap_or_default().split(',') {
            push(name.trim().to_owned(), &mut names, &mut seen);
        }
    }
    if names.iter().any(|name| name == "*") {
        return HeaderValue::from_static("*");
    }
    for written in VARY_ON {
        push(written.to_owned(), &mut names, &mut seen);
    }
    HeaderValue::from_str(&names.join(", "))
        .unwrap_or_else(|_| HeaderValue::from_static("Sec-Fetch-Dest, Accept"))
}

/// `code` and `message` out of a JSON error object (the table on
/// [`ServerErrorLanding::from_body`]).
fn describe(object: &serde_json::Map<String, Value>) -> (String, String) {
    let top_message = || {
        object
            .get("message")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(ToOwned::to_owned)
    };
    match object.get("error") {
        // `{"error": "<text>"}` — and, with a top-level `message`,
        // `{"error": "<code>", "message": "<text>"}`.
        Some(Value::String(error)) => match top_message() {
            Some(message) => (error.trim().to_owned(), message),
            None => (String::new(), error.trim().to_owned()),
        },
        // `{"error": {"code" | "type", "message"}}`.
        Some(Value::Object(inner)) => {
            let message = inner
                .get("message")
                .or_else(|| inner.get("prose"))
                .and_then(Value::as_str)
                .map(str::trim)
                .unwrap_or_default()
                .to_owned();
            let code = ["code", "type", "kind"]
                .iter()
                .find_map(|key| inner.get(*key).and_then(scalar))
                .unwrap_or_default();
            (code, message)
        }
        // `{"code", "message"}`, RFC 9457 `{"type", "title", "detail"}`.
        _ => {
            let message = top_message().unwrap_or_else(|| {
                ["detail", "title", "error"]
                    .iter()
                    .find_map(|key| object.get(*key).and_then(Value::as_str))
                    .map(str::trim)
                    .unwrap_or_default()
                    .to_owned()
            });
            let code = ["code", "type", "kind"]
                .iter()
                .find_map(|key| object.get(*key).and_then(scalar))
                .map(|code| code.rsplit(['/', '#']).next().unwrap_or(&code).to_owned())
                .unwrap_or_default();
            (code, message)
        }
    }
}

/// A code-bearing value as a chip string: strings as written, numbers
/// verbatim (JSON-RPC's `-32000`), anything else nothing.
fn scalar(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.trim().to_owned()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn island_of(landing: &ServerErrorLanding) -> String {
        landing.island()
    }

    #[test]
    fn island_is_a_data_block_with_the_version_and_no_raw_when_absent() {
        let landing = ServerErrorLanding::new(StatusCode::NOT_FOUND, "not_found", "no such file");
        assert_eq!(
            island_of(&landing),
            r#"<script type="application/json" id="hk-server-error">{"v":1,"status":404,"code":"not_found","message":"no such file"}</script>"#
        );
    }

    #[test]
    fn island_escapes_markup_so_no_payload_can_close_it() {
        let landing =
            ServerErrorLanding::new(StatusCode::BAD_REQUEST, "x", "</script><b>&\u{2028}")
                .raw(json!({ "next": "https://example.test/?a=1&b=2" }));
        let island = island_of(&landing);
        assert!(!island.contains("</script><script"));
        assert_eq!(island.matches("</script>").count(), 1);
        assert!(island.contains("\\u003c/script\\u003e\\u003cb\\u003e\\u0026\\u2028"));
        assert!(island.contains("a=1\\u0026b=2"));
    }

    #[test]
    fn an_empty_message_becomes_the_status_reason() {
        let landing = ServerErrorLanding::new(StatusCode::SERVICE_UNAVAILABLE, "", "  ");
        assert_eq!(landing.message, "Service Unavailable");
        assert_eq!(landing.status, 503);
    }

    #[test]
    fn inject_uses_the_marker_and_replaces_it() {
        let shell = "<!doctype html><html><head><!--HK-SERVER-ERROR--></head><body>x</body></html>";
        let out = inject_island(shell, "[island]");
        assert_eq!(
            out,
            "<!doctype html><html><head>[island]</head><body>x</body></html>"
        );
    }

    #[test]
    fn inject_falls_back_to_head_then_body_then_the_end() {
        assert_eq!(
            inject_island("<HTML><HEAD></HEAD><BODY></BODY></HTML>", "[i]"),
            "<HTML><HEAD>[i]</HEAD><BODY></BODY></HTML>"
        );
        assert_eq!(
            inject_island("<html><body>y</body></html>", "[i]"),
            "<html><body>y[i]</body></html>"
        );
        assert_eq!(inject_island("<p>fragment", "[i]"), "<p>fragment[i]");
        // Multi-byte text before the anchor keeps the byte offset honest.
        assert_eq!(
            inject_island("<html><head><title>错误页</title></head></html>", "[i]"),
            "<html><head><title>错误页</title>[i]</head></html>"
        );
    }

    #[test]
    fn inject_is_pure_and_leaves_the_rest_byte_identical() {
        let shell = "<!doctype html>\n<html lang=\"zh-Hans\">\n<head>\n<meta charset=\"utf-8\">\n</head>\n<body>\n<div id=\"app\"></div>\n</body>\n</html>\n";
        let out = inject_island(shell, "[i]");
        assert_eq!(out.replace("[i]", ""), shell);
    }

    #[test]
    fn from_body_reads_every_family_shape() {
        let json = HeaderValue::from_static("application/json");
        let problem = HeaderValue::from_static("application/problem+json");
        let cases: [(StatusCode, Option<HeaderValue>, &str, &str, &str); 8] = [
            (
                StatusCode::BAD_GATEWAY,
                Some(json.clone()),
                r#"{"error":"identity provider unreachable: identity provider answered 400 Bad Request"}"#,
                "",
                "identity provider unreachable: identity provider answered 400 Bad Request",
            ),
            (
                StatusCode::NOT_FOUND,
                Some(json.clone()),
                r#"{"error":"not_found","message":"no such file"}"#,
                "not_found",
                "no such file",
            ),
            (
                StatusCode::FORBIDDEN,
                Some(json.clone()),
                r#"{"error":{"message":"denied by policy","type":"forbidden","code":"policy_denied"}}"#,
                "policy_denied",
                "denied by policy",
            ),
            (
                StatusCode::UNAUTHORIZED,
                Some(json.clone()),
                r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32005,"message":"auth error"}}"#,
                "-32005",
                "auth error",
            ),
            (
                StatusCode::NOT_FOUND,
                Some(problem),
                r#"{"type":"https://example.test/probs/missing","title":"Not Found","status":404,"detail":"share was revoked"}"#,
                "missing",
                "share was revoked",
            ),
            (
                StatusCode::BAD_REQUEST,
                Some(json),
                r#"{"success":false,"error":"备案号不能为空"}"#,
                "",
                "备案号不能为空",
            ),
            (
                StatusCode::BAD_REQUEST,
                Some(HeaderValue::from_static("application/json; charset=utf-8")),
                "{not json",
                "",
                "{not json",
            ),
            (StatusCode::NOT_FOUND, None, "", "", "Not Found"),
        ];
        for (status, content_type, body, code, message) in cases {
            let landing =
                ServerErrorLanding::from_body(status, content_type.as_ref(), body.as_bytes());
            assert_eq!(
                (landing.code.as_str(), landing.message.as_str()),
                (code, message),
                "body {body}"
            );
            assert_eq!(landing.status, status.as_u16());
        }
    }

    #[test]
    fn from_body_keeps_what_it_could_not_describe_as_raw() {
        let landing = ServerErrorLanding::from_body(
            StatusCode::IM_A_TEAPOT,
            Some(&HeaderValue::from_static("application/json")),
            b"[1,2,3]",
        );
        assert_eq!(landing.code, "");
        assert_eq!(landing.message, "I'm a teapot");
        assert_eq!(landing.raw, Some(json!([1, 2, 3])));
    }

    #[test]
    fn from_body_shows_plain_text_and_drops_lossy_bytes() {
        let landing = ServerErrorLanding::from_body(
            StatusCode::BAD_GATEWAY,
            Some(&HeaderValue::from_static("text/plain; charset=utf-8")),
            b"  gateway: backend unavailable\n\xff ",
        );
        assert_eq!(landing.message, "gateway: backend unavailable\n\u{fffd}");
        assert_eq!(landing.raw, Some(Value::String(landing.message.clone())));
    }

    #[test]
    fn vary_merges_and_keeps_a_star() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));
        assert_eq!(
            merged_vary(&headers),
            "Accept-Encoding, Sec-Fetch-Dest, Accept"
        );
        headers.append(
            header::VARY,
            HeaderValue::from_static("sec-fetch-dest, Origin"),
        );
        assert_eq!(
            merged_vary(&headers),
            "Accept-Encoding, sec-fetch-dest, Origin, Accept"
        );
        headers.insert(header::VARY, HeaderValue::from_static("*"));
        assert_eq!(merged_vary(&headers), "*");
    }
}
