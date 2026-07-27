//! Column-format utilities for CLI timeline output.
//!
//! Produces lines matching: `<right-aligned label> │ <left-aligned content>`
//! where the label column has a fixed width for visual alignment.

const COL_W: usize = 20;
pub const COL_PAD: &str = "                     │ ";

fn truncate_label(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let target = max.saturating_sub(3);
        let mut end = 0;
        for (i, ch) in s.char_indices() {
            if i >= target {
                break;
            }
            end = i + ch.len_utf8();
        }
        format!("{}...", &s[..end])
    }
}

pub fn fmt_tool_line(name: &str, content: &str) -> String {
    let padded = if name.chars().count() > COL_W {
        // truncate_label already appends "..."
        let truncated = truncate_label(name, COL_W);
        if truncated.chars().count() < COL_W {
            format!("{:>w$}", truncated, w = COL_W)
        } else {
            truncated
        }
    } else {
        format!("{:>w$}", name, w = COL_W)
    };
    if content.is_empty() {
        format!("{} │", padded)
    } else {
        format!("{} │ {}", padded, content)
    }
}

pub fn fmt_cont_line(content: &str) -> String {
    format!("{}{}", COL_PAD, content)
}

pub fn fmt_truncate(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let target = max.saturating_sub(3);
        let mut end = 0;
        for (i, ch) in s.char_indices() {
            if i >= target {
                break;
            }
            end = i + ch.len_utf8();
        }
        format!("{}...", &s[..end])
    }
}
