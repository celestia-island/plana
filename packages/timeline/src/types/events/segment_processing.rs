use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::{
    mcp_block::{McpBlockData, McpBlockState, is_wtv_or_wtvj},
    timeline_types::{
        GroupState, SkillBlockStatus, TimelineContentBlock, TimelineContentKind,
        TimelineSegmentBlock,
    },
};
use _text::StreamSegment;

#[derive(Debug, Clone)]
pub struct TimelineGroupData {
    pub agent_number: String,
    pub agent_type: String,
    pub timestamp: String,
    pub skill_name: Option<String>,
    pub model_name: Option<String>,
    pub provider_label: Option<String>,
    pub status: SkillBlockStatus,
    pub retry_count: usize,
    pub max_retries: usize,
    pub inherited_tokens: Option<super::super::metrics::TokenSource>,
    pub inherited_label: Option<String>,
    pub content_blocks: Vec<TimelineContentBlock>,
    pub mcp_blocks: Vec<McpBlockData>,
    pub interleaved_blocks: Vec<TimelineSegmentBlock>,
    pub stats: Option<super::super::metrics::GroupStats>,
    pub summary: Option<String>,
    pub result_summary: Option<String>,
    pub is_error: bool,
    pub state: GroupState,
    pub gray_tail_len: usize,
    pub retry_reason: Option<_state_sync::gateway::RetryReason>,
}

impl TimelineGroupData {
    pub fn segments_to_content_blocks(segments: &[StreamSegment]) -> Vec<TimelineContentBlock> {
        let mut blocks = Vec::new();
        for seg in segments {
            if seg.is_mcp() {
                continue;
            }
            let kind = match seg {
                StreamSegment::Thinking { .. } => TimelineContentKind::Thinking,
                StreamSegment::DeepThinking { .. } => TimelineContentKind::DeepThinking,
                _ => TimelineContentKind::Text,
            };
            let text = seg.text().trim().to_string();
            if !text.is_empty() {
                blocks.push(TimelineContentBlock { text, kind });
            }
        }
        blocks
    }

    /// Convert raw StreamSegments into a list of McpBlockData.
    ///
    /// Pipeline:
    /// ```mermaid
    /// flowchart LR
    ///   S["segments"] --> F["filter MCP-only"]
    ///   F --> P["pair Call+Result by (call_id, tool_name) at i+1"]
    ///   P --> D["dedup_mcp_blocks_by_content()"]
    ///   D --> O["downgrade_orphaned_pending_mcp()"]
    ///   O --> OUT["Vec&lt;McpBlockData&gt;"]
    /// ```
    ///
    /// Note: MCP-only filtering means non-adjacent synthetic results
    /// (appended at end by `close_pending_mcp_calls`) become adjacent
    /// here, so Call+Result pairing works correctly even when Text
    /// segments originally separated them in the full stream.
    pub fn segments_to_mcp_blocks(
        segments: &[StreamSegment],
        agent_type: &str,
    ) -> Vec<McpBlockData> {
        Self::segments_to_mcp_blocks_raw(segments, agent_type)
    }

    fn segments_to_mcp_blocks_raw(
        segments: &[StreamSegment],
        fallback_agent_type: &str,
    ) -> Vec<McpBlockData> {
        let resolve = |seg: &StreamSegment| -> String {
            seg.agent_type().unwrap_or(fallback_agent_type).to_string()
        };
        let mcp_segments: Vec<&StreamSegment> = segments.iter().filter(|s| s.is_mcp()).collect();
        let mut result_map: HashMap<Uuid, usize> = HashMap::new();
        for (j, seg) in mcp_segments.iter().enumerate() {
            if let StreamSegment::McpResult { call_id, .. } = seg {
                result_map.entry(*call_id).or_insert(j);
            }
        }
        let mut consumed: HashSet<usize> = HashSet::new();
        let mut blocks = Vec::new();
        let mut i = 0;
        while i < mcp_segments.len() {
            match mcp_segments[i] {
                StreamSegment::McpCall {
                    tool_name, call_id, ..
                } => {
                    let cid = *call_id;
                    let call_text = mcp_segments[i].text_or_json().trim().to_string();
                    let at = resolve(mcp_segments[i]);
                    let wtv = if is_wtv_or_wtvj(tool_name) {
                        McpBlockData::extract_wtv_content(&call_text)
                    } else {
                        Vec::new()
                    };

                    if let Some(&ri) = result_map.get(&cid) {
                        consumed.insert(ri);
                        let result_text = mcp_segments[ri].text_or_json().trim().to_string();
                        let (success, duration_ms) = if let StreamSegment::McpResult {
                            success: s,
                            duration_ms: d,
                            ..
                        } = mcp_segments[ri]
                        {
                            (*s, *d)
                        } else {
                            (true, None)
                        };
                        let at2 = resolve(mcp_segments[ri]);
                        let state = if success {
                            McpBlockState::Done
                        } else {
                            McpBlockState::Failed
                        };
                        blocks.push(McpBlockData {
                            tool_name: tool_name.clone(),
                            call_id: cid,
                            agent_type: if at2.is_empty() { at.clone() } else { at2 },
                            call_text,
                            result_text,
                            success,
                            duration_ms,
                            state,
                            separate_call_content: wtv,
                        });
                    } else {
                        let state = if call_text.is_empty() {
                            McpBlockState::Pending
                        } else {
                            McpBlockState::Running
                        };
                        blocks.push(McpBlockData {
                            tool_name: tool_name.clone(),
                            call_id: cid,
                            agent_type: at,
                            call_text,
                            result_text: String::new(),
                            success: true,
                            duration_ms: None,
                            state,
                            separate_call_content: wtv,
                        });
                    }
                }
                StreamSegment::McpResult { call_id, .. } => {
                    if consumed.contains(&i) {
                        i += 1;
                        continue;
                    }
                    let tool_name = mcp_segments[i].tool_name().unwrap_or("").to_string();
                    let (success, duration_ms) = if let StreamSegment::McpResult {
                        success: s,
                        duration_ms: d,
                        ..
                    } = mcp_segments[i]
                    {
                        (*s, *d)
                    } else {
                        (true, None)
                    };
                    let state = if success {
                        McpBlockState::Done
                    } else {
                        McpBlockState::Failed
                    };
                    blocks.push(McpBlockData {
                        tool_name: tool_name.clone(),
                        call_id: *call_id,
                        agent_type: resolve(mcp_segments[i]),
                        call_text: String::new(),
                        result_text: mcp_segments[i].text_or_json().trim().to_string(),
                        success,
                        duration_ms,
                        state,
                        separate_call_content: Vec::new(),
                    });
                }
                _ => {}
            }
            i += 1;
        }
        Self::dedup_mcp_blocks_by_content(&mut blocks);
        Self::downgrade_orphaned_pending_mcp(&mut blocks);
        blocks
    }

    /// Remove duplicate McpBlockData sharing the same (tool_name, call_text).
    /// Same reverse-scan keep-last algorithm as `dedup_interleaved_by_content`
    /// but operates on `Vec<McpBlockData>` (identified by call_id, not index).
    ///
    /// Used cross-sub after IndexMap call_id dedup in groups.rs.
    pub fn dedup_mcp_blocks_by_content(blocks: &mut Vec<McpBlockData>) {
        let mut seen: std::collections::HashSet<(String, String)> =
            std::collections::HashSet::new();
        let mut dup_ids: Vec<Uuid> = Vec::new();
        for block in blocks.iter().rev() {
            let key = (block.tool_name.clone(), block.call_text.clone());
            if !seen.insert(key) {
                dup_ids.push(block.call_id);
            }
        }
        if dup_ids.is_empty() {
            return;
        }
        let dup_set: std::collections::HashSet<Uuid> = dup_ids.into_iter().collect();
        blocks.retain(|b| !dup_set.contains(&b.call_id));
    }

    pub(super) fn downgrade_orphaned_pending_mcp(blocks: &mut [McpBlockData]) {
        let mut has_subsequent = false;
        let mut orphaned_ids: Vec<Uuid> = Vec::new();
        for block in blocks.iter().rev() {
            let is_incomplete =
                matches!(block.state, McpBlockState::Pending | McpBlockState::Running)
                    && block.result_text.is_empty();
            if has_subsequent && is_incomplete {
                orphaned_ids.push(block.call_id);
            }
            if !is_incomplete {
                has_subsequent = true;
            }
        }
        let orphan_set: std::collections::HashSet<Uuid> = orphaned_ids.into_iter().collect();
        for block in blocks.iter_mut() {
            if orphan_set.contains(&block.call_id) {
                block.state = McpBlockState::HistoryLost;
            }
        }
    }

    /// Convert raw StreamSegments into an interleaved sequence of Content
    /// and Mcp blocks. Content blocks carry LLM text; Mcp blocks carry
    /// structured tool-call data.
    ///
    /// Full pipeline (per sub-report):
    /// ```mermaid
    /// flowchart TD
    ///   IN["segments (sealed LlmStream)"]
    ///   IN --> PRE["pre-scan: build call_id → McpResult index map"]
    ///   PRE --> RAW["segments_to_interleaved_raw()<br/>① pre-scan: HashMap call_id → McpResult index<br/>② walk segments, pair Call+Result via UUID lookup<br/>③ unpaired Call → Pending; lone Result → Failed"]
    ///   RAW --> DEDUP["dedup_interleaved_by_content()<br/>reverse scan by (tool_name, call_text), remove earlier duplicates"]
    ///   DEDUP --> ORPHAN["downgrade_orphaned_pending_interleaved()<br/>reverse scan: Pending+empty_result with substantive successor → Mcp(HistoryLost)"]
    ///   ORPHAN --> CID["call_id dedup (keep-last via reverse scan)<br/>defensive — sealed subs have unique call_ids"]
    ///   CID --> OUT["Vec&lt;TimelineSegmentBlock&gt;"]
    /// ```
    ///
    /// UUIDv7 pairing: call_id (UUIDv7, generated at tool_exec.rs:52) is
    /// shared between McpCall and McpResult segments. The pre-scan builds
    /// a HashMap<Uuid, usize> so pairing works regardless of segment
    /// adjacency — batch ordering, intermixed text, or synthetic results
    /// appended by seal_coalesced() all resolve correctly.
    ///
    /// Order constraint: dedup MUST run before orphan downgrade.
    /// If reversed, a duplicate Pending block would first become HistoryLost
    /// Mcp, then dedup could not remove it because HistoryLost blocks have
    /// a different state than expected — producing redundant output.
    pub fn segments_to_interleaved(
        segments: &[StreamSegment],
        agent_type: &str,
        streaming: bool,
    ) -> Vec<TimelineSegmentBlock> {
        let raw_blocks = Self::segments_to_interleaved_raw(segments, agent_type, streaming);
        let mut seen_call_id: HashSet<Uuid> = HashSet::new();
        let mut result = Vec::new();

        for block in raw_blocks.into_iter().rev() {
            match &block {
                TimelineSegmentBlock::Mcp(mcp) => {
                    if seen_call_id.insert(mcp.call_id) {
                        result.push(block);
                    }
                }
                _ => {
                    result.push(block);
                }
            }
        }

        result.reverse();
        result
    }

    fn segments_to_interleaved_raw(
        segments: &[StreamSegment],
        fallback_agent_type: &str,
        streaming: bool,
    ) -> Vec<TimelineSegmentBlock> {
        let resolve = |seg: &StreamSegment| -> String {
            seg.agent_type().unwrap_or(fallback_agent_type).to_string()
        };

        let mut result_map: HashMap<Uuid, usize> = HashMap::new();
        for (j, seg) in segments.iter().enumerate() {
            if let StreamSegment::McpResult { call_id, .. } = seg {
                result_map.entry(*call_id).or_insert(j);
            }
        }
        let mut consumed: HashSet<usize> = HashSet::new();

        let mut blocks = Vec::new();
        let mut i = 0;
        while i < segments.len() {
            match &segments[i] {
                StreamSegment::McpCall {
                    tool_name, call_id, ..
                } => {
                    let cid = *call_id;
                    let call_text = segments[i].text_or_json().trim().to_string();
                    let at = resolve(&segments[i]);
                    let wtv = if is_wtv_or_wtvj(tool_name) {
                        McpBlockData::extract_wtv_content(&call_text)
                    } else {
                        Vec::new()
                    };

                    if let Some(&ri) = result_map.get(&cid) {
                        consumed.insert(ri);
                        let result_text = segments[ri].text_or_json().trim().to_string();
                        let (success, duration_ms, at2) = if let StreamSegment::McpResult {
                            success: s,
                            duration_ms: d,
                            ..
                        } = &segments[ri]
                        {
                            (*s, *d, resolve(&segments[ri]))
                        } else {
                            (true, None, String::new())
                        };
                        let state = if success {
                            McpBlockState::Done
                        } else {
                            McpBlockState::Failed
                        };
                        blocks.push(TimelineSegmentBlock::Mcp(McpBlockData {
                            tool_name: tool_name.clone(),
                            call_id: cid,
                            agent_type: if at2.is_empty() { at.clone() } else { at2 },
                            call_text,
                            result_text,
                            success,
                            duration_ms,
                            state,
                            separate_call_content: wtv,
                        }));
                    } else {
                        let state = if call_text.is_empty() {
                            McpBlockState::Pending
                        } else {
                            McpBlockState::Running
                        };
                        blocks.push(TimelineSegmentBlock::Mcp(McpBlockData {
                            tool_name: tool_name.clone(),
                            call_id: cid,
                            agent_type: at,
                            call_text,
                            result_text: String::new(),
                            success: true,
                            duration_ms: None,
                            state,
                            separate_call_content: wtv,
                        }));
                    }
                    i += 1;
                }
                StreamSegment::McpResult { call_id, .. } => {
                    if consumed.contains(&i) {
                        i += 1;
                        continue;
                    }
                    let tool_name = segments[i].tool_name().unwrap_or("").to_string();
                    let (success, duration_ms) = if let StreamSegment::McpResult {
                        success: s,
                        duration_ms: d,
                        ..
                    } = &segments[i]
                    {
                        (*s, *d)
                    } else {
                        (true, None)
                    };
                    let state = if success {
                        McpBlockState::Done
                    } else {
                        McpBlockState::Failed
                    };
                    blocks.push(TimelineSegmentBlock::Mcp(McpBlockData {
                        tool_name: tool_name.clone(),
                        call_id: *call_id,
                        agent_type: resolve(&segments[i]),
                        call_text: String::new(),
                        result_text: segments[i].text_or_json().trim().to_string(),
                        success,
                        duration_ms,
                        state,
                        separate_call_content: Vec::new(),
                    }));
                    i += 1;
                }
                _ => {
                    let kind = match &segments[i] {
                        StreamSegment::Thinking { .. } => TimelineContentKind::Thinking,
                        StreamSegment::DeepThinking { .. } => TimelineContentKind::DeepThinking,
                        _ => TimelineContentKind::Text,
                    };
                    let text = segments[i].text().trim().to_string();
                    if !text.is_empty() {
                        blocks.push(TimelineSegmentBlock::Content(TimelineContentBlock {
                            text,
                            kind,
                        }));
                    }
                    i += 1;
                }
            }
        }
        Self::dedup_interleaved_by_content(&mut blocks);
        if !streaming {
            Self::downgrade_orphaned_pending_interleaved(&mut blocks);
        }
        blocks
    }

    /// Remove duplicate Mcp blocks that share the same (tool_name, call_text)
    /// across retries, keeping only the **last** chronological occurrence.
    /// Content blocks are never touched.
    ///
    /// Algorithm (reverse scan, keep-last):
    /// ```mermaid
    /// flowchart LR
    ///   IN["blocks"] --> REV["reverse iteration"]
    ///   REV --> K["key = (tool_name, call_text)"]
    ///   K --> FIRST["first encounter → seen.insert → KEEP"]
    ///   K --> DUP["already in seen → dup_indices.push(rev_idx)"]
    ///   FIRST --> OUT
    ///   DUP --> REM["drain(..).enumerate() → skip dup indices"]
    ///   REM --> OUT["filtered blocks"]
    /// ```
    ///
    /// Used both per-sub (within segments_to_interleaved_raw) and
    /// cross-sub (after concatenation in groups.rs).
    pub fn dedup_interleaved_by_content(blocks: &mut Vec<TimelineSegmentBlock>) {
        let mut seen: std::collections::HashSet<(String, String)> =
            std::collections::HashSet::new();
        let mut dup_indices: Vec<usize> = Vec::new();
        for (idx, block) in blocks.iter().rev().enumerate() {
            let rev_idx = blocks.len() - 1 - idx;
            if let TimelineSegmentBlock::Mcp(mcp) = block {
                let key = (mcp.tool_name.clone(), mcp.call_text.clone());
                if !seen.insert(key) {
                    dup_indices.push(rev_idx);
                }
            }
        }
        if dup_indices.is_empty() {
            return;
        }
        let dup_set: std::collections::HashSet<usize> = dup_indices.into_iter().collect();
        let mut result = Vec::with_capacity(blocks.len());
        for (idx, block) in blocks.drain(..).enumerate() {
            if !dup_set.contains(&idx) {
                result.push(block);
            }
        }
        *blocks = result;
    }

    pub(super) fn downgrade_orphaned_pending_interleaved(blocks: &mut [TimelineSegmentBlock]) {
        let mut has_subsequent = false;
        let mut orphan_indices: Vec<usize> = Vec::new();
        for (idx, block) in blocks.iter().rev().enumerate() {
            let rev_idx = blocks.len() - 1 - idx;
            if has_subsequent && let TimelineSegmentBlock::Mcp(mcp) = block {
                let is_incomplete =
                    matches!(mcp.state, McpBlockState::Pending | McpBlockState::Running)
                        && mcp.result_text.is_empty();
                if is_incomplete {
                    orphan_indices.push(rev_idx);
                }
            }
            let is_substantive = match block {
                TimelineSegmentBlock::Content(_) => true,
                TimelineSegmentBlock::Mcp(mcp) => {
                    !matches!(mcp.state, McpBlockState::Pending | McpBlockState::Running)
                        || !mcp.result_text.is_empty()
                }
            };
            if is_substantive {
                has_subsequent = true;
            }
        }
        let orphan_set: std::collections::HashSet<usize> = orphan_indices.into_iter().collect();
        for (idx, block) in blocks.iter_mut().enumerate() {
            if orphan_set.contains(&idx)
                && let TimelineSegmentBlock::Mcp(mcp) = block
            {
                mcp.state = McpBlockState::HistoryLost;
            }
        }
    }
}
