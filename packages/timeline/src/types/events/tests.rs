use anyhow::{Context, Result, bail};
use uuid::Uuid;

use super::*;
use _core::var_namespace;
use _text::StreamSegment;

fn txt(text: &str) -> StreamSegment {
    StreamSegment::Text {
        text: text.to_string(),
        message_id: None,
    }
}

fn tool_call(tool: &str, call_id: Uuid, text: &str) -> StreamSegment {
    let params = if text.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_str::<serde_json::Value>(text)
            .unwrap_or(serde_json::Value::String(text.to_string()))
    };
    StreamSegment::ToolCall {
        tool_name: tool.to_string(),
        call_id,
        params,
        agent_type: Some("haplotes".to_string()),
        message_id: None,
    }
}

fn tool_call_empty(tool: &str, call_id: Uuid) -> StreamSegment {
    tool_call(tool, call_id, "")
}

fn tool_result(
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
    StreamSegment::ToolResult {
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
//  Unit: downgrade_orphaned_pending_tool  (tool_blocks path)
//  Constraint: return type is Vec<ToolBlockData>, cannot
//  produce Content. Best degradation = mark HistoryLost.
//  Nothing is ever deleted.
// ═══════════════════════════════════════════════════

#[test]
fn unit_tool_empty_pending_before_done_becomes_done() -> Result<()> {
    let mut blocks = vec![
        ToolBlockData {
            tool_name: "a".to_string(),
            call_id: Uuid::now_v7(),
            agent_type: "t".to_string(),
            call_text: String::new(),
            result_text: String::new(),
            success: true,
            duration_ms: None,
            state: ToolBlockState::Pending,
            separate_call_content: Vec::new(),
        },
        ToolBlockData {
            tool_name: "b".to_string(),
            call_id: Uuid::now_v7(),
            agent_type: "t".to_string(),
            call_text: "params".to_string(),
            result_text: "ok".to_string(),
            success: true,
            duration_ms: Some(100),
            state: ToolBlockState::Done,
            separate_call_content: Vec::new(),
        },
    ];
    TimelineGroupData::downgrade_orphaned_pending_tool(&mut blocks);
    assert_eq!(blocks.len(), 2, "nothing deleted");
    assert_eq!(
        blocks[0].state,
        ToolBlockState::HistoryLost,
        "orphaned Running degraded to HistoryLost"
    );
    assert_eq!(blocks[1].state, ToolBlockState::Done);
    Ok(())
}

#[test]
fn unit_tool_running_no_result_before_done_becomes_done() -> Result<()> {
    let mut blocks = vec![
        ToolBlockData {
            tool_name: "a".to_string(),
            call_id: Uuid::now_v7(),
            agent_type: "t".to_string(),
            call_text: "partial code here".to_string(),
            result_text: String::new(),
            success: true,
            duration_ms: None,
            state: ToolBlockState::Running,
            separate_call_content: Vec::new(),
        },
        ToolBlockData {
            tool_name: "b".to_string(),
            call_id: Uuid::now_v7(),
            agent_type: "t".to_string(),
            call_text: "{}".to_string(),
            result_text: "done".to_string(),
            success: true,
            duration_ms: Some(50),
            state: ToolBlockState::Done,
            separate_call_content: Vec::new(),
        },
    ];
    TimelineGroupData::downgrade_orphaned_pending_tool(&mut blocks);
    assert_eq!(blocks.len(), 2, "nothing deleted");
    assert_eq!(blocks[0].state, ToolBlockState::HistoryLost);
    Ok(())
}

#[test]
fn unit_tool_trailing_incomplete_never_touched() -> Result<()> {
    let mut blocks = vec![
        ToolBlockData {
            tool_name: "complete".to_string(),
            call_id: Uuid::now_v7(),
            agent_type: "t".to_string(),
            call_text: "x".to_string(),
            result_text: "y".to_string(),
            success: true,
            duration_ms: Some(10),
            state: ToolBlockState::Done,
            separate_call_content: Vec::new(),
        },
        ToolBlockData {
            tool_name: "orphan".to_string(),
            call_id: Uuid::now_v7(),
            agent_type: "t".to_string(),
            call_text: String::new(),
            result_text: String::new(),
            success: true,
            duration_ms: None,
            state: ToolBlockState::Pending,
            separate_call_content: Vec::new(),
        },
    ];
    TimelineGroupData::downgrade_orphaned_pending_tool(&mut blocks);
    assert_eq!(blocks.len(), 2);
    assert_eq!(
        blocks[1].state,
        ToolBlockState::Pending,
        "trailing stays Pending"
    );
    Ok(())
}

#[test]
fn unit_tool_all_incomplete_chain_preserved_as_is() -> Result<()> {
    let mut blocks = vec![
        ToolBlockData {
            tool_name: "p1".to_string(),
            call_id: Uuid::now_v7(),
            agent_type: "t".to_string(),
            call_text: String::new(),
            result_text: String::new(),
            success: true,
            duration_ms: None,
            state: ToolBlockState::Pending,
            separate_call_content: Vec::new(),
        },
        ToolBlockData {
            tool_name: "r1".to_string(),
            call_id: Uuid::now_v7(),
            agent_type: "t".to_string(),
            call_text: "partial".to_string(),
            result_text: String::new(),
            success: true,
            duration_ms: None,
            state: ToolBlockState::Running,
            separate_call_content: Vec::new(),
        },
    ];
    TimelineGroupData::downgrade_orphaned_pending_tool(&mut blocks);
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].state, ToolBlockState::Pending);
    assert_eq!(blocks[1].state, ToolBlockState::Running);
    Ok(())
}

// ═══════════════════════════════════════════════════
//  Unit: downgrade_orphaned_pending_interleaved
//  All orphans → Tool(HistoryLost). Never delete.
// ═══════════════════════════════════════════════════

#[test]
fn unit_interleaved_empty_pending_becomes_empty_content() -> Result<()> {
    let mut blocks: Vec<TimelineSegmentBlock> = vec![
        TimelineSegmentBlock::Tool(ToolBlockData {
            tool_name: "ghost".to_string(),
            call_id: Uuid::now_v7(),
            agent_type: "t".to_string(),
            call_text: String::new(),
            result_text: String::new(),
            success: true,
            duration_ms: None,
            state: ToolBlockState::Pending,
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
        "nothing deleted; empty orphan stays Tool with HistoryLost"
    );
    let m = match &blocks[0] {
        TimelineSegmentBlock::Tool(m) => m,
        _ => bail!("expected Tool"),
    };
    assert_eq!(
        m.state,
        ToolBlockState::HistoryLost,
        "empty Pending → Tool(HistoryLost)"
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
        TimelineSegmentBlock::Tool(ToolBlockData {
            tool_name: "exec".to_string(),
            call_id: Uuid::now_v7(),
            agent_type: "t".to_string(),
            call_text: partial_code.to_string(),
            result_text: String::new(),
            success: true,
            duration_ms: None,
            state: ToolBlockState::Running,
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
        TimelineSegmentBlock::Tool(m) => m,
        _ => bail!("expected degraded Tool with HistoryLost state"),
    };
    assert_eq!(
        m.state,
        ToolBlockState::HistoryLost,
        "orphan → Tool(HistoryLost)"
    );
    assert_eq!(
        m.call_text,
        partial_code.as_str(),
        "partial call text preserved in Tool block"
    );
    Ok(())
}

// ═══════════════════════════════════════════════════
//  Integration: real StreamSegment → full pipeline
// ═══════════════════════════════════════════════════

/// Text → ToolCall(empty/Pending) → Text → ToolCall(complete) → ToolResult(Done)
/// The orphaned empty Pending in the middle degrades.
/// In interleaved path it becomes Tool(HistoryLost); in tool_blocks it becomes HistoryLost.
#[test]
fn integration_empty_pending_sandwiched_between_content() -> Result<()> {
    let id_empty = Uuid::now_v7();
    let id_complete = Uuid::now_v7();
    let segments = vec![
        txt("I see – the Chinese characters are causing issues."),
        tool_call_empty("exec", id_empty),
        txt("Let me encode as JSON first."),
        tool_call("exec", id_complete, r#"{"code": "payload()"}"#),
        tool_result("exec", id_complete, "ok", true, Some(3)),
    ];

    let interleaved = TimelineGroupData::segments_to_interleaved(&segments, "agent", false);

    assert_eq!(
        interleaved.len(),
        4,
        "4 blocks: text + orphan→Tool(HistoryLost) + text + done-tool (call+result merge into 1)"
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

    let tool_count = interleaved
        .iter()
        .filter(|b| matches!(b, TimelineSegmentBlock::Tool(_)))
        .count();
    assert_eq!(
        tool_count, 2,
        "orphan stays Tool(HistoryLost) + complete exec Tool(Done)"
    );
    Ok(())
}

/// Unclosed/partial ToolCall immediately before a complete pair.
/// Partial has non-empty call_text but no result → Running+no_result → orphan → Tool(HistoryLost).
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
        tool_call("exec", id_partial, partial_code),
        tool_call(
            "exec",
            id_ok,
            &format!(
                r#"orexis.report_human({})"#,
                var_namespace::ref_bracket("reply_payload")
            ),
        ),
        tool_result("exec", id_ok, "reported ok", true, Some(5)),
        txt("Report sent successfully."),
    ];

    let interleaved = TimelineGroupData::segments_to_interleaved(&segments, "agent", false);

    assert_eq!(
        interleaved.len(),
        4,
        "4 blocks: text + orphan→Tool(HistoryLost) + done-tool(merged) + text"
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
        TimelineSegmentBlock::Tool(m) => {
            m.state == ToolBlockState::HistoryLost && m.call_text == *partial_code
        }
        _ => false,
    });
    assert!(
        orphan_history_lost,
        "partial orphan stays Tool with HistoryLost state and original call_text"
    );

    let done_tools: Vec<_> = interleaved
        .iter()
        .filter_map(|b| match b {
            TimelineSegmentBlock::Tool(m) if m.state == ToolBlockState::Done => Some(&m.tool_name),
            _ => None,
        })
        .collect();
    assert_eq!(done_tools.len(), 1);
}

/// Multiple orphans then one complete. All orphans degrade, none deleted.
#[test]
fn integration_multiple_orphans_then_complete() {
    let id_p1 = Uuid::now_v7();
    let id_p2 = Uuid::now_v7();
    let id_ok = Uuid::now_v7();
    let segments = vec![
        tool_call_empty("report", id_p1),
        tool_call(
            "exec",
            id_p2,
            "import { report } from 'hubris'; report({content: 'incomplete",
        ),
        tool_call("report", id_ok, r#"{"content": "full report"}"#),
        tool_result("report", id_ok, "accepted", true, Some(10)),
    ];

    let tool_blocks = TimelineGroupData::segments_to_tool_blocks(&segments, "agent");
    assert_eq!(
        tool_blocks.len(),
        3,
        "3 ToolBlocks total (2 orphans→HistoryLost + 1 real Done), none deleted"
    );
    assert_eq!(
        tool_blocks[0].state,
        ToolBlockState::HistoryLost,
        "empty orphan → HistoryLost"
    );
    assert_eq!(
        tool_blocks[1].state,
        ToolBlockState::HistoryLost,
        "partial orphan → HistoryLost"
    );
    assert_eq!(
        tool_blocks[2].state,
        ToolBlockState::Done,
        "real complete block → Done"
    );

    let interleaved = TimelineGroupData::segments_to_interleaved(&segments, "agent", false);
    assert_eq!(
        interleaved.len(),
        3,
        "3 items: 2 orphans→Tool(HistoryLost) + 1 done-Tool(merged)"
    );
    let content_count = interleaved
        .iter()
        .filter(|b| matches!(b, TimelineSegmentBlock::Content(_)))
        .count();
    assert_eq!(
        content_count, 0,
        "orphans stay Tool(HistoryLost), not converted to Content"
    );
}

/// Trailing incomplete at end of stream — must NOT be touched.
#[test]
fn integration_trailing_incomplete_untouched() -> Result<()> {
    let id_p1 = Uuid::now_v7();
    let id_done = Uuid::now_v7();
    let segments = vec![
        tool_call("report", id_done, r#"{"msg":"done"}"#),
        tool_result("report", id_done, "ok", true, Some(1)),
        tool_call_empty("exec", id_p1),
    ];

    let tool_blocks = TimelineGroupData::segments_to_tool_blocks(&segments, "agent");
    assert_eq!(tool_blocks.len(), 2);
    assert_eq!(tool_blocks[0].state, ToolBlockState::Done);
    assert_eq!(
        tool_blocks[1].state,
        ToolBlockState::Pending,
        "trailing Pending untouched"
    );
    Ok(())
}

/// Full realistic scenario from the bug screenshot:
/// WTV set → **empty exec(Pending)** → WTV set → exec(error) → thinking text →
/// WTV JSON set → **unclosed exec(partial)** → exec(complete)
///
/// Orphans marked: ghost=empty Pending between WTVs, partial=unclosed before final exec.
/// In interleaved: both stay Tool(HistoryLost). In tool_blocks: both become HistoryLost.
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
        tool_call(
            "write_to_var",
            id_wtv1,
            r#"{"var_name":"reply_summary","value":"I am Entelecheia..."}"#,
        ),
        tool_result("write_to_var", id_wtv1, "set ok", true, Some(5)),
        tool_call_empty("exec", id_ghost),
        tool_call(
            "write_to_var",
            id_wtv2,
            r#"{"var_name":"reply_mode","value":"reply"}"#,
        ),
        tool_result("write_to_var", id_wtv2, "set ok", true, Some(1)),
        tool_call("exec", id_exec_err, {
            let summary_ref = var_namespace::ref_bracket("reply_summary");
            let call_text = format!(
                r#"{{"code":"orexis.report_human({{summary:{} }})}}"#,
                summary_ref
            );
            call_text.leak() as &str
        }),
        tool_result(
            "exec",
            id_exec_err,
            "Tool 'exec' failed: SyntaxError: abrupt end",
            false,
            Some(25),
        ),
        txt("I see – Chinese chars causing sandbox parser fail. Let me encode as JSON first."),
        tool_call(
            "write_to_var_json",
            id_wtv_json,
            r#"{"var_name":"reply_payload","value":{"key":"val"}}"#,
        ),
        tool_result(
            "write_to_var_json",
            id_wtv_json,
            "parsed JSON ok",
            true,
            Some(3),
        ),
        tool_call(
            "exec",
            id_partial,
            &format!(
                r#"{{"code":"orexis.report_human({})}}"#,
                var_namespace::ref_bracket("reply_payload")
            ),
        ),
        tool_call("exec", id_final, r#"{"code":"console.log('done')"}"#),
        tool_result("exec", id_final, "undefined", true, Some(2)),
    ];

    let interleaved = TimelineGroupData::segments_to_interleaved(&segments, "skill_agent", false);

    let total = interleaved.len();
    assert_eq!(
        total, 8,
        "13 segments → 8 blocks (5 call+result merge into Tool + 1 text + 2 orphans→Tool(HistoryLost)); nothing deleted"
    );

    let tool_blocks: Vec<_> = interleaved
        .iter()
        .filter_map(|b| match b {
            TimelineSegmentBlock::Tool(m) => Some((&m.tool_name, m.state)),
            _ => None,
        })
        .collect();

    let pending_count = tool_blocks
        .iter()
        .filter(|(_, s)| *s == ToolBlockState::Pending)
        .count();
    assert_eq!(
        pending_count, 0,
        "no Pending left — ghost orphan gone from Tool list"
    );

    let partial_as_history_lost = interleaved.iter().any(|b| match b {
        TimelineSegmentBlock::Tool(m) => {
            m.state == ToolBlockState::HistoryLost
                && m.call_text.contains(&format!(
                    "orexis.report_human({})",
                    var_namespace::ref_bracket("reply_payload")
                ))
        }
        _ => false,
    });
    assert!(
        partial_as_history_lost,
        "unclosed partial exec stays Tool with HistoryLost state"
    );

    let ghost_as_history_lost = interleaved
            .iter()
            .any(|b| matches!(b, TimelineSegmentBlock::Tool(m) if m.state == ToolBlockState::HistoryLost && m.call_text.is_empty()));
    assert!(
        ghost_as_history_lost,
        "empty ghost exec stays Tool with HistoryLost state (not deleted)"
    );

    let has_final_done = tool_blocks
        .iter()
        .any(|(t, s)| *t == "exec" && *s == ToolBlockState::Done);
    assert!(
        has_final_done,
        "final complete exec still a proper Done Tool"
    );
    Ok(())
}

// ═════════════════════════════════════════════════
//  Content-based dedup: same (tool_name, call_text)
//  across retries should collapse to last occurrence.
// ═════════════════════════════════════════════════

#[test]
fn unit_dedup_tool_by_content_removes_earlier_duplicate() -> Result<()> {
    let id_a = Uuid::now_v7();
    let id_b = Uuid::now_v7();
    let id_c = Uuid::now_v7();
    let code = r#"orexis.report_human({summary:"hello"})"#;
    let mut blocks: Vec<TimelineSegmentBlock> = vec![
        TimelineSegmentBlock::Tool(ToolBlockData {
            tool_name: "exec".to_string(),
            call_id: id_a,
            agent_type: "t".to_string(),
            call_text: code.to_string(),
            result_text: "error 1".to_string(),
            success: false,
            duration_ms: Some(10),
            state: ToolBlockState::Failed,
            separate_call_content: Vec::new(),
        }),
        TimelineSegmentBlock::Tool(ToolBlockData {
            tool_name: "exec".to_string(),
            call_id: id_b,
            agent_type: "t".to_string(),
            call_text: code.to_string(),
            result_text: "error 2".to_string(),
            success: false,
            duration_ms: Some(20),
            state: ToolBlockState::Failed,
            separate_call_content: Vec::new(),
        }),
        TimelineSegmentBlock::Tool(ToolBlockData {
            tool_name: "exec".to_string(),
            call_id: id_c,
            agent_type: "t".to_string(),
            call_text: code.to_string(),
            result_text: "error 3".to_string(),
            success: false,
            duration_ms: Some(30),
            state: ToolBlockState::Failed,
            separate_call_content: Vec::new(),
        }),
    ];
    TimelineGroupData::dedup_interleaved_by_content(&mut blocks);

    assert_eq!(
        blocks.len(),
        1,
        "3 identical exec calls → keep only the last"
    );
    let tool = match &blocks[0] {
        TimelineSegmentBlock::Tool(tool) => tool,
        _ => bail!("expected Tool"),
    };
    assert_eq!(tool.call_id, id_c, "last occurrence kept");
    assert_eq!(tool.result_text, "error 3", "last result preserved");
    Ok(())
}

#[test]
fn unit_dedup_keeps_different_calls() -> Result<()> {
    let id_a = Uuid::now_v7();
    let id_b = Uuid::now_v7();
    let mut blocks: Vec<TimelineSegmentBlock> = vec![
        TimelineSegmentBlock::Tool(ToolBlockData {
            tool_name: "exec".to_string(),
            call_id: id_a,
            agent_type: "t".to_string(),
            call_text: r#"console.log("a")"#.to_string(),
            result_text: "ok".to_string(),
            success: true,
            duration_ms: Some(5),
            state: ToolBlockState::Done,
            separate_call_content: Vec::new(),
        }),
        TimelineSegmentBlock::Tool(ToolBlockData {
            tool_name: "exec".to_string(),
            call_id: id_b,
            agent_type: "t".to_string(),
            call_text: r#"console.log("b")"#.to_string(),
            result_text: "ok".to_string(),
            success: true,
            duration_ms: Some(6),
            state: ToolBlockState::Done,
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
        tool_call("exec", id_r1, bad_code),
        tool_result(
            "exec",
            id_r1,
            "SyntaxError: abrupt end at :181",
            false,
            Some(25),
        ),
        txt("Nudge retry:"),
        tool_call("exec", id_r2, bad_code),
        tool_result(
            "exec",
            id_r2,
            "SyntaxError: abrupt end at :181",
            false,
            Some(25),
        ),
        txt("Second retry:"),
        tool_call("exec", id_r3, bad_code),
        tool_result(
            "exec",
            id_r3,
            "SyntaxError: abrupt end at :174",
            false,
            Some(25),
        ),
        txt("Finally got it right:"),
        tool_call(
            "exec",
            id_ok,
            &format!(
                r#"orexis.report_human({})"#,
                var_namespace::ref_bracket("reply_payload")
            ),
        ),
        tool_result("exec", id_ok, "reported ok", true, Some(5)),
    ];

    let interleaved = TimelineGroupData::segments_to_interleaved(&segments, "agent", false);

    let exec_failed_count = interleaved
        .iter()
        .filter(|b| {
            matches!(b,
                TimelineSegmentBlock::Tool(m) if m.tool_name == "exec"
                    && m.state == ToolBlockState::Failed
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
                TimelineSegmentBlock::Tool(m) if m.tool_name == "exec"
                    && m.state == ToolBlockState::Done
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
        tool_call("exec", id_r1, &bad_code),
        tool_result("exec", id_r1, "SyntaxError", false, Some(10)),
    ];
    let sub2 = vec![
        txt("Attempt 2"),
        tool_call("exec", id_r2, &bad_code),
        tool_result("exec", id_r2, "SyntaxError", false, Some(10)),
    ];
    let sub3 = vec![
        txt("Attempt 3"),
        tool_call("exec", id_r3, &bad_code),
        tool_result("exec", id_r3, "SyntaxError", false, Some(10)),
    ];
    let sub4 = vec![
        txt("Finally correct"),
        tool_call("exec", id_ok, &good_code),
        tool_result("exec", id_ok, "ok", true, Some(5)),
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
                TimelineSegmentBlock::Tool(m) if m.tool_name == "exec"
                    && m.state == ToolBlockState::Failed
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
                TimelineSegmentBlock::Tool(m) if m.tool_name == "exec"
                    && m.state == ToolBlockState::Done
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
        tool_call_empty("exec", id_orphan_a),
        tool_call("exec", id_ok_a, r#"console.log('a')"#),
        tool_result("exec", id_ok_a, "a", true, Some(1)),
    ];
    let sub_b = vec![
        tool_call_empty("exec", id_orphan_b),
        tool_call("exec", id_ok_b, r#"console.log('b')"#),
        tool_result("exec", id_ok_b, "b", true, Some(1)),
    ];

    let sub_a_interleaved = TimelineGroupData::segments_to_interleaved(&sub_a, "agent", false);
    let sub_b_interleaved = TimelineGroupData::segments_to_interleaved(&sub_b, "agent", false);

    let orphan_a_degraded = sub_a_interleaved.iter().any(|b| {
        matches!(b,
            TimelineSegmentBlock::Tool(m) if m.state == ToolBlockState::HistoryLost
        )
    });
    assert!(
        orphan_a_degraded,
        "sub_a orphan degraded to Tool(HistoryLost) within its own boundary"
    );

    let orphan_b_degraded = sub_b_interleaved.iter().any(|b| {
        matches!(b,
            TimelineSegmentBlock::Tool(m) if m.state == ToolBlockState::HistoryLost
        )
    });
    assert!(
        orphan_b_degraded,
        "sub_b orphan degraded to Tool(HistoryLost) within its own boundary"
    );
    Ok(())
}

/// Cross-sub tool_blocks dedup: same tool_name+call_text across different
/// sub-reports should collapse after cross-sub content dedup.
#[test]
fn cross_sub_tool_blocks_dedup_by_content() -> Result<()> {
    let id_r1 = Uuid::now_v7();
    let id_r2 = Uuid::now_v7();
    let id_ok = Uuid::now_v7();
    let code = r#"console.log("same")"#;

    let sub1 = vec![
        tool_call("exec", id_r1, code),
        tool_result("exec", id_r1, "error", false, Some(5)),
    ];
    let sub2 = vec![
        tool_call("exec", id_r2, code),
        tool_result("exec", id_r2, "error", false, Some(5)),
    ];
    let sub3 = vec![
        tool_call("exec", id_ok, r#"console.log("different")"#),
        tool_result("exec", id_ok, "ok", true, Some(3)),
    ];

    let mut tool_blocks: Vec<ToolBlockData> = Vec::new();
    for sub in [&sub1, &sub2, &sub3] {
        tool_blocks.extend(TimelineGroupData::segments_to_tool_blocks(sub, "agent"));
    }
    TimelineGroupData::dedup_tool_blocks_by_content(&mut tool_blocks);

    let same_code_blocks: Vec<_> = tool_blocks.iter().filter(|b| b.call_text == code).collect();
    assert_eq!(
        same_code_blocks.len(),
        1,
        "2 identical exec blocks → 1 after cross-sub dedup"
    );
    assert_eq!(same_code_blocks[0].call_id, id_r2, "keeps last occurrence");

    assert_eq!(tool_blocks.len(), 2, "total: 1 dedup'd + 1 different");
    Ok(())
}

/// Simulates close_pending_tool_calls() behavior: a ToolCall whose synthetic
/// ToolResult is appended at the END of segments (non-adjacent). The
/// interleaved path pairs non-adjacent Call+Result via result_map,
/// producing a single Failed block with both call_text and result_text.
#[test]
fn integration_non_adjacent_synthetic_result_orphan_handling() -> Result<()> {
    let id_x = Uuid::now_v7();
    let id_ok = Uuid::now_v7();
    let segments = vec![
        txt("Starting work..."),
        tool_call("exec", id_x, r#"orexis.report_human({summary:"partial"})"#),
        txt("Stream interrupted here."),
        tool_result(
            "exec",
            id_x,
            "[stream sealed before result arrived]",
            false,
            None,
        ),
        tool_call("exec", id_ok, r#"console.log("recovered")"#),
        tool_result("exec", id_ok, "ok", true, Some(2)),
    ];

    let interleaved = TimelineGroupData::segments_to_interleaved(&segments, "agent", false);

    let tool_count = interleaved
        .iter()
        .filter(|b| matches!(b, TimelineSegmentBlock::Tool(_)))
        .count();
    assert_eq!(tool_count, 2, "2 MCP blocks: paired Failed + real Done");

    let pending_count = interleaved
        .iter()
        .filter(|b| {
            matches!(b,
                TimelineSegmentBlock::Tool(m) if m.state == ToolBlockState::Pending
            )
        })
        .count();
    assert_eq!(
        pending_count, 0,
        "no Pending blocks remain — call paired with non-adjacent result"
    );

    let paired_call = interleaved.iter().any(|b| match b {
        TimelineSegmentBlock::Tool(m) => {
            m.call_text.contains("orexis.report_human")
                && m.result_text
                    .contains("[stream sealed before result arrived]")
        }
        _ => false,
    });
    assert!(
        paired_call,
        "non-adjacent ToolCall paired with synthetic ToolResult into single Failed block"
    );

    let synthetic_failed = interleaved.iter().any(|b| match b {
        TimelineSegmentBlock::Tool(m) => m
            .result_text
            .contains("[stream sealed before result arrived]"),
        _ => false,
    });
    assert!(
        synthetic_failed,
        "non-adjacent synthetic ToolResult appears as standalone Failed block"
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
        tool_call("exec", id_a, r#"bad_code"#),
        tool_result("exec", id_a, "error", false, Some(5)),
    ];
    let sub2 = vec![
        txt(same_text),
        tool_call("exec", id_b, r#"bad_code"#),
        tool_result("exec", id_b, "error", false, Some(5)),
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

    let tool_count = interleaved
        .iter()
        .filter(|b| matches!(b, TimelineSegmentBlock::Tool(_)))
        .count();
    assert_eq!(tool_count, 1, "identical MCP blocks collapsed to 1");
    Ok(())
}

/// Documents pairing behavior for non-adjacent synthetic results:
///
/// For non-adjacent synthetic results (seal_coalesced appends ToolResult at
/// the END of segments, separated from their ToolCall by Content segments):
///
/// - tool_blocks path: filters to MCP-only segments → Call+Result become
///   adjacent → correctly paired → single ToolBlockData with both call_text
///   and result_text.
///
/// - interleaved path: uses result_map (HashMap<Uuid, usize>) built from
///   ALL segments → non-adjacent Call+Result ARE paired → single Tool block
///   with both call_text and result_text.
#[test]
fn integration_tool_blocks_pairs_non_adjacent_synthetic_but_interleaved_does_not() -> Result<()> {
    let id_x = Uuid::now_v7();
    let call_code = r#"orexis.report_human({summary:"partial"})"#;
    let sealed_msg = "[stream sealed before result arrived]";

    let segments = vec![
        txt("Working..."),
        tool_call("exec", id_x, call_code),
        txt("Interrupted."),
        tool_result("exec", id_x, sealed_msg, false, None),
    ];

    let tool_blocks = TimelineGroupData::segments_to_tool_blocks(&segments, "agent");
    assert_eq!(
        tool_blocks.len(),
        1,
        "tool_blocks: filters non-MCP → adjacent → 1 paired block"
    );
    assert_eq!(
        tool_blocks[0].call_text, call_code,
        "tool_blocks: call_text preserved"
    );
    assert_eq!(
        tool_blocks[0].result_text, sealed_msg,
        "tool_blocks: result_text preserved"
    );
    assert_eq!(tool_blocks[0].state, ToolBlockState::Failed);

    let interleaved = TimelineGroupData::segments_to_interleaved(&segments, "agent", false);

    let paired_tool = interleaved.iter().any(|b| match b {
        TimelineSegmentBlock::Tool(m) => m.call_text == call_code && m.result_text == sealed_msg,
        _ => false,
    });
    assert!(
        paired_tool,
        "interleaved: non-adjacent segments paired into single block via result_map lookup"
    );

    let code_in_tool = interleaved.iter().any(|b| match b {
        TimelineSegmentBlock::Tool(m) => m.call_text == call_code,
        _ => false,
    });
    assert!(
        code_in_tool,
        "interleaved: call code preserved in paired Tool block"
    );

    let synthetic_paired = interleaved.iter().any(|b| match b {
        TimelineSegmentBlock::Tool(m) => {
            m.call_text == call_code && m.result_text.contains(sealed_msg)
        }
        _ => false,
    });
    assert!(
        synthetic_paired,
        "interleaved: synthetic result paired with call into single Failed block"
    );
    Ok(())
}

/// Verifies that a trailing orphan whose synthetic ToolResult is appended
/// at the end gets paired via result_map lookup, producing a single
/// Failed Tool block with both call_text and result_text.
#[test]
fn integration_trailing_orphan_degraded_by_synthetic_successor() -> Result<()> {
    let id_x = Uuid::now_v7();
    let call_code = r#"console.log("interrupted")"#;
    let sealed_msg = "[stream sealed before result arrived]";

    let segments = vec![
        txt("Starting..."),
        tool_call("exec", id_x, call_code),
        txt("Stream interrupted here."),
        tool_result("exec", id_x, sealed_msg, false, None),
    ];

    let interleaved = TimelineGroupData::segments_to_interleaved(&segments, "agent", false);

    let pending_count = interleaved
        .iter()
        .filter(|b| {
            matches!(b,
                TimelineSegmentBlock::Tool(m) if m.state == ToolBlockState::Pending
            )
        })
        .count();
    assert_eq!(
        pending_count, 0,
        "no Pending blocks — call paired with synthetic Result via result_map"
    );

    let code_in_paired_tool = interleaved.iter().any(|b| match b {
        TimelineSegmentBlock::Tool(m) => m.call_text == call_code,
        _ => false,
    });
    assert!(
        code_in_paired_tool,
        "trailing orphan's call_text preserved in paired Tool block"
    );

    let synthetic_paired = interleaved.iter().any(|b| match b {
        TimelineSegmentBlock::Tool(m) => {
            m.call_text == call_code && m.result_text.contains(sealed_msg)
        }
        _ => false,
    });
    assert!(
        synthetic_paired,
        "synthetic ToolResult paired with call into single Failed block"
    );
    Ok(())
}

/// Edge case: ToolCall with empty call_text + non-adjacent synthetic ToolResult.
/// Both produce key ("exec", "") → dedup removes the Pending block (earlier
/// occurrence). No information lost since call_text was empty.
#[test]
fn integration_empty_call_text_dedup_with_non_adjacent_synthetic() -> Result<()> {
    let id_x = Uuid::now_v7();
    let sealed_msg = "[stream sealed before result arrived]";

    let segments = vec![
        tool_call_empty("exec", id_x),
        txt("some text between"),
        tool_result("exec", id_x, sealed_msg, false, None),
    ];

    let interleaved = TimelineGroupData::segments_to_interleaved(&segments, "agent", false);

    let pending_count = interleaved
        .iter()
        .filter(|b| {
            matches!(b,
                TimelineSegmentBlock::Tool(m) if m.state == ToolBlockState::Pending
            )
        })
        .count();
    assert_eq!(pending_count, 0, "empty Pending deduped or downgraded");

    let tool_count = interleaved
        .iter()
        .filter(|b| matches!(b, TimelineSegmentBlock::Tool(_)))
        .count();
    assert_eq!(tool_count, 1, "only synthetic Failed block remains");

    let failed = interleaved.iter().find_map(|b| match b {
        TimelineSegmentBlock::Tool(m) if m.call_text.is_empty() => Some(m),
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
