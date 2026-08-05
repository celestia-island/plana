use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{StreamChunkKind, StreamSegment};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmStream {
    #[serde(default)]
    pub(crate) segments: Vec<StreamSegment>,
}

impl LlmStream {
    pub fn empty() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn from_plain_text(text: &str) -> Self {
        let segments = if text.is_empty() {
            Vec::new()
        } else {
            vec![StreamSegment::Text {
                text: text.to_string(),
                message_id: None,
            }]
        };
        Self { segments }
    }

    pub fn raw_text(&self) -> String {
        self.segments.iter().map(|s| s.text_or_json()).collect()
    }

    pub fn segments(&self) -> &[StreamSegment] {
        &self.segments
    }

    pub fn segments_mut(&mut self) -> &mut [StreamSegment] {
        &mut self.segments
    }

    pub fn patch_mcp_result_for_call(
        &mut self,
        call_id: Uuid,
        result_data: serde_json::Value,
        success: bool,
        duration_ms: Option<u64>,
    ) -> bool {
        let sealed_marker = "[stream sealed before result arrived]";
        let sealed_value = serde_json::Value::String(sealed_marker.to_string());

        for seg in &mut self.segments {
            if let StreamSegment::McpResult {
                call_id: seg_call_id,
                data,
                success: seg_success,
                duration_ms: seg_duration,
                ..
            } = seg
                && *seg_call_id == call_id
                && *data == sealed_value
            {
                *data = result_data;
                *seg_success = success;
                *seg_duration = duration_ms;
                return true;
            }
        }

        let has_call = self
            .segments
            .iter()
            .any(|s| matches!(s, StreamSegment::McpCall { call_id: cid, .. } if *cid == call_id));
        if has_call {
            let tool_name = self
                .segments
                .iter()
                .find_map(|s| match s {
                    StreamSegment::McpCall {
                        call_id: cid,
                        tool_name,
                        ..
                    } if *cid == call_id => Some(tool_name.clone()),
                    _ => None,
                })
                .unwrap_or_default();
            let agent_type = self
                .segments
                .iter()
                .find_map(|s| match s {
                    StreamSegment::McpCall {
                        call_id: cid,
                        agent_type,
                        ..
                    } if *cid == call_id => Some(agent_type.clone()),
                    _ => None,
                })
                .flatten();

            let pos = self
                .segments
                .iter()
                .rposition(
                    |s| matches!(s, StreamSegment::McpCall { call_id: cid, .. } if *cid == call_id),
                )
                .map(|i| i + 1)
                .unwrap_or(self.segments.len());
            self.segments.insert(
                pos,
                StreamSegment::McpResult {
                    tool_name,
                    call_id,
                    success,
                    data: result_data,
                    duration_ms,
                    agent_type,
                    message_id: None,
                },
            );
            return true;
        }

        false
    }

    pub fn has_mcp_call_with_id(&self, call_id: Uuid) -> bool {
        self.segments
            .iter()
            .any(|s| matches!(s, StreamSegment::McpCall { call_id: cid, .. } if *cid == call_id))
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
            || self.segments.iter().all(|s| match s {
                StreamSegment::Text { text, .. }
                | StreamSegment::Thinking { text, .. }
                | StreamSegment::DeepThinking { text, .. } => text.is_empty(),
                StreamSegment::McpCall { params, .. } => params.is_null(),
                StreamSegment::McpResult { data, .. } => data.is_null(),
                StreamSegment::AudioPcm { .. }
                | StreamSegment::VideoFrame { .. }
                | StreamSegment::ImagePartial { .. } => false,
            })
    }

    pub fn text_segments(&self) -> impl Iterator<Item = (StreamChunkKind, &str)> {
        self.segments.iter().filter_map(|seg| {
            let kind = seg.chunk_kind()?;
            Some((kind, seg.text()))
        })
    }

    pub fn mcp_segments(&self) -> impl Iterator<Item = &StreamSegment> {
        self.segments.iter().filter(|s| s.is_mcp())
    }

    pub fn thinking_text(&self) -> String {
        self.text_segments()
            .filter(|(kind, _)| {
                matches!(
                    kind,
                    StreamChunkKind::Thinking | StreamChunkKind::DeepThinking
                )
            })
            .map(|(_, text)| text)
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn display_text(&self) -> String {
        self.text_segments()
            .filter(|(kind, _)| matches!(kind, StreamChunkKind::Text))
            .map(|(_, text)| text)
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn len(&self) -> usize {
        self.segments.iter().map(|s| s.text_or_json().len()).sum()
    }
}

impl std::fmt::Display for LlmStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for seg in &self.segments {
            f.write_str(&seg.text_or_json())?;
        }
        Ok(())
    }
}

impl Default for LlmStream {
    fn default() -> Self {
        Self::empty()
    }
}

impl From<String> for LlmStream {
    fn from(s: String) -> Self {
        Self::from_plain_text(&s)
    }
}

impl From<&str> for LlmStream {
    fn from(s: &str) -> Self {
        Self::from_plain_text(s)
    }
}

impl PartialEq for LlmStream {
    fn eq(&self, other: &Self) -> bool {
        self.segments == other.segments
    }
}

impl Eq for LlmStream {}
