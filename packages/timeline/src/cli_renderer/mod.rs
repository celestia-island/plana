mod buffer;
mod column;
#[cfg(test)]
mod mcp_block;

pub use buffer::CliTimelineBuffer;
pub use column::{COL_PAD, fmt_cont_line, fmt_tool_line, fmt_truncate};

use super::{
    chars::{ARROW_DOWN, ARROW_SWAP, ARROW_UP},
    renderer::TimelineRenderer,
    types::{GroupStats, TimelineAskHumanGroup, TimelineGroupData, TimelineHumanGroup},
};

pub struct CliTimelineRenderer;

impl TimelineRenderer for CliTimelineRenderer {
    type Line = String;

    fn format_stats(&self, stats: &GroupStats) -> String {
        let mut parts = Vec::new();
        if let Some(ref inp) = stats.input_tokens {
            parts.push(format!("{} {}", ARROW_UP, inp.format()));
        }
        if let Some(ref out) = stats.output_tokens {
            parts.push(format!("{} {}", ARROW_DOWN, out.format()));
        }
        if stats.mcp_count > 0 {
            parts.push(format!("{} {}", ARROW_SWAP, stats.mcp_count));
        }
        if let Some(dur) = stats.duration_secs {
            parts.push(format!("{:.1}s", dur));
        }
        parts.join(" ")
    }

    fn render_human_group(&self, group: &TimelineHumanGroup) -> Vec<String> {
        vec![fmt_tool_line("user", &group.content)]
    }

    fn render_skill_group(&self, group: &TimelineGroupData) -> Vec<String> {
        let skill_tag = group.skill_name.as_deref().unwrap_or(&group.agent_type);
        let mut lines = Vec::new();

        // Build status + stats summary for the content portion
        let mut parts = Vec::new();
        let status_str = match group.status {
            crate::types::SkillBlockStatus::Done => "Done",
            crate::types::SkillBlockStatus::Thinking => "Thinking",
            crate::types::SkillBlockStatus::Executing => "Executing",
            crate::types::SkillBlockStatus::Failed => "Failed",
            crate::types::SkillBlockStatus::Retried => "Retried",
        };
        parts.push(status_str.to_string());

        if let Some(ref stats) = group.stats {
            let stats_text = self.format_stats(stats);
            if !stats_text.is_empty() {
                parts.push(stats_text);
            }
        }

        let header_content = parts.join("  ");

        // Inherited tokens note
        if let Some(tokens) = group.inherited_tokens {
            lines.push(fmt_tool_line(
                skill_tag,
                &format!("{} (inherited {} tokens)", header_content, tokens.value()),
            ));
        } else {
            lines.push(fmt_tool_line(skill_tag, &header_content));
        }

        // Summary lines (from report title)
        if let Some(summary_text) = group.summary.as_deref().filter(|s| !s.is_empty()) {
            for line in summary_text.lines().take(3) {
                lines.push(fmt_cont_line(line));
            }
        }

        lines
    }

    fn render_ask_human_group(&self, group: &TimelineAskHumanGroup) -> Vec<String> {
        let mut lines = Vec::new();
        let label = format!("ask:{}", group.agent_type);
        lines.push(fmt_tool_line(&label, &group.question));
        for opt in &group.options {
            lines.push(fmt_cont_line(&format!("- {}", opt)));
        }
        if let Some(ref answer) = group.answer {
            lines.push(fmt_cont_line(&format!("auto-reply: {}", answer)));
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        GroupState, GroupStats, McpBlockData, McpBlockState, SkillBlockStatus, TimelineGroupData,
        TimelineHumanGroup, TokenSource,
    };
    use uuid::Uuid;

    fn render_skill(data: TimelineGroupData) -> Vec<String> {
        CliTimelineRenderer.render_skill_group(&data)
    }

    fn render_human(content: &str, timestamp: &str) -> Vec<String> {
        CliTimelineRenderer.render_human_group(&TimelineHumanGroup {
            content: content.to_string(),
            timestamp: timestamp.to_string(),
            agent_number: None,
            username: "user".to_string(),
            status_text: None,
        })
    }

    #[test]
    fn test_human_block_column_format() -> anyhow::Result<()> {
        let lines = render_human("Please scan the workspace", "11:22:33");
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("user"));
        assert!(lines[0].contains("│"));
        assert!(lines[0].contains("Please scan the workspace"));
        Ok(())
    }

    #[test]
    fn test_skill_summary_line() -> anyhow::Result<()> {
        let data = TimelineGroupData {
            agent_number: "616".to_string(),
            agent_type: "HubRis".to_string(),
            timestamp: "11:22:35".to_string(),
            skill_name: Some("hubris::task_decompose".to_string()),
            model_name: None,
            provider_label: None,
            status: SkillBlockStatus::Done,
            retry_count: 0,
            max_retries: 0,
            inherited_tokens: None,
            inherited_label: None,
            content_blocks: vec![],
            mcp_blocks: vec![],
            interleaved_blocks: vec![],
            stats: Some(GroupStats {
                input_tokens: Some(TokenSource::CloudResponse(156)),
                output_tokens: Some(TokenSource::CloudResponse(234)),
                duration_secs: Some(23.5),
                mcp_count: 1,
                exchange_count: Some(1),
            }),
            summary: None,
            is_error: false,
            state: GroupState::Finalized,
            gray_tail_len: 0,
            result_summary: None,
            retry_reason: None,
        };

        let lines = render_skill(data);

        println!("=== RENDERED OUTPUT ===");
        for line in &lines {
            println!("{}", line);
        }

        assert!(lines[0].contains("hubris::task"));
        assert!(lines[0].contains("Done"));
        assert!(lines[0].contains("in 156"));
        assert!(lines[0].contains("out 234"));
        assert!(lines[0].contains("23.5s"));
        Ok(())
    }

    #[test]
    fn test_skill_with_summary() -> anyhow::Result<()> {
        let data = TimelineGroupData {
            agent_number: "616".to_string(),
            agent_type: "HubRis".to_string(),
            timestamp: "11:22:35".to_string(),
            skill_name: Some("hubris::task_decompose".to_string()),
            model_name: None,
            provider_label: None,
            status: SkillBlockStatus::Done,
            retry_count: 0,
            max_retries: 0,
            inherited_tokens: Some(TokenSource::LocalStreamQuickAmount(512)),
            inherited_label: None,
            content_blocks: vec![],
            mcp_blocks: vec![],
            interleaved_blocks: vec![],
            stats: Some(GroupStats {
                input_tokens: Some(TokenSource::CloudResponse(1000)),
                output_tokens: Some(TokenSource::CloudResponse(500)),
                duration_secs: Some(15.0),
                mcp_count: 1,
                exchange_count: Some(1),
            }),
            summary: Some("Task decomposition complete".to_string()),
            is_error: false,
            state: GroupState::Finalized,
            gray_tail_len: 0,
            result_summary: None,
            retry_reason: None,
        };

        let lines = render_skill(data);

        println!("=== FULL SCENARIO ===");
        for line in &lines {
            println!("{}", line);
        }

        assert!(lines[0].contains("hubris::task"));
        assert!(lines[0].contains("inherited 512 tokens"));
        assert!(lines[0].contains("in 1.0k"));
        assert!(lines[0].contains("out 500"));
        assert!(lines[0].contains("15.0s"));

        // Summary on continuation line
        assert!(
            lines
                .iter()
                .any(|l| l.contains("Task decomposition complete"))
        );
        Ok(())
    }

    #[test]
    fn test_column_format_alignment() -> anyhow::Result<()> {
        let line = fmt_tool_line("exec", "OK (10ms)");
        assert!(line.contains("│"));
        // Right-aligned: spaces before "exec"
        assert!(line.starts_with(" "));
        assert!(line.contains("exec │ OK (10ms)"));
        Ok(())
    }

    #[test]
    fn test_mcp_block_rendering() -> anyhow::Result<()> {
        let mcp = McpBlockData {
            tool_name: "test_tool".to_string(),
            call_id: Uuid::now_v7(),
            agent_type: "haplotes".to_string(),
            call_text: "hello".to_string(),
            result_text: "world".to_string(),
            success: true,
            duration_ms: Some(100),
            state: McpBlockState::Done,
            separate_call_content: Vec::new(),
        };

        let lines = mcp_block::render_cli_mcp_block(&mcp);
        assert!(!lines.is_empty());
        assert!(lines[0].contains("haplotes::test_tool"));
        Ok(())
    }
}
