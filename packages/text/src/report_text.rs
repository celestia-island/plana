//! Markdown-aware report text hygiene shared by the report pipeline.
//!
//! Both report consumers — shittim-chest `packages/core` (the report bridge
//! that renders report cards) and entelecheia `packages/scepter`
//! (`report_dispatch.rs` / `report_extract.rs`, which summarize and translate
//! skill-chain reports) — need the same three text judgments, and each had
//! grown a private, drifting copy of them. Per the workspace dependency plan,
//! shared capability used by two or more services goes upstream first: this
//! module is that upstream home. It exists because report cards on
//! dev.celestia.world kept surfacing four classes of noise:
//!
//! 1. **Truncated teasers** — naive byte slicing cut mid-CJK-grapheme and
//!    dragged markdown table gutters (`|`) and code-fence bodies onto the
//!    card, or appended `...` on top of an already-over-budget string.
//! 2. **Translation-model reasoning** — the visible summary slot showed the
//!    model talking to itself about the task instead of performing it (live
//!    example: `我们只需要根据报告内容输出摘要…要注意字数。输出简体中文。`).
//! 3. **Raw machinery JSON** — exec-call tool payloads (`code`,
//!    `agent_name`, `tool_calls`, …), including wire-truncated fragments,
//!    rendered verbatim where a report should be.
//! 4. **Bare data JSON** — result blobs such as `{"polemos":[…]}` shown
//!    where a human-readable report was expected.
//!
//! [`plain_text_summary`] addresses (1), [`looks_like_llm_meta_text`]
//! addresses (2), [`classify_report_json`] addresses (3) and (4), and
//! [`is_markdown_structured`] picks the renderer for whatever survives.

use serde_json::Value;

/// Keys whose presence marks a JSON payload as agent machinery (tool calls,
/// exec payloads, chain-internal state) rather than a report for humans.
const MACHINERY_KEYS: [&str; 7] = [
    "code",
    "agent_name",
    "chain_step",
    "next_skill",
    "tool_calls",
    "arguments",
    "function",
];

/// Quoted forms of [`MACHINERY_KEYS`], matched against the raw text of JSON
/// that fails to parse (wire-truncated fragments still carry the fingerprint).
const MACHINERY_FINGERPRINTS: [&str; 7] = [
    "\"code\"",
    "\"agent_name\"",
    "\"chain_step\"",
    "\"next_skill\"",
    "\"tool_calls\"",
    "\"arguments\"",
    "\"function\"",
];

/// Keys a legitimate report envelope may carry as string-valued fields.
const ENVELOPE_KEYS: [&str; 5] = ["content", "text", "body", "summary", "title"];

/// Total envelope-unwrap depth budget: an envelope whose payload is itself an
/// envelope classifies the innermost one, capped at three levels.
const MAX_ENVELOPE_DEPTH: usize = 3;

/// Signals that alone prove the text is model self-talk (one hit fires).
const STRONG_META_SIGNALS: [&str; 27] = [
    "the user wants me to",
    "user wants me to",
    "i need to translate",
    "we need to translate",
    "let me translate",
    "i'll translate",
    "let me parse",
    "we need to parse",
    "i should provide",
    "my job is done",
    "the instruction says",
    "the prompt says",
    "let's produce",
    "let me produce",
    "i will now output",
    "output only the",
    "translate the following",
    "the text is:",
    "the source text is",
    "summarize the following report",
    "exactly 3 lines",
    "output only the summary",
    "summarize the report as",
    "two hundred characters",
    "output only text",
    "without any other content",
    "you are a report synthesizer",
];

/// Hedging / self-referential fragments that only indicate model self-talk
/// when at least two *distinct* ones appear in the same text.
const WEAK_META_SIGNALS: [&str; 26] = [
    "we need",
    "i need",
    "let me",
    "let's",
    "i think",
    "maybe",
    "probably",
    "the user",
    "as an ai",
    "translat",
    "summar",
    "under 300 char",
    "no more than",
    "characters total",
    "nothing else",
    "three lines",
    "state what was done",
    "most significant findings",
    "note any issues",
    "我们只需要",
    "我需要",
    "让我",
    "用户想要",
    "接下来我们",
    "字数",
    "输出简体中文",
];

// ─── Extractive summary ────────────────────────────────────────────────

/// Produces a flat, single-line teaser of a markdown report, at most
/// `max_chars` characters long.
///
/// Report cards on dev.celestia.world only have room for one teaser line,
/// and the legacy per-service copies cut on byte offsets — mid-CJK-grapheme,
/// through table gutters, and appended `...` that pushed the result past the
/// budget it was supposed to enforce. This function is the shared
/// replacement: it first *removes the markdown* so structure never leaks
/// onto the card, then truncates on character boundaries.
///
/// Markdown stripping, per line:
/// - fenced code blocks are dropped entirely (state toggles on lines that
///   start with ` ``` `; the fence markers themselves are dropped too),
/// - table rows (trimmed line starts with `|`) are dropped,
/// - leading `#` runs, leading `>` runs, unordered bullets (`- ` / `* ` /
///   `+ `) and ordered markers (`N. ` / `N) `) are stripped repeatedly, so
///   stacked markers such as `> - ## text` collapse fully,
/// - a line that is all dashes after stripping (a horizontal rule)
///   contributes nothing,
/// - remaining `*`, `` ` `` and `_` characters become spaces (word
///   separators, so `**bold**` and `snake_case` flatten to plain words),
/// - whitespace collapses to single spaces.
///
/// Truncation of the cleaned text (all slicing is char-based, never a byte
/// slice mid-character, so CJK is safe):
/// - if it fits within `max_chars`, it is returned verbatim;
/// - otherwise the cut prefers the **last sentence boundary** inside the
///   window — `。！？` terminate unconditionally, ASCII `.!?` only when
///   followed by whitespace or end of text, so `3.14` and `main.rs` never
///   split a sentence — returning a clean cut with no ellipsis;
/// - otherwise it cuts at the last whitespace inside the window and appends
///   `…`;
/// - otherwise it hard-cuts at `max_chars - 1` characters and appends `…`.
///
/// The ellipsis always counts toward the cap: the returned string is never
/// longer than `max_chars` characters. `max_chars == 0` yields an empty
/// string.
pub fn plain_text_summary(content: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let cleaned = collapse_markdown(content);
    let chars: Vec<char> = cleaned.chars().collect();
    if chars.len() <= max_chars {
        return cleaned;
    }

    // Preferred cut: the last sentence boundary inside the window.
    for i in (0..max_chars).rev() {
        if is_sentence_terminator(&chars, i) {
            return chars[..=i].iter().collect();
        }
    }

    // Second choice: the last whitespace inside the window.
    for i in (0..max_chars).rev() {
        if chars[i].is_whitespace() {
            let mut out: String = chars[..i].iter().collect();
            out.truncate(out.trim_end().len());
            out.push('…');
            return out;
        }
    }

    // Last resort: hard cut, keeping room for the ellipsis inside the cap.
    let mut out: String = chars[..max_chars - 1].iter().collect();
    out.push('…');
    out
}

/// Flattens markdown `content` into single-spaced prose, dropping fences,
/// tables, horizontal rules and block markers.
fn collapse_markdown(content: &str) -> String {
    let mut kept: Vec<&str> = Vec::new();
    let mut in_fence = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence || trimmed.starts_with('|') {
            continue;
        }
        let stripped = strip_line_markers(line).trim();
        if stripped.is_empty() || stripped.chars().all(|c| c == '-') {
            // Blank line or horizontal rule: contributes nothing.
            continue;
        }
        kept.push(stripped);
    }
    let joined = kept.join(" ");
    let separated: String = joined
        .chars()
        .map(|c| match c {
            '*' | '`' | '_' => ' ',
            _ => c,
        })
        .collect();
    separated
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

/// Repeatedly strips leading block markers so stacks (`> - ## text`)
/// collapse; returns the line's textual content.
fn strip_line_markers(line: &str) -> &str {
    let mut s = line;
    loop {
        let t = s.trim_start();
        s = if t.starts_with('#') {
            t.trim_start_matches('#')
        } else if t.starts_with('>') {
            t.trim_start_matches('>')
        } else if let Some(len) = unordered_bullet_len(t) {
            &t[len..]
        } else if let Some(len) = ordered_marker_len(t) {
            &t[len..]
        } else {
            return t;
        };
    }
}

/// Length of an unordered bullet marker (`- `, `* `, `+ `), if any.
fn unordered_bullet_len(t: &str) -> Option<usize> {
    let b = t.as_bytes();
    match b.first() {
        Some(b'-' | b'*' | b'+') if b.get(1) == Some(&b' ') => Some(2),
        _ => None,
    }
}

/// Length of an ordered-list marker (`N. ` / `N) ` with non-empty ASCII
/// digits), if any. Requires the space, so `3.14` is prose, not a list.
fn ordered_marker_len(t: &str) -> Option<usize> {
    let b = t.as_bytes();
    let digits = b.iter().take_while(|c| c.is_ascii_digit()).count();
    if digits > 0 && matches!(b.get(digits), Some(b'.' | b')')) && b.get(digits + 1) == Some(&b' ')
    {
        Some(digits + 2)
    } else {
        None
    }
}

/// Whether `chars[i]` ends a sentence: CJK `。！？` terminate
/// unconditionally; ASCII `.!?` only when followed by whitespace or the end
/// of text, so decimals (`3.14`) and file names (`main.rs`) never do.
fn is_sentence_terminator(chars: &[char], i: usize) -> bool {
    match chars[i] {
        '。' | '！' | '？' => true,
        '.' | '!' | '?' => chars.get(i + 1).is_none_or(|c| c.is_whitespace()),
        _ => false,
    }
}

// ─── LLM meta-text detection ───────────────────────────────────────────

/// Heuristically decides whether `text` is an LLM talking *about* the task
/// instead of performing it.
///
/// When the report pipeline asks a translation or summarization model for
/// output, some models emit their reasoning first — "we need to translate
/// the following…" / `我们只需要根据报告内容输出摘要…要注意字数。` — and the
/// report cards on dev.celestia.world rendered that chatter verbatim in the
/// summary slot. Consumers use this predicate to reject such output and fall
/// back to a deterministic summary.
///
/// Semantics: the text is lowercased and matched by substring. Any **one**
/// strong signal (explicit self-talk such as "the prompt says", "let me
/// translate", "you are a report synthesizer", …) fires. Otherwise, **two
/// distinct** weak signals (hedging fragments such as "i think", "maybe",
/// "translat", plus the CJK markers `让我`, `字数`, `输出简体中文`, …) must
/// fire — a single weak signal also occurs in legitimate prose (for example
/// "no more than three stations were offline").
///
/// `"final answer:"` is deliberately absent: it false-positives on
/// legitimate bare answers, so callers that want it must check for it
/// themselves.
pub fn looks_like_llm_meta_text(text: &str) -> bool {
    let lower = text.to_lowercase();
    for s in STRONG_META_SIGNALS {
        if lower.contains(s) {
            return true;
        }
    }
    let weak_hits = WEAK_META_SIGNALS
        .iter()
        .filter(|s| lower.contains(*s))
        .count();
    weak_hits >= 2
}

// ─── Tool-payload JSON classification ──────────────────────────────────

/// What kind of JSON a report-slot payload actually is.
///
/// Downstream rendering chooses its behavior per variant: machinery and bare
/// data must never be shown as report prose, envelopes carry the real report
/// text to extract, and `NotJson` is ordinary markdown/prose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReportJsonShape {
    /// Not JSON at all — plain prose or markdown (includes bare JSON
    /// scalars such as `"42"` or `"true"`, which carry no report).
    NotJson,
    /// A report envelope (only known payload keys as string fields); carries
    /// the extracted payload string, unwrapped to the innermost envelope.
    ReportEnvelope(String),
    /// Agent machinery: tool calls, exec payloads, chain-internal state.
    Machinery,
    /// Bare structured data with no report semantics (for example
    /// `{"polemos":[…]}`).
    BareData,
}

/// Classifies a report-slot payload as prose, report envelope, agent
/// machinery, or bare data.
///
/// Report cards on dev.celestia.world twice rendered raw agent plumbing in
/// the report slot: a full exec-call JSON (with `code` / `agent_name` /
/// `arguments` fields) and a wire-truncated fragment of the same. Both came
/// from tool payloads leaking through a "report" field, so consumers need
/// one shared gate that separates them from actual report content.
///
/// Semantics, in order:
/// 1. One leading ` ``` ` / ` ```lang ` fence is stripped (info-string line
///    skipped, missing closing fence tolerated, only at the very start
///    after trimming).
/// 2. The remainder is parsed as JSON. A bare string/number/bool/null
///    scalar is [`ReportJsonShape::NotJson`].
/// 3. An object with any machinery key ([`MACHINERY_KEYS`]), or an array
///    with any element object carrying one, is
///    [`ReportJsonShape::Machinery`].
/// 4. An object whose string-valued fields are all envelope payload keys
///    (`content` / `text` / `body` / `summary` / `title`; non-string fields
///    count as metadata) is a [`ReportJsonShape::ReportEnvelope`] carrying
///    the first present payload key's value, recursing this classification
///    into that string up to [`MAX_ENVELOPE_DEPTH`] levels total — an
///    envelope whose payload is itself an envelope classifies the
///    innermost. A structured (machinery / bare-data) payload keeps its
///    inner classification; a textual payload becomes the envelope content.
///    An object with no string payload key at all (for example
///    `{"polemos":[],"hubris":[]}`) is *not* an envelope.
/// 5. Anything else (including unparseable text without a machinery
///    fingerprint) maps to [`ReportJsonShape::BareData`] for arrays and
///    non-envelope objects, or [`ReportJsonShape::NotJson`] for
///    unparseable text.
/// 6. Unparseable text that still starts with `{` or `[` and contains a
///    quoted machinery key fingerprint (`"code"`, …) — the wire-truncated
///    exec-call case — is [`ReportJsonShape::Machinery`].
pub fn classify_report_json(text: &str) -> ReportJsonShape {
    classify_stripped(strip_leading_fence(text), 1)
}

/// Strips one leading code fence, tolerating a missing closing fence.
fn strip_leading_fence(text: &str) -> &str {
    let t = text.trim();
    let Some(rest) = t.strip_prefix("```") else {
        return t;
    };
    // Skip the fence's info-string line (```json, ```rust, …).
    let body = match rest.find('\n') {
        Some(i) => &rest[i + 1..],
        None => return "",
    };
    let body = body.trim_end();
    body.strip_suffix("```").map_or(body, str::trim_end)
}

/// Core classification on already fence-stripped text; `depth` counts
/// envelope-unwrap levels from 1.
fn classify_stripped(text: &str, depth: usize) -> ReportJsonShape {
    let t = text.trim();
    let value = match serde_json::from_str::<Value>(t) {
        Ok(v) => v,
        Err(_) => return unparseable_shape(t),
    };
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            ReportJsonShape::NotJson
        }
        Value::Array(items) => {
            if items.iter().any(is_machinery_object) {
                ReportJsonShape::Machinery
            } else {
                ReportJsonShape::BareData
            }
        }
        Value::Object(map) => {
            if map.keys().any(|k| MACHINERY_KEYS.contains(&k.as_str())) {
                return ReportJsonShape::Machinery;
            }
            match envelope_payload(&map) {
                Some(payload) => {
                    if depth < MAX_ENVELOPE_DEPTH {
                        match classify_stripped(payload, depth + 1) {
                            // Textual payload: the envelope *is* the report.
                            ReportJsonShape::NotJson => {
                                ReportJsonShape::ReportEnvelope(payload.to_owned())
                            }
                            // Structured payload hidden inside an envelope
                            // keeps its innermost classification.
                            inner => inner,
                        }
                    } else {
                        ReportJsonShape::ReportEnvelope(payload.to_owned())
                    }
                }
                None => ReportJsonShape::BareData,
            }
        }
    }
}

/// Whether `value` is an object carrying any machinery key.
fn is_machinery_object(value: &Value) -> bool {
    value
        .as_object()
        .is_some_and(|m| m.keys().any(|k| MACHINERY_KEYS.contains(&k.as_str())))
}

/// Returns the envelope payload string when every string-valued field of
/// `map` is a known payload key and one of those keys holds the payload.
fn envelope_payload(map: &serde_json::Map<String, Value>) -> Option<&str> {
    let strings_are_payload_keys = map
        .iter()
        .all(|(k, v)| !v.is_string() || ENVELOPE_KEYS.contains(&k.as_str()));
    if !strings_are_payload_keys {
        return None;
    }
    ENVELOPE_KEYS
        .iter()
        .find_map(|k| map.get(*k).and_then(Value::as_str))
}

/// Classifies JSON-ish text that failed to parse: truncated machinery still
/// carries a quoted key fingerprint.
fn unparseable_shape(t: &str) -> ReportJsonShape {
    let structural = t.starts_with('{') || t.starts_with('[');
    if structural && MACHINERY_FINGERPRINTS.iter().any(|f| t.contains(*f)) {
        ReportJsonShape::Machinery
    } else {
        ReportJsonShape::NotJson
    }
}

// ─── Structure gate ────────────────────────────────────────────────────

/// Cheaply decides whether `text` should render as markdown instead of flat
/// prose.
///
/// Report payloads that survived the gauntlet above are sometimes markdown
/// (an envelope carrying `## 最终报告`) and sometimes a plain sentence;
/// rendering markdown through a prose widget — or prose through a markdown
/// renderer that swallows line breaks — both look broken on the card. This
/// predicate gates the renderer choice.
///
/// True when any line (after leading whitespace) starts with `#` (heading),
/// `|` (table row), ` ``` ` (code fence), or is a list item: `- ` / `* ` /
/// `+ ` followed by a space, or `N. ` / `N) ` with non-empty ASCII digits.
/// The required space after the marker keeps `-5°C` and `3.14 …` prose.
pub fn is_markdown_structured(text: &str) -> bool {
    text.lines().any(|line| {
        let l = line.trim_start();
        l.starts_with('#')
            || l.starts_with('|')
            || l.starts_with("```")
            || unordered_bullet_len(l).is_some()
            || ordered_marker_len(l).is_some()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── plain_text_summary ────────────────────────────────────────────

    #[test]
    fn summary_drops_table_rows_but_keeps_prose() {
        let report = "巡检报告完成。\n\n| 节点 | 状态 |\n| --- | --- |\n| node-1 | 正常 |\n\n全部子系统运行正常。";
        let s = plain_text_summary(report, 200);
        assert!(!s.contains('|'), "table gutters must not leak: {s}");
        assert!(s.contains("巡检报告完成。"));
        assert!(s.contains("全部子系统运行正常。"));
    }

    #[test]
    fn summary_strips_fenced_code_body_and_markers() {
        let report = "Before.\n```rust\nfn main() {}\n```\nAfter.";
        let s = plain_text_summary(report, 200);
        assert!(!s.contains("```"), "fence markers must be gone: {s}");
        assert!(!s.contains("fn main"), "fence body must be gone: {s}");
        assert!(s.contains("Before."));
        assert!(s.contains("After."));
        // A report that is only a fence has no prose left at all.
        assert_eq!(plain_text_summary("```\ncode only\n```", 10), "");
    }

    #[test]
    fn summary_strips_block_markers_and_horizontal_rules() {
        let md = "## 标题\n\n> 引用行\n\n- 列表项\n\n1. 第一\n\n---\n\n正文";
        assert_eq!(plain_text_summary(md, 100), "标题 引用行 列表项 第一 正文");
    }

    #[test]
    fn summary_never_splits_decimals_or_filenames() {
        let text =
            "Fixed 3.14 in main.rs today. Additional work continues on the parser elsewhere.";
        // Sentence cut lands on the real sentence end, not on 3.14 / main.rs.
        let s = plain_text_summary(text, 30);
        assert!(s.ends_with("today."), "bad cut: {s}");
        assert!(!s.ends_with('…'));
        assert!(s.chars().count() <= 30);
        // Window ends inside the decimal: whitespace cut keeps `3.14` intact.
        let s = plain_text_summary(text, 12);
        assert_eq!(s, "Fixed 3.14…");
    }

    #[test]
    fn summary_cuts_cjk_on_sentence_boundary() {
        let s = plain_text_summary("第一句结束。第二句也很长很长很长很长", 8);
        assert_eq!(s, "第一句结束。");
        assert!(!s.contains('…'));
    }

    #[test]
    fn summary_cjk_truncation_is_char_safe() {
        let cjk = "一二三四五六七八九十百千万亿";
        for max in 1..=12 {
            let s = plain_text_summary(cjk, max);
            assert!(s.chars().count() <= max, "max={max} got {s:?}");
        }
        // Hard-cap path: exactly max_chars chars including the ellipsis.
        let s = plain_text_summary(cjk, 5);
        assert_eq!(s.chars().count(), 5);
        assert_eq!(s, "一二三四…");
    }

    #[test]
    fn summary_whitespace_cut_appends_ellipsis() {
        let s = plain_text_summary("alpha beta gamma delta", 11);
        assert_eq!(s, "alpha beta…");
        assert!(s.ends_with('…'));
    }

    #[test]
    fn summary_short_input_passes_through_verbatim() {
        assert_eq!(plain_text_summary("hello world", 50), "hello world");
        assert_eq!(plain_text_summary("", 10), "");
    }

    #[test]
    fn summary_zero_max_is_empty() {
        assert_eq!(plain_text_summary("anything at all", 0), "");
    }

    // ─── looks_like_llm_meta_text ──────────────────────────────────────

    #[test]
    fn meta_text_english_chatter_with_two_weak_signals_fires() {
        assert!(looks_like_llm_meta_text(
            "I think we need to review this further."
        ));
    }

    #[test]
    fn meta_text_single_strong_signal_fires() {
        assert!(looks_like_llm_meta_text("You are a report synthesizer."));
        assert!(looks_like_llm_meta_text(
            "Let me translate the whole thing."
        ));
        assert!(looks_like_llm_meta_text(
            "The prompt says to keep it short."
        ));
    }

    #[test]
    fn meta_text_clean_chinese_prose_does_not_fire() {
        assert!(!looks_like_llm_meta_text(
            "巡检已完成，三台设备全部正常运行，无需处理，建议保持观察频率。"
        ));
    }

    #[test]
    fn meta_text_live_chinese_chatter_fires() {
        // Verbatim from the dev.celestia.world incident.
        assert!(looks_like_llm_meta_text(
            "我们只需要根据报告内容输出摘要…要注意字数。输出简体中文。"
        ));
    }

    #[test]
    fn meta_text_single_cjk_marker_alone_does_not_fire() {
        assert!(!looks_like_llm_meta_text("字数统计功能已上线。"));
        assert!(!looks_like_llm_meta_text("让我看看这份表格。"));
    }

    #[test]
    fn meta_text_single_english_weak_signal_does_not_fire() {
        assert!(!looks_like_llm_meta_text(
            "no more than three stations were offline"
        ));
    }

    #[test]
    fn meta_text_final_answer_is_not_a_signal() {
        assert!(!looks_like_llm_meta_text("Final answer: 42"));
    }

    // ─── classify_report_json ──────────────────────────────────────────

    const EXEC_CALL: &str = r#"{"name":"execute","arguments":{"code":"import os\nos.system(\"df -h\")","agent_name":"scanner","chain_step":2},"next_skill":null}"#;

    #[test]
    fn classify_live_exec_call_json_is_machinery() {
        assert_eq!(classify_report_json(EXEC_CALL), ReportJsonShape::Machinery);
    }

    #[test]
    fn classify_truncated_exec_call_json_is_machinery() {
        let truncated = &EXEC_CALL[..EXEC_CALL.len() - 30];
        assert!(serde_json::from_str::<Value>(truncated).is_err());
        assert_eq!(classify_report_json(truncated), ReportJsonShape::Machinery);
    }

    #[test]
    fn classify_fenced_exec_call_json_is_machinery() {
        let fenced = format!("```json\n{EXEC_CALL}\n```");
        assert_eq!(classify_report_json(&fenced), ReportJsonShape::Machinery);
    }

    #[test]
    fn classify_report_envelope_carries_unescaped_markdown() {
        // The payload is JSON-source text with literal \n escapes.
        let payload = "## 最终报告\\n\\n| 节点 | 状态 |\\n| --- | --- |\\n| node-1 | 正常 |";
        let src = format!(r#"{{"content":"{}"}}"#, payload);
        match classify_report_json(&src) {
            ReportJsonShape::ReportEnvelope(s) => {
                assert!(s.starts_with("## "), "got {s:?}");
                assert!(s.contains('\n'), "escapes must be unescaped: {s:?}");
                assert!(s.contains("node-1"));
            }
            other => panic!("expected envelope, got {other:?}"),
        }
    }

    #[test]
    fn classify_double_wrapped_envelope_unwraps_innermost() {
        let inner = r#"{"summary":"全部系统正常"}"#;
        let outer = format!(r#"{{"content":"{}"}}"#, inner.replace('"', "\\\""));
        assert_eq!(
            classify_report_json(&outer),
            ReportJsonShape::ReportEnvelope("全部系统正常".to_owned())
        );
    }

    #[test]
    fn classify_envelope_unwrap_depth_is_capped_at_three() {
        fn wrap(payload: &str) -> String {
            let escaped = payload.replace('\\', "\\\\").replace('"', "\\\"");
            format!(r#"{{"content":"{}"}}"#, escaped)
        }
        let mut text = "deepest".to_owned();
        for _ in 0..3 {
            text = wrap(&text);
        }
        // Three envelope levels: all three unwrap, exposing the raw text.
        assert_eq!(
            classify_report_json(&text),
            ReportJsonShape::ReportEnvelope("deepest".to_owned())
        );
        // A fourth level exceeds the budget: the depth-3 payload comes back raw.
        let quad = wrap(&text);
        assert_eq!(
            classify_report_json(&quad),
            ReportJsonShape::ReportEnvelope(wrap("deepest"))
        );
    }

    #[test]
    fn classify_machinery_hidden_in_envelope_is_machinery() {
        let src = r#"{"content":"{\"code\":\"import os\",\"agent_name\":\"scanner\"}"}"#;
        assert_eq!(classify_report_json(src), ReportJsonShape::Machinery);
    }

    #[test]
    fn classify_bare_result_data_is_bare_data() {
        assert_eq!(
            classify_report_json(r#"{"polemos":[],"hubris":[]}"#),
            ReportJsonShape::BareData
        );
        assert_eq!(classify_report_json("[1,2,3]"), ReportJsonShape::BareData);
    }

    #[test]
    fn classify_plain_markdown_and_prose_are_not_json() {
        assert_eq!(
            classify_report_json("## 报告\n\n- 项目一\n"),
            ReportJsonShape::NotJson
        );
        assert_eq!(classify_report_json("just prose"), ReportJsonShape::NotJson);
        assert_eq!(classify_report_json("42"), ReportJsonShape::NotJson);
        assert_eq!(classify_report_json("\"quoted\""), ReportJsonShape::NotJson);
    }

    #[test]
    fn classify_summary_only_envelope() {
        assert_eq!(
            classify_report_json(r#"{"summary":"x"}"#),
            ReportJsonShape::ReportEnvelope("x".to_owned())
        );
    }

    #[test]
    fn classify_envelope_allows_non_string_metadata_fields() {
        assert_eq!(
            classify_report_json(r#"{"content":"报告正文","id":7}"#),
            ReportJsonShape::ReportEnvelope("报告正文".to_owned())
        );
    }

    // ─── is_markdown_structured ────────────────────────────────────────

    #[test]
    fn markdown_structure_positives() {
        assert!(is_markdown_structured("# Heading"));
        assert!(is_markdown_structured("plain\n| a | b |"));
        assert!(is_markdown_structured("```rust\nfn x() {}\n```"));
        assert!(is_markdown_structured("- bullet"));
        assert!(is_markdown_structured("* bullet"));
        assert!(is_markdown_structured("+ bullet"));
        assert!(is_markdown_structured("1. first"));
        assert!(is_markdown_structured("2) second"));
        assert!(is_markdown_structured("  - indented bullet"));
    }

    #[test]
    fn markdown_structure_negatives() {
        assert!(!is_markdown_structured("just plain prose here"));
        assert!(!is_markdown_structured("-5°C outside"));
        assert!(!is_markdown_structured("3.14 is approximately pi"));
        assert!(!is_markdown_structured(""));
    }
}
