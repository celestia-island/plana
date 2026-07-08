use anyhow::{Context, Result, bail};
use uuid::Uuid;

use super::*;
use arona_core::var_namespace;
use arona_text::StreamSegment;

fn txt(text: &str) -> StreamSegment {
    StreamSegment::Text {
        text: text.to_string(),
        message_id: None,
    }
}

fn mcp_call(tool: &str, call_id: Uuid, text: &str) -> StreamSegment {
    let params = if text.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_str::<serde_json::Value>(text)
            .unwrap_or(serde_json::Value::String(text.to_string()))
    };
    StreamSegment::McpCall {
        tool_name: tool.to_string(),
        call_id,
        params,
        agent_type: Some("haplotes".to_string()),
        message_id: None,
    }
}

fn mcp_call_empty(tool: &str, call_id: Uuid) -> StreamSegment {
    mcp_call(tool, call_id, "")
}

fn mcp_result(
    tool: &str,
    call_id: Uuid,
    result: &str,
    success: bool,
    dur_ms: Option<u64>,
) -> StreamSegment {
    let data = if result.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_str::<serde_json::Value>(result)
            .unwrap_or(serde_json::Value::String(result.to_string()))
    };
    StreamSegment::McpResult {
        tool_name: tool.to_string(),
        call_id,
        success,
        data,
        duration_ms: dur_ms,
        agent_type: Some("haplotes".to_string()),
        message_id: None,
    }
}

// ═══════════════════════════════════════════════════
//  Unit: downgrade_orphaned_pending_mcp  (mcp_blocks path)
//  Constraint: return type is Vec<McpBlockData>, cannot
//  produce Content. Best degradation = mark HistoryLost.
//  Nothing is ever deleted.
// ═══════════════════════════════════════════════════

#[test]
fn unit_mcp_empty_pending_before_done_becomes_done() -> Result<()> {
    let mut blocks = vec![
        McpBlockData {
            tool_name: "a".to_string(),
            call_id: Uuid::now_v7(),
            agent_type: "t".to_string(),
            call_text: String::new(),
            result_text: String::new(),
            success: true,
            duration_ms: None,
            state: McpBlockState::Pending,
            separate_call_content: Vec::new(),
        },
        McpBlockData {
            tool_name: "b".to_string(),
            call_id: Uuid::now_v7(),
            agent_type: "t".to_string(),
            call_text: "params".to_string(),
            result_text: "ok".to_string(),
            success: true,
            duration_ms: Some(100),
            state: McpBlockState::Done,
            separate_call_content: Vec::new(),
        },
    ];
    TimelineGroupData::downgrade_orphaned_pending_mcp(&mut blocks);
    assert_eq!(blocks.len(), 2, "nothing deleted");
    assert_eq!(
        blocks[0].state,
        McpBlockState::HistoryLost,
        "orphaned Running degraded to HistoryLost"
    );
    assert_eq!(blocks[1].state, McpBlockState::Done);
    Ok(())
}

#[test]
fn unit_mcp_running_no_result_before_done_becomes_done() -> Result<()> {
    let mut blocks = vec![
        McpBlockData {
            tool_name: "a".to_string(),
            call_id: Uuid::now_v7(),
            agent_type: "t".to_string(),
            call_text: "partial code here".to_string(),
            result_text: String::new(),
            success: true,
            duration_ms: None,
            state: McpBlockState::Running,
            separate_call_content: Vec::new(),
        },
        McpBlockData {
            tool_name: "b".to_string(),
            call_id: Uuid::now_v7(),
            agent_type: "t".to_string(),
            call_text: "{}".to_string(),
            result_text: "done".to_string(),
            success: true,
            duration_ms: Some(50),
            state: McpBlockState::Done,
            separate_call_content: Vec::new(),
        },
    ];
    TimelineGroupData::downgrade_orphaned_pending_mcp(&mut blocks);
    assert_eq!(blocks.len(), 2, "nothing deleted");
    assert_eq!(blocks[0].state, McpBlockState::HistoryLost);
    Ok(())
}

#[test]
fn unit_mcp_trailing_incomplete_never_touched() -> Result<()> {
    let mut blocks = vec![
        McpBlockData {
            tool_name: "complete".to_string(),
            call_id: Uuid::now_v7(),
            agent_type: "t".to_string(),
            call_text: "x".to_string(),
            result_text: "y".to_string(),
            success: true,
            duration_ms: Some(10),
            state: McpBlockState::Done,
            separate_call_content: Vec::new(),
        },
        McpBlockData {
            tool_name: "orphan".to_string(),
            call_id: Uuid::now_v7(),
            agent_type: "t".to_string(),
            call_text: String::new(),
            result_text: String::new(),
            success: true,
            duration_ms: None,
            state: McpBlockState::Pending,
            separate_call_content: Vec::new(),
        },
    ];
    TimelineGroupData::downgrade_orphaned_pending_mcp(&mut blocks);
    assert_eq!(blocks.len(), 2);
    assert_eq!(
        blocks[1].state,
        McpBlockState::Pending,
        "trailing stays Pending"
    );
    Ok(())
}

#[test]
fn unit_mcp_all_incomplete_chain_preserved_as_is() -> Result<()> {
    let mut blocks = vec![
        McpBlockData {
            tool_name: "p1".to_string(),
            call_id: Uuid::now_v7(),
            agent_type: "t".to_string(),
            call_text: String::new(),
            result_text: String::new(),
            success: true,
            duration_ms: None,
            state: McpBlockState::Pending,
            separate_call_content: Vec::new(),
        },
        McpBlockData {
            tool_name: "r1".to_string(),
            call_id: Uuid::now_v7(),
            agent_type: "t".to_string(),
            call_text: "partial".to_string(),
            result_text: String::new(),
            success: true,
            duration_ms: None,
            state: McpBlockState::Running,
            separate_call_content: Vec::new(),
        },
    ];
    TimelineGroupData::downgrade_orphaned_pending_mcp(&mut blocks);
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].state, McpBlockState::Pending);
    assert_eq!(blocks[1].state, McpBlockState::Running);
    Ok(())
}

// ═══════════════════════════════════════════════════
//  Unit: downgrade_orphaned_pending_interleaved
//  All orphans → Mcp(HistoryLost). Never delete.
// ═══════════════════════════════════════════════════

#[test]
fn unit_interleaved_empty_pending_becomes_empty_content() -> Result<()> {
    let mut blocks: Vec<TimelineSegmentBlock> = vec![
        TimelineSegmentBlock::Mcp(McpBlockData {
            tool_name: "ghost".to_string(),
            call_id: Uuid::now_v7(),
            agent_type: "t".to_string(),
            call_text: String::new(),
            result_text: String::new(),
            success: true,
            duration_ms: None,
            state: McpBlockState::Pending,
            separate_call_content: Vec::new(),
        }),
        TimelineSegmentBlock::Content(TimelineContentBlock {
            text: "real content after".to_string(),
            kind: TimelineContentKind::Text,
        }),
    ];
    TimelineGroupData::downgrade_orphaned_pending_interleaved(&mut blocks);
    assert_eq!(
        blocks.len(),
        2,
        "nothing deleted; empty orphan stays Mcp with HistoryLost"
    );
    let m = match &blocks[0] {
        TimelineSegmentBlock::Mcp(m) => m,
        _ => bail!("expected Mcp"),
    };
    assert_eq!(
        m.state,
        McpBlockState::HistoryLost,
        "empty Pending → Mcp(HistoryLost)"
    );
    assert!(m.call_text.is_empty(), "call_text still empty");
    matches!(&blocks[1], TimelineSegmentBlock::Content(c) if c.text == "real content after");
    Ok(())
}

#[test]
fn unit_interleaved_partial_running_degrades_to_content_with_text() -> Result<()> {
    let partial_code = &format!(
        "orexis.report_human({{ summary: {} }})",
        var_namespace::ref_bracket("reply_summary")
    );
    let mut blocks: Vec<TimelineSegmentBlock> = vec![
        TimelineSegmentBlock::Mcp(McpBlockData {
            tool_name: "exec".to_string(),
            call_id: Uuid::now_v7(),
            agent_type: "t".to_string(),
            call_text: partial_code.to_string(),
            result_text: String::new(),
            success: true,
            duration_ms: None,
            state: McpBlockState::Running,
            separate_call_content: Vec::new(),
        }),
        TimelineSegmentBlock::Content(TimelineContentBlock {
            text: "next line of thinking".to_string(),
            kind: TimelineContentKind::Text,
        }),
    ];
    TimelineGroupData::downgrade_orphaned_pending_interleaved(&mut blocks);
    assert_eq!(blocks.len(), 2, "nothing deleted");
    let m = match &blocks[0] {
        TimelineSegmentBlock::Mcp(m) => m,
        _ => bail!("expected degraded Mcp with HistoryLost state"),
    };
    assert_eq!(
        m.state,
        McpBlockState::HistoryLost,
        "orphan → Mcp(HistoryLost)"
    );
    assert_eq!(
        m.call_text,
        partial_code.as_str(),
        "partial call text preserved in Mcp block"
    );
    Ok(())
}

// ═══════════════════════════════════════════════════
//  Integration: real StreamSegment → full pipeline
// ═══════════════════════════════════════════════════

/// Text → McpCall(empty/Pending) → Text → McpCall(complete) → McpResult(Done)
/// The orphaned empty Pending in the middle degrades.
/// In interleaved path it becomes Mcp(HistoryLost); in mcp_blocks it becomes HistoryLost.
#[test]
fn integration_empty_pending_sandwiched_between_content() -> Result<()> {
    let id_empty = Uuid::now_v7();
    let id_complete = Uuid::now_v7();
    let segments = vec![
        txt("I see – the Chinese characters are causing issues."),
        mcp_call_empty("exec", id_empty),
        txt("Let me encode as JSON first."),
        mcp_call("exec", id_complete, r#"{"code": "payload()"}"#),
        mcp_result("exec", id_complete, "ok", true, Some(3)),
    ];

    let interleaved = TimelineGroupData::segments_to_interleaved(&segments, "agent", false);

    assert_eq!(
        interleaved.len(),
        4,
        "4 blocks: text + orphan→Mcp(HistoryLost) + text + done-mcp (call+result merge into 1)"
    );

    let content_texts: Vec<&str> = interleaved
        .iter()
        .filter_map(|b| match b {
            TimelineSegmentBlock::Content(c) => Some(c.text.as_str()),
            _ => None,
        })
        .collect();

    assert!(content_texts_contain(&content_texts, "Chinese characters"));
    assert!(content_texts_contain(&content_texts, "encode as JSON"));

    let mcp_count = interleaved
        .iter()
        .filter(|b| matches!(b, TimelineSegmentBlock::Mcp(_)))
        .count();
    assert_eq!(
        mcp_count, 2,
        "orphan stays Mcp(HistoryLost) + complete exec Mcp(Done)"
    );
    Ok(())
}

/// Unclosed/partial McpCall immediately before a complete pair.
/// Partial has non-empty call_text but no result → Running+no_result → orphan → Mcp(HistoryLost).
#[test]
fn integration_unclosed_call_before_complete_pair() {
    let id_partial = Uuid::now_v7();
    let id_ok = Uuid::now_v7();
    let partial_code = &format!(
        "orexis.report_human({{ summary: {}, mode:",
        var_namespace::ref_bracket("reply_summary")
    );
    let segments = vec![
        txt("Thinking about how to report..."),
        mcp_call("exec", id_partial, partial_code),
        mcp_call(
            "exec",
            id_ok,
            &format!(
                r#"orexis.report_human({})"#,
                var_namespace::ref_bracket("reply_payload")
            ),
        ),
        mcp_result("exec", id_ok, "reported ok", true, Some(5)),
        txt("Report sent successfully."),
    ];

    let interleaved = TimelineGroupData::segments_to_interleaved(&segments, "agent", false);

    assert_eq!(
        interleaved.len(),
        4,
        "4 blocks: text + orphan→Mcp(HistoryLost) + done-mcp(merged) + text"
    );

    let content_blocks: Vec<&str> = interleaved
        .iter()
        .filter_map(|b| match b {
            TimelineSegmentBlock::Content(c) => Some(c.text.as_str()),
            _ => None,
        })
        .collect();

    assert!(content_texts_contain(&content_blocks, "Thinking about"));
    assert!(content_texts_contain(&content_blocks, "successfully"));

    let orphan_history_lost = interleaved.iter().any(|b| match b {
        TimelineSegmentBlock::Mcp(m) => {
            m.state == McpBlockState::HistoryLost && m.call_text == *partial_code
        },
        _ => false,
    });
    assert!(
        orphan_history_lost,
        "partial orphan stays Mcp with HistoryLost state and original call_text"
    );

    let done_mcps: Vec<_> = interleaved
        .iter()
        .filter_map(|b| match b {
            TimelineSegmentBlock::Mcp(m) if m.state == McpBlockState::Done => Some(&m.tool_name),
            _ => None,
        })
        .collect();
    assert_eq!(done_mcps.len(), 1);
}

/// Multiple orphans then one complete. All orphans degrade, none deleted.
#[test]
fn integration_multiple_orphans_then_complete() {
    let id_p1 = Uuid::now_v7();
    let id_p2 = Uuid::now_v7();
    let id_ok = Uuid::now_v7();
    let segments = vec![
        mcp_call_empty("report", id_p1),
        mcp_call(
            "exec",
            id_p2,
            "import { report } from 'hubris'; report({content: 'incomplete",
        ),
        mcp_call("report", id_ok, r#"{"content": "full report"}"#),
        mcp_result("report", id_ok, "accepted", true, Some(10)),
    ];

    let mcps = TimelineGroupData::segments_to_mcp_blocks(&segments, "agent");
    assert_eq!(
        mcps.len(),
        3,
        "3 McpBlocks total (2 orphans→HistoryLost + 1 real Done), none deleted"
    );
    assert_eq!(
        mcps[0].state,
        McpBlockState::HistoryLost,
        "empty orphan → HistoryLost"
    );
    assert_eq!(
        mcps[1].state,
        McpBlockState::HistoryLost,
        "partial orphan → HistoryLost"
    );
    assert_eq!(
        mcps[2].state,
        McpBlockState::Done,
        "real complete block → Done"
    );

    let interleaved = TimelineGroupData::segments_to_interleaved(&segments, "agent", false);
    assert_eq!(
        interleaved.len(),
        3,
        "3 items: 2 orphans→Mcp(HistoryLost) + 1 done-Mcp(merged)"
    );
    let content_count = interleaved
        .iter()
        .filter(|b| matches!(b, TimelineSegmentBlock::Content(_)))
        .count();
    assert_eq!(
        content_count, 0,
        "orphans stay Mcp(HistoryLost), not converted to Content"
    );
}

/// Trailing incomplete at end of stream — must NOT be touched.
#[test]
fn integration_trailing_incomplete_untouched() -> Result<()> {
    let id_p1 = Uuid::now_v7();
    let id_done = Uuid::now_v7();
    let segments = vec![
        mcp_call("report", id_done, r#"{"msg":"done"}"#),
        mcp_result("report", id_done, "ok", true, Some(1)),
        mcp_call_empty("exec", id_p1),
    ];

    let mcps = TimelineGroupData::segments_to_mcp_blocks(&segments, "agent");
    assert_eq!(mcps.len(), 2);
    assert_eq!(mcps[0].state, McpBlockState::Done);
    assert_eq!(
        mcps[1].state,
        McpBlockState::Pending,
        "trailing Pending untouched"
    );
    Ok(())
}

/// Full realistic scenario from the bug screenshot:
/// WTV set → **empty exec(Pending)** → WTV set → exec(error) → thinking text →
/// WTV JSON set → **unclosed exec(partial)** → exec(complete)
///
/// Orphans marked: ghost=empty Pending between WTVs, partial=unclosed before final exec.
/// In interleaved: both stay Mcp(HistoryLost). In mcp_blocks: both become HistoryLost.
#[test]
fn integration_realistic_truncated_skill_stream() -> Result<()> {
    let id_wtv1 = Uuid::now_v7();
    let id_ghost = Uuid::now_v7();
    let id_wtv2 = Uuid::now_v7();
    let id_exec_err = Uuid::now_v7();
    let id_wtv_json = Uuid::now_v7();
    let id_partial = Uuid::now_v7();
    let id_final = Uuid::now_v7();

    let segments = vec![
        mcp_call(
            "write_to_var",
            id_wtv1,
            r#"{"var_name":"reply_summary","value":"I am Entelecheia..."}"#,
        ),
        mcp_result("write_to_var", id_wtv1, "set ok", true, Some(5)),
        mcp_call_empty("exec", id_ghost),
        mcp_call(
            "write_to_var",
            id_wtv2,
            r#"{"var_name":"reply_mode","value":"reply"}"#,
        ),
        mcp_result("write_to_var", id_wtv2, "set ok", true, Some(1)),
        mcp_call("exec", id_exec_err, {
            let summary_ref = var_namespace::ref_bracket("reply_summary");
            let call_text = format!(
                r#"{{"code":"orexis.report_human({{summary:{} }})}}"#,
                summary_ref
            );
            call_text.leak() as &str
        }),
        mcp_result(
            "exec",
            id_exec_err,
            "Tool 'exec' failed: SyntaxError: abrupt end",
            false,
            Some(25),
        ),
        txt("I see – Chinese chars causing sandbox parser fail. Let me encode as JSON first."),
        mcp_call(
            "write_to_var_json",
            id_wtv_json,
            r#"{"var_name":"reply_payload","value":{"key":"val"}}"#,
        ),
        mcp_result(
            "write_to_var_json",
            id_wtv_json,
            "parsed JSON ok",
            true,
            Some(3),
        ),
        mcp_call(
            "exec",
            id_partial,
            &format!(
                r#"{{"code":"orexis.report_human({})}}"#,
                var_namespace::ref_bracket("reply_payload")
            ),
        ),
        mcp_call("exec", id_final, r#"{"code":"console.log('done')"}"#),
        mcp_result("exec", id_final, "undefined", true, Some(2)),
    ];

    let interleaved = TimelineGroupData::segments_to_interleaved(&segments, "skill_agent", false);

    let total = interleaved.len();
    assert_eq!(
        total, 8,
        "13 segments → 8 blocks (5 call+result merge into Mcp + 1 text + 2 orphans→Mcp(HistoryLost)); nothing deleted"
    );

    let mcp_blocks: Vec<_> = interleaved
        .iter()
        .filter_map(|b| match b {
            TimelineSegmentBlock::Mcp(m) => Some((&m.tool_name, m.state)),
            _ => None,
        })
        .collect();

    let pending_count = mcp_blocks
        .iter()
        .filter(|(_, s)| *s == McpBlockState::Pending)
        .count();
    assert_eq!(
        pending_count, 0,
        "no Pending left — ghost orphan gone from Mcp list"
    );

    let partial_as_history_lost = interleaved.iter().any(|b| match b {
        TimelineSegmentBlock::Mcp(m) => {
            m.state == McpBlockState::HistoryLost
                && m.call_text.contains(&format!(
                    "orexis.report_human({})",
                    var_namespace::ref_bracket("reply_payload")
                ))
        },
        _ => false,
    });
    assert!(
        partial_as_history_lost,
        "unclosed partial exec stays Mcp with HistoryLost state"
    );

    let ghost_as_history_lost = interleaved
            .iter()
            .any(|b| matches!(b, TimelineSegmentBlock::Mcp(m) if m.state == McpBlockState::HistoryLost && m.call_text.is_empty()));
    assert!(
        ghost_as_history_lost,
        "empty ghost exec stays Mcp with HistoryLost state (not deleted)"
    );

    let has_final_done = mcp_blocks
        .iter()
        .any(|(t, s)| *t == "exec" && *s == McpBlockState::Done);
    assert!(
        has_final_done,
        "final complete exec still a proper Done Mcp"
    );
    Ok(())
}

// ═════════════════════════════════════════════════
//  Content-based dedup: same (tool_name, call_text)
//  across retries should collapse to last occurrence.
// ═════════════════════════════════════════════════

#[test]
fn unit_dedup_mcp_by_content_removes_earlier_duplicate() -> Result<()> {
    let id_a = Uuid::now_v7();
    let id_b = Uuid::now_v7();
    let id_c = Uuid::now_v7();
    let code = r#"orexis.report_human({summary:"hello"})"#;
    let mut blocks: Vec<TimelineSegmentBlock> = vec![
        TimelineSegmentBlock::Mcp(McpBlockData {
            tool_name: "exec".to_string(),
            call_id: id_a,
            agent_type: "t".to_string(),
            call_text: code.to_string(),
            result_text: "error 1".to_string(),
            success: false,
            duration_ms: Some(10),
            state: McpBlockState::Failed,
            separate_call_content: Vec::new(),
        }),
        TimelineSegmentBlock::Mcp(McpBlockData {
            tool_name: "exec".to_string(),
            call_id: id_b,
            agent_type: "t".to_string(),
            call_text: code.to_string(),
            result_text: "error 2".to_string(),
            success: false,
            duration_ms: Some(20),
            state: McpBlockState::Failed,
            separate_call_content: Vec::new(),
        }),
        TimelineSegmentBlock::Mcp(McpBlockData {
            tool_name: "exec".to_string(),
            call_id: id_c,
            agent_type: "t".to_string(),
            call_text: code.to_string(),
            result_text: "error 3".to_string(),
            success: false,
            duration_ms: Some(30),
            state: McpBlockState::Failed,
            separate_call_content: Vec::new(),
        }),
    ];
    TimelineGroupData::dedup_interleaved_by_content(&mut blocks);

    assert_eq!(
        blocks.len(),
        1,
        "3 identical exec calls → keep only the last"
    );
    let mcp = match &blocks[0] {
        TimelineSegmentBlock::Mcp(mcp) => mcp,
        _ => bail!("expected Mcp"),
    };
    assert_eq!(mcp.call_id, id_c, "last occurrence kept");
    assert_eq!(mcp.result_text, "error 3", "last result preserved");
    Ok(())
}

#[test]
fn unit_dedup_keeps_different_calls() -> Result<()> {
    let id_a = Uuid::now_v7();
    let id_b = Uuid::now_v7();
    let mut blocks: Vec<TimelineSegmentBlock> = vec![
        TimelineSegmentBlock::Mcp(McpBlockData {
            tool_name: "exec".to_string(),
            call_id: id_a,
            agent_type: "t".to_string(),
            call_text: r#"console.log("a")"#.to_string(),
            result_text: "ok".to_string(),
            success: true,
            duration_ms: Some(5),
            state: McpBlockState::Done,
            separate_call_content: Vec::new(),
        }),
        TimelineSegmentBlock::Mcp(McpBlockData {
            tool_name: "exec".to_string(),
            call_id: id_b,
            agent_type: "t".to_string(),
            call_text: r#"console.log("b")"#.to_string(),
            result_text: "ok".to_string(),
            success: true,
            duration_ms: Some(6),
            state: McpBlockState::Done,
            separate_call_content: Vec::new(),
        }),
    ];
    TimelineGroupData::dedup_interleaved_by_content(&mut blocks);

    assert_eq!(blocks.len(), 2, "different call_text → both kept");
    Ok(())
}

#[test]
fn integration_dedup_across_retries_in_pipeline() {
    let id_r1 = Uuid::now_v7();
    let id_r2 = Uuid::now_v7();
    let id_r3 = Uuid::now_v7();
    let id_ok = Uuid::now_v7();
    let bad_code_ref = var_namespace::ref_bracket("reply_text");
    let bad_code = &format!(
        "orexis.report_human({{ summary: {}, mode: incomplete",
        bad_code_ref
    );
    let segments = vec![
        txt("First attempt:"),
        mcp_call("exec", id_r1, bad_code),
        mcp_result(
            "exec",
            id_r1,
            "SyntaxError: abrupt end at :181",
            false,
            Some(25),
        ),
        txt("Nudge retry:"),
        mcp_call("exec", id_r2, bad_code),
        mcp_result(
            "exec",
            id_r2,
            "SyntaxError: abrupt end at :181",
            false,
            Some(25),
        ),
        txt("Second retry:"),
        mcp_call("exec", id_r3, bad_code),
        mcp_result(
            "exec",
            id_r3,
            "SyntaxError: abrupt end at :174",
            false,
            Some(25),
        ),
        txt("Finally got it right:"),
        mcp_call(
            "exec",
            id_ok,
            &format!(
                r#"orexis.report_human({})"#,
                var_namespace::ref_bracket("reply_payload")
            ),
        ),
        mcp_result("exec", id_ok, "reported ok", true, Some(5)),
    ];

    let interleaved = TimelineGroupData::segments_to_interleaved(&segments, "agent", false);

    let exec_failed_count = interleaved
        .iter()
        .filter(|b| {
            matches!(b,
                TimelineSegmentBlock::Mcp(m) if m.tool_name == "exec"
                    && m.state == McpBlockState::Failed
                    && m.result_text.contains("abrupt end")
            )
        })
        .count();
    assert_eq!(
        exec_failed_count, 1,
        "3 identical failed exec retries → collapsed to 1 (last occurrence)"
    );

    let exec_done_count = interleaved
        .iter()
        .filter(|b| {
            matches!(b,
                TimelineSegmentBlock::Mcp(m) if m.tool_name == "exec"
                    && m.state == McpBlockState::Done
            )
        })
        .count();
    assert_eq!(exec_done_count, 1, "the successful exec stays");

    let text_count = interleaved
        .iter()
        .filter(|b| matches!(b, TimelineSegmentBlock::Content(_)))
        .count();
    assert_eq!(
        text_count, 4,
        "all 4 text chunks preserved (First attempt / Nudge / Second retry / Finally)"
    );
}

fn content_texts_contain(texts: &[&str], needle: &str) -> bool {
    texts.iter().any(|t| t.contains(needle))
}

// ═══════════════════════════════════════════════════
//  Cross-sub-report boundary tests
//  Simulates the groups.rs pattern: process per-sub
//  then concatenate and apply cross-sub dedup.
// ═══════════════════════════════════════════════════

/// Simulates 3 retries each producing the same failed exec,
/// then a final successful attempt — each in its own sub-report.
/// After per-sub processing + cross-sub dedup, only the last
/// failed and the success should remain.
#[test]
fn cross_sub_three_retries_dedup_to_last_plus_success() -> Result<()> {
    let bad_ref = var_namespace::ref_bracket("reply_text");
    let good_ref = var_namespace::ref_bracket("reply_payload");
    let bad_code = format!(r#"orexis.report_human({{summary:{} }})"#, bad_ref);
    let good_code = format!(r#"orexis.report_human({})"#, good_ref);

    let id_r1 = Uuid::now_v7();
    let id_r2 = Uuid::now_v7();
    let id_r3 = Uuid::now_v7();
    let id_ok = Uuid::now_v7();

    let sub1 = vec![
        txt("Attempt 1"),
        mcp_call("exec", id_r1, &bad_code),
        mcp_result("exec", id_r1, "SyntaxError", false, Some(10)),
    ];
    let sub2 = vec![
        txt("Attempt 2"),
        mcp_call("exec", id_r2, &bad_code),
        mcp_result("exec", id_r2, "SyntaxError", false, Some(10)),
    ];
    let sub3 = vec![
        txt("Attempt 3"),
        mcp_call("exec", id_r3, &bad_code),
        mcp_result("exec", id_r3, "SyntaxError", false, Some(10)),
    ];
    let sub4 = vec![
        txt("Finally correct"),
        mcp_call("exec", id_ok, &good_code),
        mcp_result("exec", id_ok, "ok", true, Some(5)),
    ];

    let subs: Vec<&[StreamSegment]> = vec![&sub1, &sub2, &sub3, &sub4];

    let mut interleaved: Vec<TimelineSegmentBlock> = Vec::new();
    for sub in &subs {
        interleaved.extend(TimelineGroupData::segments_to_interleaved(
            sub, "agent", false,
        ));
    }
    TimelineGroupData::dedup_interleaved_by_content(&mut interleaved);

    let failed_execs: Vec<_> = interleaved
        .iter()
        .filter(|b| {
            matches!(b,
                TimelineSegmentBlock::Mcp(m) if m.tool_name == "exec"
                    && m.state == McpBlockState::Failed
            )
        })
        .collect();
    assert_eq!(
        failed_execs.len(),
        1,
        "3 identical failed execs across sub-reports → 1"
    );

    let ok_execs: Vec<_> = interleaved
        .iter()
        .filter(|b| {
            matches!(b,
                TimelineSegmentBlock::Mcp(m) if m.tool_name == "exec"
                    && m.state == McpBlockState::Done
            )
        })
        .collect();
    assert_eq!(ok_execs.len(), 1, "successful exec preserved");

    let text_count = interleaved
        .iter()
        .filter(|b| matches!(b, TimelineSegmentBlock::Content(_)))
        .count();
    assert_eq!(
        text_count, 4,
        "all 4 text chunks preserved across sub-reports"
    );
    Ok(())
}

/// Cross-sub orphan boundary: each sub-report has its own orphan + complete pair.
/// Per-sub processing ensures orphans are correctly identified within their own
/// sub-report context — no cross-sub contamination.
#[test]
fn cross_sub_orphans_isolated_per_sub() -> Result<()> {
    let id_orphan_a = Uuid::now_v7();
    let id_ok_a = Uuid::now_v7();
    let id_orphan_b = Uuid::now_v7();
    let id_ok_b = Uuid::now_v7();

    let sub_a = vec![
        mcp_call_empty("exec", id_orphan_a),
        mcp_call("exec", id_ok_a, r#"console.log('a')"#),
        mcp_result("exec", id_ok_a, "a", true, Some(1)),
    ];
    let sub_b = vec![
        mcp_call_empty("exec", id_orphan_b),
        mcp_call("exec", id_ok_b, r#"console.log('b')"#),
        mcp_result("exec", id_ok_b, "b", true, Some(1)),
    ];

    let sub_a_interleaved = TimelineGroupData::segments_to_interleaved(&sub_a, "agent", false);
    let sub_b_interleaved = TimelineGroupData::segments_to_interleaved(&sub_b, "agent", false);

    let orphan_a_degraded = sub_a_interleaved.iter().any(|b| {
        matches!(b,
            TimelineSegmentBlock::Mcp(m) if m.state == McpBlockState::HistoryLost
        )
    });
    assert!(
        orphan_a_degraded,
        "sub_a orphan degraded to Mcp(HistoryLost) within its own boundary"
    );

    let orphan_b_degraded = sub_b_interleaved.iter().any(|b| {
        matches!(b,
            TimelineSegmentBlock::Mcp(m) if m.state == McpBlockState::HistoryLost
        )
    });
    assert!(
        orphan_b_degraded,
        "sub_b orphan degraded to Mcp(HistoryLost) within its own boundary"
    );
    Ok(())
}

/// Cross-sub mcp_blocks dedup: same tool_name+call_text across different
/// sub-reports should collapse after cross-sub content dedup.
#[test]
fn cross_sub_mcp_blocks_dedup_by_content() -> Result<()> {
    let id_r1 = Uuid::now_v7();
    let id_r2 = Uuid::now_v7();
    let id_ok = Uuid::now_v7();
    let code = r#"console.log("same")"#;

    let sub1 = vec![
        mcp_call("exec", id_r1, code),
        mcp_result("exec", id_r1, "error", false, Some(5)),
    ];
    let sub2 = vec![
        mcp_call("exec", id_r2, code),
        mcp_result("exec", id_r2, "error", false, Some(5)),
    ];
    let sub3 = vec![
        mcp_call("exec", id_ok, r#"console.log("different")"#),
        mcp_result("exec", id_ok, "ok", true, Some(3)),
    ];

    let mut mcp_blocks: Vec<McpBlockData> = Vec::new();
    for sub in [&sub1, &sub2, &sub3] {
        mcp_blocks.extend(TimelineGroupData::segments_to_mcp_blocks(sub, "agent"));
    }
    TimelineGroupData::dedup_mcp_blocks_by_content(&mut mcp_blocks);

    let same_code_blocks: Vec<_> = mcp_blocks.iter().filter(|b| b.call_text == code).collect();
    assert_eq!(
        same_code_blocks.len(),
        1,
        "2 identical exec blocks → 1 after cross-sub dedup"
    );
    assert_eq!(same_code_blocks[0].call_id, id_r2, "keeps last occurrence");

    assert_eq!(mcp_blocks.len(), 2, "total: 1 dedup'd + 1 different");
    Ok(())
}

/// Simulates close_pending_mcp_calls() behavior: a McpCall whose synthetic
/// McpResult is appended at the END of segments (non-adjacent). The
/// interleaved path pairs non-adjacent Call+Result via result_map,
/// producing a single Failed block with both call_text and result_text.
#[test]
fn integration_non_adjacent_synthetic_result_orphan_handling() -> Result<()> {
    let id_x = Uuid::now_v7();
    let id_ok = Uuid::now_v7();
    let segments = vec![
        txt("Starting work..."),
        mcp_call("exec", id_x, r#"orexis.report_human({summary:"partial"})"#),
        txt("Stream interrupted here."),
        mcp_result(
            "exec",
            id_x,
            "[stream sealed before result arrived]",
            false,
            None,
        ),
        mcp_call("exec", id_ok, r#"console.log("recovered")"#),
        mcp_result("exec", id_ok, "ok", true, Some(2)),
    ];

    let interleaved = TimelineGroupData::segments_to_interleaved(&segments, "agent", false);

    let mcp_count = interleaved
        .iter()
        .filter(|b| matches!(b, TimelineSegmentBlock::Mcp(_)))
        .count();
    assert_eq!(mcp_count, 2, "2 MCP blocks: paired Failed + real Done");

    let pending_count = interleaved
        .iter()
        .filter(|b| {
            matches!(b,
                TimelineSegmentBlock::Mcp(m) if m.state == McpBlockState::Pending
            )
        })
        .count();
    assert_eq!(
        pending_count, 0,
        "no Pending blocks remain — call paired with non-adjacent result"
    );

    let paired_call = interleaved.iter().any(|b| match b {
        TimelineSegmentBlock::Mcp(m) => {
            m.call_text.contains("orexis.report_human")
                && m.result_text
                    .contains("[stream sealed before result arrived]")
        },
        _ => false,
    });
    assert!(
        paired_call,
        "non-adjacent McpCall paired with synthetic McpResult into single Failed block"
    );

    let synthetic_failed = interleaved.iter().any(|b| match b {
        TimelineSegmentBlock::Mcp(m) => m
            .result_text
            .contains("[stream sealed before result arrived]"),
        _ => false,
    });
    assert!(
        synthetic_failed,
        "non-adjacent synthetic McpResult appears as standalone Failed block"
    );
    Ok(())
}

/// Documents expected behavior: duplicate Content text across sub-reports
/// is NOT deduped (Content is never deleted). Both are preserved.
#[test]
fn cross_sub_duplicate_content_preserved_by_design() -> Result<()> {
    let id_a = Uuid::now_v7();
    let id_b = Uuid::now_v7();
    let same_text = "Let me try executing the code";

    let sub1 = vec![
        txt(same_text),
        mcp_call("exec", id_a, r#"bad_code"#),
        mcp_result("exec", id_a, "error", false, Some(5)),
    ];
    let sub2 = vec![
        txt(same_text),
        mcp_call("exec", id_b, r#"bad_code"#),
        mcp_result("exec", id_b, "error", false, Some(5)),
    ];

    let mut interleaved: Vec<TimelineSegmentBlock> = Vec::new();
    for sub in [&sub1 as &[StreamSegment], &sub2 as &[StreamSegment]] {
        interleaved.extend(TimelineGroupData::segments_to_interleaved(
            sub, "agent", false,
        ));
    }
    TimelineGroupData::dedup_interleaved_by_content(&mut interleaved);

    let content_texts: Vec<&str> = interleaved
        .iter()
        .filter_map(|b| match b {
            TimelineSegmentBlock::Content(c) => Some(c.text.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(
        content_texts.len(),
        2,
        "duplicate Content text across sub-reports preserved (never deleted)"
    );
    assert!(
        content_texts.iter().all(|t| *t == same_text),
        "both Content blocks contain the same text"
    );

    let mcp_count = interleaved
        .iter()
        .filter(|b| matches!(b, TimelineSegmentBlock::Mcp(_)))
        .count();
    assert_eq!(mcp_count, 1, "identical MCP blocks collapsed to 1");
    Ok(())
}

/// Documents pairing behavior for non-adjacent synthetic results:
///
/// For non-adjacent synthetic results (seal_coalesced appends McpResult at
/// the END of segments, separated from their McpCall by Content segments):
///
/// - mcp_blocks path: filters to MCP-only segments → Call+Result become
///   adjacent → correctly paired → single McpBlockData with both call_text
///   and result_text.
///
/// - interleaved path: uses result_map (HashMap<Uuid, usize>) built from
///   ALL segments → non-adjacent Call+Result ARE paired → single Mcp block
///   with both call_text and result_text.
#[test]
fn integration_mcp_blocks_pairs_non_adjacent_synthetic_but_interleaved_does_not() -> Result<()> {
    let id_x = Uuid::now_v7();
    let call_code = r#"orexis.report_human({summary:"partial"})"#;
    let sealed_msg = "[stream sealed before result arrived]";

    let segments = vec![
        txt("Working..."),
        mcp_call("exec", id_x, call_code),
        txt("Interrupted."),
        mcp_result("exec", id_x, sealed_msg, false, None),
    ];

    let mcps = TimelineGroupData::segments_to_mcp_blocks(&segments, "agent");
    assert_eq!(
        mcps.len(),
        1,
        "mcp_blocks: filters non-MCP → adjacent → 1 paired block"
    );
    assert_eq!(
        mcps[0].call_text, call_code,
        "mcp_blocks: call_text preserved"
    );
    assert_eq!(
        mcps[0].result_text, sealed_msg,
        "mcp_blocks: result_text preserved"
    );
    assert_eq!(mcps[0].state, McpBlockState::Failed);

    let interleaved = TimelineGroupData::segments_to_interleaved(&segments, "agent", false);

    let paired_mcp = interleaved.iter().any(|b| match b {
        TimelineSegmentBlock::Mcp(m) => m.call_text == call_code && m.result_text == sealed_msg,
        _ => false,
    });
    assert!(
        paired_mcp,
        "interleaved: non-adjacent segments paired into single block via result_map lookup"
    );

    let code_in_mcp = interleaved.iter().any(|b| match b {
        TimelineSegmentBlock::Mcp(m) => m.call_text == call_code,
        _ => false,
    });
    assert!(
        code_in_mcp,
        "interleaved: call code preserved in paired Mcp block"
    );

    let synthetic_paired = interleaved.iter().any(|b| match b {
        TimelineSegmentBlock::Mcp(m) => {
            m.call_text == call_code && m.result_text.contains(sealed_msg)
        },
        _ => false,
    });
    assert!(
        synthetic_paired,
        "interleaved: synthetic result paired with call into single Failed block"
    );
    Ok(())
}

/// Verifies that a trailing orphan whose synthetic McpResult is appended
/// at the end gets paired via result_map lookup, producing a single
/// Failed Mcp block with both call_text and result_text.
#[test]
fn integration_trailing_orphan_degraded_by_synthetic_successor() -> Result<()> {
    let id_x = Uuid::now_v7();
    let call_code = r#"console.log("interrupted")"#;
    let sealed_msg = "[stream sealed before result arrived]";

    let segments = vec![
        txt("Starting..."),
        mcp_call("exec", id_x, call_code),
        txt("Stream interrupted here."),
        mcp_result("exec", id_x, sealed_msg, false, None),
    ];

    let interleaved = TimelineGroupData::segments_to_interleaved(&segments, "agent", false);

    let pending_count = interleaved
        .iter()
        .filter(|b| {
            matches!(b,
                TimelineSegmentBlock::Mcp(m) if m.state == McpBlockState::Pending
            )
        })
        .count();
    assert_eq!(
        pending_count, 0,
        "no Pending blocks — call paired with synthetic Result via result_map"
    );

    let code_in_paired_mcp = interleaved.iter().any(|b| match b {
        TimelineSegmentBlock::Mcp(m) => m.call_text == call_code,
        _ => false,
    });
    assert!(
        code_in_paired_mcp,
        "trailing orphan's call_text preserved in paired Mcp block"
    );

    let synthetic_paired = interleaved.iter().any(|b| match b {
        TimelineSegmentBlock::Mcp(m) => {
            m.call_text == call_code && m.result_text.contains(sealed_msg)
        },
        _ => false,
    });
    assert!(
        synthetic_paired,
        "synthetic McpResult paired with call into single Failed block"
    );
    Ok(())
}

/// Edge case: McpCall with empty call_text + non-adjacent synthetic McpResult.
/// Both produce key ("exec", "") → dedup removes the Pending block (earlier
/// occurrence). No information lost since call_text was empty.
#[test]
fn integration_empty_call_text_dedup_with_non_adjacent_synthetic() -> Result<()> {
    let id_x = Uuid::now_v7();
    let sealed_msg = "[stream sealed before result arrived]";

    let segments = vec![
        mcp_call_empty("exec", id_x),
        txt("some text between"),
        mcp_result("exec", id_x, sealed_msg, false, None),
    ];

    let interleaved = TimelineGroupData::segments_to_interleaved(&segments, "agent", false);

    let pending_count = interleaved
        .iter()
        .filter(|b| {
            matches!(b,
                TimelineSegmentBlock::Mcp(m) if m.state == McpBlockState::Pending
            )
        })
        .count();
    assert_eq!(pending_count, 0, "empty Pending deduped or downgraded");

    let mcp_count = interleaved
        .iter()
        .filter(|b| matches!(b, TimelineSegmentBlock::Mcp(_)))
        .count();
    assert_eq!(mcp_count, 1, "only synthetic Failed block remains");

    let failed = interleaved.iter().find_map(|b| match b {
        TimelineSegmentBlock::Mcp(m) if m.call_text.is_empty() => Some(m),
        _ => None,
    });
    assert!(
        failed.is_some(),
        "synthetic Failed block with empty call_text preserved"
    );
    let failed = failed.context("synthetic Failed block")?;
    assert_eq!(
        failed.result_text, sealed_msg,
        "synthetic result text preserved"
    );
    Ok(())
}
