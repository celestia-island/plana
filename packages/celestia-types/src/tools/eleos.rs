use crate::enums::WebSearchEngine;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/eleos.ts")]
pub struct WebSearchItem {
    pub url: String,
    pub title: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/eleos.ts")]
pub struct WebSearchResult {
    pub query: String,
    pub engine: WebSearchEngine,
    pub count: usize,
    pub results: Vec<WebSearchItem>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/eleos.ts")]
pub struct WebFetchResult {
    pub url: String,
    pub title: String,
    pub status_code: u16,
    pub headers: String,
    pub content: String,
    pub content_preview: String,
    pub content_length: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/eleos.ts")]
pub struct RemoteRefEntry {
    pub ref_id: String,
    pub url: String,
    pub title: String,
    pub ref_type: String,
    pub registered_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/eleos.ts")]
pub struct QueryRemoteRefsResult {
    pub count: usize,
    pub refs: Vec<RemoteRefEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/eleos.ts")]
pub struct RegisterRemoteRefsResult {
    pub ref_id: String,
    pub url: String,
    pub registered: bool,
}

// ── Tool parameter structs (for .d.ts API signature generation) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/eleos.ts")]
pub struct WebFetchParams {
    pub url: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/eleos.ts")]
pub struct WebSearchParams {
    pub query: String,
    pub engine: Option<String>,
    pub limit: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::WebSearchEngine;
    use serde_json::json;

    #[test]
    fn web_search_result_round_trip() {
        let r = WebSearchResult {
            query: "rust async runtime".into(),
            engine: WebSearchEngine::Duckduckgo,
            count: 2,
            results: vec![
                WebSearchItem {
                    url: "https://tokio.rs".into(),
                    title: "Tokio".into(),
                },
                WebSearchItem {
                    url: "https://async.rs".into(),
                    title: "async-std".into(),
                },
            ],
        };
        let v = serde_json::to_value(&r).unwrap();
        // WebSearchEngine serializes as PascalCase variant name.
        assert_eq!(v["engine"], "Duckduckgo");
        assert_eq!(v["count"], 2);
        assert_eq!(v["results"][0]["url"], "https://tokio.rs");
        let back: WebSearchResult = serde_json::from_value(v).unwrap();
        assert_eq!(back.results.len(), 2);
    }

    #[test]
    fn web_search_result_empty() {
        let r = WebSearchResult {
            query: "nothing".into(),
            engine: WebSearchEngine::Duckduckgo,
            count: 0,
            results: vec![],
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["results"], json!([]));
    }

    #[test]
    fn web_fetch_result_round_trip() {
        let r = WebFetchResult {
            url: "https://example.com".into(),
            title: "Example".into(),
            status_code: 200,
            headers: "content-type: text/html".into(),
            content: "<html>...</html>".into(),
            content_preview: "<html>...".into(),
            content_length: 1024,
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["status_code"], 200);
        assert_eq!(v["content_length"], 1024);
        let back: WebFetchResult = serde_json::from_value(v).unwrap();
        assert_eq!(back.status_code, 200);
    }

    #[test]
    fn remote_ref_entry_round_trip() {
        let r = RemoteRefEntry {
            ref_id: "ref-001".into(),
            url: "https://docs.rs".into(),
            title: "Rust Docs".into(),
            ref_type: "documentation".into(),
            registered_at: "2026-07-07T00:00:00Z".into(),
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["ref_type"], "documentation");
        let back: RemoteRefEntry = serde_json::from_value(v).unwrap();
        assert_eq!(back.ref_id, "ref-001");
    }

    #[test]
    fn query_remote_refs_result_round_trip() {
        let r = QueryRemoteRefsResult {
            count: 1,
            refs: vec![RemoteRefEntry {
                ref_id: "r1".into(),
                url: "https://x.com".into(),
                title: "X".into(),
                ref_type: "article".into(),
                registered_at: "2026-01-01".into(),
            }],
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["count"], 1);
    }

    #[test]
    fn register_remote_refs_result_round_trip() {
        let r = RegisterRemoteRefsResult {
            ref_id: "ref-002".into(),
            url: "https://new.url".into(),
            registered: true,
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["registered"], true);
    }

    #[test]
    fn web_search_params_minimal() {
        let p = WebSearchParams {
            query: "test".into(),
            engine: None,
            limit: None,
        };
        let v = serde_json::to_value(&p).unwrap();
        assert_eq!(v["query"], "test");
        assert_eq!(v["engine"], serde_json::Value::Null);
    }

    #[test]
    fn web_search_params_with_all_fields() {
        let p = WebSearchParams {
            query: "test".into(),
            engine: Some("duckduckgo".into()),
            limit: Some(10),
        };
        let v = serde_json::to_value(&p).unwrap();
        assert_eq!(v["limit"], 10);
    }

    #[test]
    fn web_search_engine_enum_round_trip() {
        let e = WebSearchEngine::Duckduckgo;
        let s = serde_json::to_string(&e).unwrap();
        // serde uses PascalCase variant name; as_str() returns the wire value.
        assert_eq!(s, r#""Duckduckgo""#);
        let back: WebSearchEngine = serde_json::from_str(&s).unwrap();
        assert_eq!(back, e);
        // as_str() gives the lowercase domain vocabulary string.
        assert_eq!(e.as_str(), "duckduckgo");
    }
}
