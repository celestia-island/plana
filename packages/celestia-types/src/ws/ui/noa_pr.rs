//! PR platform — SyncMessage variant params (P6#B4).
//!
//! Mirrors the PR record shape implemented in noa (`src/forge` + the
//! self-hosted `/api/v1/prs` store): forge-agnostic PR summaries and details
//! carrying the platform-specific metadata markers (model / token counts /
//! cost) produced by the entelecheia dogfood loop.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/noaPr.ts")]
#[serde(rename_all = "lowercase")]
pub enum PrState {
    Open,
    Closed,
    Merged,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/noaPr.ts")]
pub struct PrMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub input_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub output_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub cost_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/noaPr.ts")]
pub struct PullRequestSummary {
    /// Forge-local identifier (GitHub number / self-hosted id).
    pub id: String,
    pub number: u64,
    pub title: String,
    pub state: PrState,
    /// Target branch / workspace.
    pub base: String,
    /// Source branch / workspace.
    pub head: String,
    /// Web URL of the PR (empty for the self-hosted store in v1b).
    pub url: String,
    /// Unix seconds.
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/noaPr.ts")]
pub struct PullRequestDetail {
    /// Forge-local identifier (GitHub number / self-hosted id).
    pub id: String,
    pub number: u64,
    pub title: String,
    pub state: PrState,
    pub base: String,
    pub head: String,
    pub url: String,
    pub created_at: i64,
    pub body: String,
    pub author: String,
    #[serde(default)]
    pub metadata: PrMetadata,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pr_state_serde_lowercase() {
        assert_eq!(serde_json::to_string(&PrState::Open).unwrap(), r#""open""#);
        assert_eq!(
            serde_json::to_string(&PrState::Merged).unwrap(),
            r#""merged""#
        );
        assert_eq!(
            serde_json::from_str::<PrState>(r#""closed""#).unwrap(),
            PrState::Closed
        );
    }

    #[test]
    fn pr_summary_roundtrip() {
        let summary = PullRequestSummary {
            id: "42".to_string(),
            number: 42,
            title: "✨ Add feature.".to_string(),
            state: PrState::Open,
            base: "master".to_string(),
            head: "feat/x".to_string(),
            url: "https://github.com/celestia-island/noa/pull/42".to_string(),
            created_at: 1_752_000_000,
        };
        let json = serde_json::to_string(&summary).unwrap();
        let back: PullRequestSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(back.title, "✨ Add feature.");
        assert_eq!(back.state, PrState::Open);
        assert_eq!(back.number, 42);
    }

    #[test]
    fn pr_detail_metadata_defaults_lenient() {
        let detail: PullRequestDetail = serde_json::from_str(
            r#"{
                "id": "1", "number": 1, "title": "t", "state": "open",
                "base": "master", "head": "feat/x", "url": "", "created_at": 0,
                "body": "b", "author": "noa"
            }"#,
        )
        .unwrap();
        assert_eq!(detail.metadata, PrMetadata::default());
    }

    #[test]
    fn pr_metadata_roundtrip() {
        let meta = PrMetadata {
            model: Some("deepseek/deepseek-chat".to_string()),
            input_tokens: Some(10),
            output_tokens: Some(20),
            cost_usd: Some(0.001),
        };
        let json = serde_json::to_string(&meta).unwrap();
        let back: PrMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(back, meta);
    }
}
