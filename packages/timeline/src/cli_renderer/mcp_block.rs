use super::super::{
    chars::*,
    types::{McpBlockData, McpBlockState},
};

const EXEC: &str = "exec";
const WRITE_TO_VAR: &str = "write_to_var";
const WRITE_TO_VAR_JSON: &str = "write_to_var_json";

const MAX_BLOCK_LINES: usize = 14;
const WTV_MAX_LINES: usize = 16;
const WTV_MAX_CHARS: usize = 2000;

pub(super) fn extract_cli_exec_code(call_text: &str) -> String {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(call_text) {
        val.get("code")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| call_text.to_string())
    } else {
        call_text.to_string()
    }
}

fn truncate_text_preview(
    content: &str,
    max_lines: usize,
    max_chars: usize,
) -> (String, bool, usize) {
    let char_count = content.chars().count();
    let line_count = content.lines().count();
    let needs_truncation = line_count > max_lines || char_count > max_chars;
    if !needs_truncation {
        return (content.to_string(), false, char_count);
    }
    let mut preview_lines: Vec<&str> = Vec::new();
    let mut char_acc = 0usize;
    for line in content.lines() {
        if preview_lines.len() >= max_lines || char_acc + line.chars().count() > max_chars {
            break;
        }
        char_acc += line.chars().count();
        preview_lines.push(line);
    }
    let preview = if preview_lines.is_empty() {
        content.chars().take(max_chars).collect()
    } else {
        preview_lines.join("\n")
    };
    (preview, true, char_count)
}

pub(super) fn render_cli_mcp_block(mcp: &McpBlockData) -> Vec<String> {
    let is_exec = mcp.tool_name == EXEC;
    let is_wtv = mcp.tool_name == WRITE_TO_VAR || mcp.tool_name == WRITE_TO_VAR_JSON;
    let mut lines = Vec::new();
    if is_exec {
        lines.push("  Execute Script".to_string());
    } else if mcp.state == McpBlockState::HistoryLost {
        lines.push("  History Lost".to_string());
    } else if is_wtv {
        let var_name = mcp
            .call_text
            .lines()
            .next()
            .and_then(|l| serde_json::from_str::<serde_json::Value>(l).ok())
            .and_then(|v| {
                v.get("var_name")
                    .and_then(|n| n.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "?".to_string());
        let label = if mcp.tool_name == WRITE_TO_VAR_JSON {
            "Write JSON to variable"
        } else {
            "Write text to variable"
        };
        lines.push(format!("  {}：{}", label, var_name));
    } else {
        lines.push(format!("  {}::{}", mcp.agent_type, mcp.tool_name));
    }
    if mcp.state == McpBlockState::Pending {
        lines.push("    ⠋ Waiting for parameters...".to_string());
    } else if !mcp.call_text.is_empty() {
        if is_exec {
            let display_text = extract_cli_exec_code(&mcp.call_text);
            for param_line in display_text.lines() {
                lines.push(format!("    {}", param_line));
            }
        } else if is_wtv {
            if !mcp.separate_call_content.is_empty() {
                for (_label, content) in &mcp.separate_call_content {
                    let (preview, needs_truncation, char_count) =
                        truncate_text_preview(content, WTV_MAX_LINES, WTV_MAX_CHARS);
                    for content_line in preview.lines() {
                        lines.push(format!("    {}", content_line));
                    }
                    if needs_truncation {
                        lines.push(format!("    ... ({} chars)", char_count));
                    }
                }
            }
        } else {
            for param_line in mcp.call_text.lines() {
                lines.push(format!("    {}", param_line));
            }
        }
    }
    let has_call = !mcp.call_text.is_empty() || mcp.state == McpBlockState::Pending;
    let has_result = !mcp.result_text.is_empty() && mcp.state != McpBlockState::HistoryLost;
    if has_call && has_result {
        lines.push("  --".to_string());
    }
    if has_result {
        let remaining = MAX_BLOCK_LINES.saturating_sub(lines.len());
        let remaining = remaining.max(3);
        let total_chars = mcp.result_text.chars().count();
        let total_lines = mcp.result_text.lines().count();
        let needs_trunc = total_lines > remaining || total_chars > 2000;

        let display_text = if needs_trunc {
            let (preview, _, _) = truncate_text_preview(&mcp.result_text, remaining, 2000);
            preview
        } else if mcp.state == McpBlockState::Failed {
            mcp.result_text.clone()
        } else if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&mcp.result_text) {
            if parsed.is_string() {
                parsed.as_str().unwrap_or(&mcp.result_text).to_string()
            } else if parsed.is_null() {
                String::new()
            } else {
                serde_json::to_string_pretty(&parsed).unwrap_or_default()
            }
        } else {
            mcp.result_text.clone()
        };

        for line in display_text.lines() {
            lines.push(format!("    {}", line));
        }

        if needs_trunc {
            lines.push(format!("    ... ({} chars)", total_chars));
        }
    }
    let label = mcp.close_label();
    let call_tokens = (mcp.call_text.len() as u64).div_ceil(4);
    let result_tokens = (mcp.result_text.len() as u64).div_ceil(4);
    let char_info = match mcp.duration_ms {
        Some(d) => {
            let mut parts = Vec::new();
            if call_tokens > 0 || result_tokens > 0 {
                let fmt_tok = |v: u64| -> String {
                    if v >= 1000 {
                        format!("{:.1}k", v as f64 / 1000.0)
                    } else {
                        v.to_string()
                    }
                };
                parts.push(format!("{} {}", ARROW_UP, fmt_tok(call_tokens)));
                if result_tokens > 0 {
                    parts.push(format!("{} {}", ARROW_DOWN, fmt_tok(result_tokens)));
                }
            }
            if parts.is_empty() {
                format!("{}ms {}", d, label)
            } else {
                format!("{} {}ms", parts.join(" "), d)
            }
        },
        None => format!("{}", label),
    };
    lines.push(format!("  {}", char_info));
    lines
}
