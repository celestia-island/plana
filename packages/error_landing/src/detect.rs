//! Which requests are a browser navigation, and which bodies a browser
//! would have been shown as bare text.

use axum::http::header;
use axum::http::{HeaderMap, HeaderValue, Method};

/// What a response body is, by `Content-Type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyKind {
    /// `application/json` and every `+json` subtype (RFC 9457 included).
    Json,
    /// `text/plain`, whatever its charset parameter.
    Text,
    /// No `Content-Type` at all (axum's bare 404 / 405 answers).
    None,
    /// Anything else: HTML, JavaScript, CSV, an event stream, an image …
    Other,
}

impl BodyKind {
    /// Classify one `Content-Type` header value. Parameters (`; charset=…`)
    /// and case are ignored; an unparsable value is [`Self::Other`] — a body
    /// the service could not even label is not something to replace.
    #[must_use]
    pub fn of(value: &HeaderValue) -> Self {
        let Ok(raw) = value.to_str() else {
            return Self::Other;
        };
        let essence = raw
            .split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        if essence.is_empty() {
            return Self::None;
        }
        match essence.as_str() {
            "application/json" => Self::Json,
            other if other.starts_with("application/") && other.ends_with("+json") => Self::Json,
            other if other.starts_with("text/plain") => Self::Text,
            _ => Self::Other,
        }
    }
}

/// Whether the request is a browser NAVIGATION for a document — the only
/// kind of request a landing can be the answer to.
///
/// `Sec-Fetch-Dest` is authoritative where present (every current browser
/// sends it): `document` is a tab or a top-level redirect chain, `iframe` a
/// framed document; `image`, `script`, `empty` (fetch/XHR), `websocket` and
/// the rest are resource loads for a page that is already up and renders
/// its own errors. Clients that send no `Sec-Fetch-*` (an old proxy that
/// strips them, a text browser) fall back to `Accept`: a navigation names
/// `text/html`, an API client names `*/*` or a JSON type and must keep its
/// JSON. A WebSocket upgrade never qualifies, and neither do `HEAD` (a
/// landing has a body) or `OPTIONS` (a CORS preflight).
#[must_use]
pub fn is_browser_navigation(method: &Method, headers: &HeaderMap) -> bool {
    if method == Method::HEAD || method == Method::OPTIONS {
        return false;
    }
    if headers.contains_key(header::UPGRADE) {
        return false;
    }
    let dests: Vec<&HeaderValue> = headers.get_all("sec-fetch-dest").iter().collect();
    if !dests.is_empty() {
        let mut unreadable = false;
        for value in &dests {
            match value.to_str() {
                Ok(dest) => {
                    if matches!(
                        dest.trim().to_ascii_lowercase().as_str(),
                        "document" | "iframe" | "nested-document"
                    ) {
                        return true;
                    }
                }
                // Present but not UTF-8: the sender is a browser-class client
                // whose verdict cannot be read, so the Accept fallback decides.
                Err(_) => unreadable = true,
            }
        }
        if !unreadable {
            return false;
        }
    }
    accepts_html(headers)
}

/// The `Accept` fallback: at least one media range names `text/html`, and
/// not with `q=0` — that is an explicit refusal of it.
fn accepts_html(headers: &HeaderMap) -> bool {
    headers.get_all(header::ACCEPT).iter().any(|value| {
        value
            .to_str()
            .unwrap_or_default()
            .split(',')
            .any(range_wants_html)
    })
}

/// Whether one `Accept` media range (the text between commas) asks for
/// `text/html`: the media type is it, and no `q` parameter refuses it.
fn range_wants_html(range: &str) -> bool {
    let mut parts = range.split(';');
    let media = parts.next().unwrap_or_default().trim();
    if !media.eq_ignore_ascii_case("text/html") {
        return false;
    }
    !parts.any(|parameter| {
        parameter
            .trim()
            .split_once('=')
            .is_some_and(|(name, value)| {
                name.trim().eq_ignore_ascii_case("q")
                    && value
                        .trim()
                        .parse::<f32>()
                        .is_ok_and(|weight| weight == 0.0)
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(&'static str, &str)]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (name, value) in pairs {
            headers.append(
                *name,
                HeaderValue::from_str(value).expect("static header value"),
            );
        }
        headers
    }

    #[test]
    fn a_tab_navigation_is_a_document() {
        assert!(is_browser_navigation(
            &Method::GET,
            &headers(&[("sec-fetch-dest", "document")])
        ));
        assert!(is_browser_navigation(
            &Method::GET,
            &headers(&[("sec-fetch-dest", "Document"), ("accept", "*/*")])
        ));
        assert!(is_browser_navigation(
            &Method::GET,
            &headers(&[("sec-fetch-dest", "iframe")])
        ));
        // A document nested through <embed>/<object> is a framed document too.
        assert!(is_browser_navigation(
            &Method::GET,
            &headers(&[("sec-fetch-dest", "nested-document")])
        ));
        // A repeated header is read as a whole: a navigation anywhere in it
        // is a navigation.
        let mut repeated = headers(&[("sec-fetch-dest", "empty")]);
        repeated.append("sec-fetch-dest", HeaderValue::from_static("document"));
        assert!(is_browser_navigation(&Method::GET, &repeated));
    }

    #[test]
    fn resource_loads_and_api_calls_are_not() {
        for dest in [
            "empty",
            "image",
            "script",
            "style",
            "websocket",
            "worker",
            "audio",
            "video",
        ] {
            // Even with a browser-like Accept: Sec-Fetch-Dest wins.
            assert!(
                !is_browser_navigation(
                    &Method::GET,
                    &headers(&[("sec-fetch-dest", dest), ("accept", "text/html,*/*;q=0.8")])
                ),
                "dest {dest}"
            );
        }
    }

    #[test]
    fn without_sec_fetch_the_accept_header_decides() {
        let nav = headers(&[(
            "accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        )]);
        assert!(is_browser_navigation(&Method::GET, &nav));
        let nav_spaced = headers(&[("accept", "application/json, TEXT/HTML ;q=0.9")]);
        assert!(is_browser_navigation(&Method::GET, &nav_spaced));
        for accept in [
            "*/*",
            "application/json",
            "application/json, text/plain",
            "text/html-ish",
            // `q=0` is an explicit refusal of HTML, not a request for it.
            "text/html;q=0",
            "text/html; q=0.0, application/json",
            "",
        ] {
            assert!(
                !is_browser_navigation(&Method::GET, &headers(&[("accept", accept)])),
                "accept {accept:?}"
            );
        }
        assert!(!is_browser_navigation(&Method::GET, &HeaderMap::new()));
    }

    #[test]
    fn upgrades_head_and_preflights_never_qualify() {
        assert!(!is_browser_navigation(
            &Method::GET,
            &headers(&[("sec-fetch-dest", "document"), ("upgrade", "websocket")])
        ));
        assert!(!is_browser_navigation(
            &Method::HEAD,
            &headers(&[("sec-fetch-dest", "document")])
        ));
        assert!(!is_browser_navigation(
            &Method::OPTIONS,
            &headers(&[("accept", "text/html")])
        ));
    }

    /// The two headers pull in opposite directions here: `Sec-Fetch-Dest`
    /// names a document, `Accept` names JSON. `Sec-Fetch-Dest` is
    /// authoritative, so the request is a navigation and the landing's
    /// destination half is what must stay. The reverse pair — a resource
    /// load that ASKS for HTML — belongs to the test below it, and both
    /// under the same rule.
    ///
    /// Where `Sec-Fetch-Dest` names no document, `Accept` alone decides:
    /// `empty` beside `*/*` is not a navigation, `document` beside `*/*` is.
    // MUT:m3-detect-accept-fallback drops the `return false` for a readable
    // non-document destination; MUT:m4-detect-permissive-return turns it into
    // `return true`. Both red the `empty` rows below.
    #[test]
    fn a_document_destination_outranks_a_json_accept() {
        for (dest, accept, navigation) in [
            ("document", "application/json", true),
            ("iframe", "application/json", true),
            // The controls on either side of that verdict.
            ("document", "*/*", true),
            ("empty", "text/html", false),
            ("empty", "application/json", false),
            ("empty", "*/*", false),
        ] {
            assert_eq!(
                is_browser_navigation(
                    &Method::GET,
                    &headers(&[("sec-fetch-dest", dest), ("accept", accept)])
                ),
                navigation,
                "dest {dest} accept {accept}"
            );
        }
        // No `Sec-Fetch-Dest` at all: the `Accept` fallback answers.
        assert!(is_browser_navigation(
            &Method::GET,
            &headers(&[("accept", "text/html")])
        ));
        assert!(!is_browser_navigation(
            &Method::GET,
            &headers(&[("accept", "application/json")])
        ));
    }

    #[test]
    fn body_kind_by_content_type() {
        for (raw, kind) in [
            ("application/json", BodyKind::Json),
            ("application/json; charset=utf-8", BodyKind::Json),
            ("Application/JSON", BodyKind::Json),
            ("application/problem+json", BodyKind::Json),
            ("application/vnd.erp.error+json", BodyKind::Json),
            ("text/plain", BodyKind::Text),
            ("text/plain; charset=iso-8859-1", BodyKind::Text),
            ("", BodyKind::None),
            ("text/html; charset=utf-8", BodyKind::Other),
            ("application/javascript", BodyKind::Other),
            ("text/csv", BodyKind::Other),
            ("text/event-stream", BodyKind::Other),
            ("application/octet-stream", BodyKind::Other),
            ("image/webp", BodyKind::Other),
        ] {
            assert_eq!(
                BodyKind::of(&HeaderValue::from_str(raw).expect("static")),
                kind,
                "{raw:?}"
            );
        }
    }
}
