//! Markdown-aware report text hygiene shared by the report pipeline.
//!
//! Both report consumers — shittim-chest `packages/core` (the report bridge
//! that renders report cards) and entelecheia `packages/scepter`
//! (`report_dispatch.rs` / `report_extract.rs`, which summarize and translate
//! skill-chain reports) — need the same three text judgments, and each had
//! grown a private, drifting copy of them. Per the workspace dependency plan,
//! shared capability used by two or more services goes upstream first: this
//! module is that upstream home. It exists because report cards on
//! dev.celestia.world kept surfacing five classes of noise:
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
//! 5. **Stringified machinery blobs** — JSON-stringified chain-state
//!    echoes and sandbox console transcripts that no longer OPEN as a
//!    JSON object (`write_to_var` envelope echoes, `__vars['x'] set …`
//!    confirmations, `... (N chars)` truncation markers, literal `\n` /
//!    `\"` escapes), so every brace-anchored gate against (3)/(4) misses
//!    them (live example: the 2026-10-01 `reflect_on_output` card).
//!
//! [`plain_text_summary`] addresses (1), [`looks_like_llm_meta_text`]
//! addresses (2), [`classify_report_json`] addresses (3) and (4),
//! [`looks_like_stringified_machinery`] addresses (5), and
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

// ─── Stringified-machinery detection ─────────────────────────────

/// Whether `text` is a *stringified* machinery blob: a JSON-stringified
/// chain-state echo or a sandbox console transcript that no longer opens
/// as a JSON object — the one shape every brace-anchored gate above
/// misses (live case 2026-10-01: the `reflect_on_output` card rendered a
/// one-line `JSON.stringify`-style mash of `write_to_var` envelope
/// echoes, `__vars['x'] set …` confirmations and `... (N chars)`
/// truncation markers, with literal `\n` / `\"` escapes where newlines
/// and quotes should be — it classified as `NotJson` prose and sailed
/// through both consumers' report paths).
///
/// Semantics (all position-independent). Quoted code is removed first —
/// fenced blocks AND inline backtick spans — so a report *quoting* a
/// transcript sample is judged by its prose alone. A bare inline
/// machinery quote (no fence, no backticks) is deliberately still
/// condemned: the fingerprints below are the strongest signal this class
/// has, and the worst case is a placeholder card — the safe error
/// direction for a user-facing surface.
///
/// 1. A sandbox transcript confirmation marker — `vars['…'] set`
///    (the skemma runtime's `write_to_var` echo, in its `__vars`,
///    `$.vars` and front-clipped `_vars` forms, with the real
///    `'] set:` / `'] set (` suffix) — never appears in prose.
/// 2. An ESCAPED envelope key — `\"var_name\":` / `\"skill_name\":`
///    after one escape-level collapse, so the single- and
///    double-escaped forms fold together; prose never carries `\"`.
///    A plain compact key (`"var_name":"` with no escaped quotes)
///    only counts next to a `... (N chars)` truncation marker, and a
///    spaced `"var_name": "x"` mention never matches — those two are
///    how a report may *mention* the envelope shape without being one.
/// 3. Pure escape density: a 200+-char text with NO real newlines but
///    three or more literal `\n` escapes and a literal `\"` (or a
///    truncation marker) is a stringified JSON body, not prose.
///
/// Accepted misses (documented after the 2026-10-01 R1/R2 reviews, all
/// shapes no live producer has emitted): Python-style spaced
/// serialization (`"var_name": "rep"` — JSON.stringify is compact),
/// double-quoted vars (`__vars["x"]`), a single-level plain stringify
/// carrying neither transcript marker nor truncation marker, keys
/// escaped three-plus levels deep, and machinery sitting ON the info
/// string of a fence opener that never closes (the tail lines are still
/// scanned; only the opener line itself is dropped).
pub fn looks_like_stringified_machinery(text: &str) -> bool {
    let prose = without_quoted_code(text);
    let prose = prose.trim_end();
    // (1) transcript confirmation marker.
    if has_vars_confirmation_marker(prose) {
        return true;
    }
    let truncated = has_chars_truncation_marker(prose);
    // (2) escaped envelope keys: one escape-level collapse folds
    // `\\"var_name\\":` into `\"var_name\":`, so both serialized forms
    // match the single-backslash needle.
    let one_level = collapse_one_escape_level(prose);
    const ESCAPED_ENVELOPE_KEYS: [&str; 2] = ["\\\"var_name\\\":", "\\\"skill_name\\\":"];
    if ESCAPED_ENVELOPE_KEYS.iter().any(|k| one_level.contains(k)) {
        return true;
    }
    // …or a plain compact key, but only next to a truncation marker.
    let unescaped: String = prose.chars().filter(|c| *c != '\\').collect();
    const PLAIN_ENVELOPE_KEYS: [&str; 2] = ["\"var_name\":\"", "\"skill_name\":\""];
    if truncated && PLAIN_ENVELOPE_KEYS.iter().any(|k| unescaped.contains(k)) {
        return true;
    }
    // (3) escape density on a single-line blob.
    prose.chars().count() >= 200
        && !prose.contains('\n')
        && prose.matches("\\n").count() >= 3
        && (prose.contains("\\\"") || truncated)
}

/// The skemma runtime's `write_to_var` confirmation echo:
/// `vars['<name>'] set …` (covering the `__vars`, `$.vars` and
/// front-clipped `_vars` forms — the needle is matched without its
/// prefix). The confirmation suffix must be the real thing —
/// `'] set:` / `'] set (` — so prose like `vars['x'] setting` does
/// not pair, and the var name is scanned within a bounded window so
/// an unrelated suffix later in the text cannot either.
fn has_vars_confirmation_marker(text: &str) -> bool {
    let mut rest = text;
    while let Some(i) = rest.find("vars['") {
        let after = &rest[i + "vars['".len()..];
        let window: String = after.chars().take(64).collect();
        if window.contains("'] set:") || window.contains("'] set (") {
            return true;
        }
        rest = after;
    }
    false
}

/// `... (N chars)` — the skemma runtime's write_to_var truncation
/// suffix. What FOLLOWS it is part of the fingerprint: inside a
/// stringified blob the marker rides against the escaped quotes of the
/// echo it cut (`\`, `"`, `,`, `}`, `]`) or lands at a line/text end; a
/// natural-language "(500 chars)" phrase continues with prose
/// punctuation and does not count.
fn has_chars_truncation_marker(text: &str) -> bool {
    let mut rest = text;
    while let Some(i) = rest.find("... (") {
        let tail = &rest[i + "... (".len()..];
        let digits = tail.bytes().take_while(u8::is_ascii_digit).count();
        if digits > 0 && tail[digits..].starts_with(" chars)") {
            let after = tail[digits + " chars)".len()..].chars().next();
            match after {
                None | Some('\n') | Some('\\') | Some('"') | Some(',') | Some('}') | Some(']') => {
                    return true;
                }
                _ => {}
            }
        }
        rest = tail;
    }
    false
}

/// Collapse one level of backslash escaping (`\\` → `\`), folding the
/// double-escaped forms of a stringified-inside-stringified blob onto
/// the single-escaped ones. Lone backslashes pass through untouched.
fn collapse_one_escape_level(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && chars.peek() == Some(&'\\') {
            chars.next();
        }
        out.push(c);
    }
    out
}

/// Drop quoted code — fenced blocks (``` … ```) and inline backtick
/// spans — keeping the prose around them, so a report *quoting* a
/// transcript sample is judged by its prose alone. An UNCLOSED fence
/// does not swallow the tail: these blobs are truncation products, so a
/// fence that never closes is treated as never having opened (its lines
/// are scanned as prose).
fn without_quoted_code(text: &str) -> String {
    let mut kept: Vec<String> = Vec::new();
    let mut fenced: Vec<&str> = Vec::new();
    let mut in_fence = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            if in_fence {
                // A closing fence: discard the buffered body.
                fenced.clear();
                in_fence = false;
            } else {
                in_fence = true;
            }
            continue;
        }
        if in_fence {
            fenced.push(line);
        } else {
            kept.push(strip_inline_code_spans(line));
        }
    }
    if in_fence {
        // Unclosed fence: scan the tail as prose after all.
        kept.extend(fenced.iter().map(|l| strip_inline_code_spans(l)));
    }
    kept.join("\n")
}

/// Remove PAIRED single-backtick inline code spans from one line. An
/// unpaired backtick is literal prose (markdown renders it as typed), so
/// an unclosed span's buffered content is restored rather than dropped
/// (R2 edge: a stray tick used to hide the rest of its line from the
/// machinery scan).
fn strip_inline_code_spans(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut span: Option<String> = None;
    for c in line.chars() {
        if c == '`' {
            if span.take().is_none() {
                span = Some(String::new());
            }
            continue;
        }
        match span.as_mut() {
            Some(buffered) => buffered.push(c),
            None => out.push(c),
        }
    }
    // Unclosed span: the opening tick was prose — keep its content.
    if let Some(buffered) = span {
        out.push_str(&buffered);
    }
    out
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

    // ─── looks_like_stringified_machinery ─────────────────────────

    /// High-fidelity reconstruction of the 2026-10-01 `reflect_on_output`
    /// card body: a front-clipped, one-line stringified mash of
    /// `write_to_var` envelope echoes, transcript confirmations and
    /// truncation markers (literal `\n` / `\"` escapes throughout).
    const STRINGIFIED_TRANSCRIPT_BLOB: &str = "report on data persistence and system integrity\\\\n\\\\nAcceptance Criteria:\\\\n- Final report includes: status summary, key findings, and recommendations\\\\n- All findings are supported by evidence from previous phases\\\\n- Report is concise, technically accurate, and free of contradictions\\\\n\\\\nRisk Factors:\\\\n- Misinterpretation of data due to ambiguous results (medium probability, medium impact)\\\\n- Incomplete reporting due to oversight (low probability, medium impact)\\\\n\\\\n### Total Estimated Time: 2.0 hours\\\\n\\\\n### Critical Path: Phase 1 → Phase 2 → Phase 3\\\\n\\\\n### Risk Register\\\\n| Risk | Probability | Impact | Mitigation |\\\\n|------|-----------|--------|------------|\\\\n| Index corruption | Low | Medium | Re-index if status indicates failure |\\\\n| Missing records | Medium | High | Cross-verify with backup or source |\\\\n\\\\n### Next Step\\\\nThe work plan has been generated and is ready for execution. The plan_execute skill will now implement the plan, applying the workspace_status() and ragSearch() tools in sequence to verify system state and validate data persistence.\\\"},\\\"var_name\\\":\\\"rep\\\"}\n__vars['rep'] set:\n## Work Plan: System Status Analysis\\\\n\\\\n### Phase 1: System State Verification (Duration: 0.5h)\\\\n\\\\nGoal: Confirm the integrity and availability of the workspace index.\\\\n... (3400 chars)\\\",\\\"skill_name\\\":\\\"workplan_generate\\\"},\\\"var_name\\\":\\\"input_data\\\"}\n__vars['input_data'] set (parsed JSON): object with 6 key(s)";

    #[test]
    fn stringified_live_reflect_on_output_card_is_machinery() {
        // The blob must NOT parse as JSON and must NOT open with a brace —
        // those are exactly why the brace-anchored gates missed it.
        assert!(serde_json::from_str::<Value>(STRINGIFIED_TRANSCRIPT_BLOB).is_err());
        assert!(!STRINGIFIED_TRANSCRIPT_BLOB.trim_start().starts_with('{'));
        assert_eq!(
            classify_report_json(STRINGIFIED_TRANSCRIPT_BLOB),
            ReportJsonShape::NotJson,
            "classifier still sees prose — the stringified gate is the catch"
        );
        assert!(looks_like_stringified_machinery(
            STRINGIFIED_TRANSCRIPT_BLOB
        ));
    }

    #[test]
    fn stringified_vars_confirmation_marker_alone_fires() {
        assert!(looks_like_stringified_machinery(
            "## 报告\n\n节点正常。\n__vars['input_data'] set (parsed JSON): object with 6 key(s)"
        ));
        assert!(looks_like_stringified_machinery(
            "$.vars['reply_payload'] set (parsed JSON): object with 2 keys"
        ));
        // Front-clipped form: the leading underscores are gone too.
        assert!(looks_like_stringified_machinery(
            "…echo tail…_vars['rep'] set:\n## Work Plan"
        ));
    }

    #[test]
    fn stringified_escaped_envelope_keys_fire() {
        // Single-escaped key form (the serialized envelope riding inside
        // a larger blob).
        assert!(looks_like_stringified_machinery(
            "…clipped head…\\n\\\",\\\"var_name\\\":\\\"rep\\\"} trailing fragment"
        ));
        // Escaped form plus truncation marker; skill_name is the ONLY
        // envelope key present (mutation gap b: the key must stand alone).
        assert!(looks_like_stringified_machinery(
            "preview tail... (3400 chars)\\\",\\\"skill_name\\\":\\\"workplan_generate\\\"}"
        ));
    }

    #[test]
    fn stringified_plain_compact_key_needs_the_truncation_marker() {
        // A plain compact key WITHOUT any escaped quotes is a mention,
        // not a serialization (mutation gap c: rule 2's conjunction is
        // the only thing keeping this prose safe).
        assert!(!looks_like_stringified_machinery(
            "存储格式为 {\"var_name\":\"rep\"}，调用方负责填充。"
        ));
        // …but next to a truncation marker it is the cut edge of a
        // serialized envelope.
        assert!(looks_like_stringified_machinery(
            "{\"content\":\"## 计划... (3400 chars)\",\"var_name\":\"rep\"}"
        ));
    }

    #[test]
    fn stringified_double_escaped_keys_fold_and_fire() {
        // One escape level up: the double-escaped form (text
        // `\\"var_name\\":\\"rep\\"`) folds onto the single-escaped
        // needle by the one-level collapse.
        assert!(looks_like_stringified_machinery(
            "outer string embedding \\\\\"var_name\\\\\":\\\\\"rep\\\\\" inside"
        ));
    }

    #[test]
    fn stringified_escape_density_alone_fires() {
        // One 200+ char line, no real newlines, dense literal \n + \".
        let blob = format!(
            "clipped mid-sentence {}\\n{}\\n{}\\nend\\\"",
            "x".repeat(120),
            "y".repeat(80),
            "z".repeat(40)
        );
        assert!(!blob.contains('\n'));
        assert!(looks_like_stringified_machinery(&blob));
    }

    #[test]
    fn stringified_clean_markdown_report_does_not_fire() {
        let report = "## 巡检报告\n\n| 节点 | 状态 |\n| --- | --- |\n| node-1 | 正常 |\n\n- 全部子系统运行正常。\n- 建议保持观察频率。\n";
        assert!(!looks_like_stringified_machinery(report));
    }

    #[test]
    fn stringified_fenced_transcript_quote_does_not_fire() {
        // A report QUOTING a transcript sample inside a fence is prose
        // about the machinery, not machinery itself.
        let report = "## 调试记录\n\n沙箱确认行形如：\n\n```text\n__vars['input_data'] set (parsed JSON): object with 6 key(s)\n```\n\n结论：行为符合预期。";
        assert!(!looks_like_stringified_machinery(report));
    }

    #[test]
    fn stringified_spaced_key_mention_does_not_fire() {
        // A spaced `"var_name": "x"` mention (not the serialized form).
        assert!(!looks_like_stringified_machinery(
            "参数对象形如 { \"var_name\": \"rep\" }，调用方负责填充。"
        ));
    }

    #[test]
    fn stringified_long_single_line_prose_does_not_fire() {
        let line = format!("巡检完成，一切正常。{}", "确认无误。".repeat(60));
        assert!(!looks_like_stringified_machinery(&line));
    }

    #[test]
    fn stringified_windows_paths_do_not_fire() {
        // Backslash-n inside Windows paths is not an escape density signal
        // without escaped quotes or a truncation marker.
        let paths = (0..10)
            .map(|i| format!("C:\\new\\node_{i}\\next\\bin"))
            .collect::<Vec<_>>()
            .join(" ");
        assert!(paths.chars().count() >= 200);
        assert!(!looks_like_stringified_machinery(&paths));
    }

    #[test]
    fn stringified_vars_marker_requires_the_set_suffix() {
        // `_vars['x']` mentioned without the `'] set` confirmation is prose.
        assert!(!looks_like_stringified_machinery(
            "模型随后读取 _vars['rep'] 的内容并继续推理，没有产生副作用。"
        ));
    }

    #[test]
    fn stringified_backtick_quoted_marker_does_not_fire() {
        // Inline backtick spans get the same protection as fenced
        // blocks: quoting the echo is prose about machinery.
        assert!(!looks_like_stringified_machinery(
            "调试发现沙箱回显 `__vars['rep'] set: {...}` 之后紧跟完整 JSON，属于预期行为。"
        ));
    }

    #[test]
    fn stringified_bare_inline_quote_is_condemned_by_design() {
        // The deliberate counterpart: a BARE inline quote (no fence, no
        // backticks) still fires — the transcript marker is the
        // strongest signal this class has, and the cost of a wrong
        // condemnation is only a placeholder card (documented accepted
        // FP, 2026-10-01 R1 review).
        assert!(looks_like_stringified_machinery(
            "调试发现沙箱回显 __vars['rep'] set: 之后紧跟完整 JSON，属于预期行为。"
        ));
    }

    #[test]
    fn stringified_unclosed_fence_tail_is_still_scanned() {
        // These blobs are truncation products, so a fence that never
        // closes must not swallow the tail it opens.
        assert!(looks_like_stringified_machinery(
            "## 报告\n\n正文完毕。\n\n```text\n__vars['x'] set: 噪声尾巴"
        ));
        // …and a properly closed fence still protects the quote.
        assert!(!looks_like_stringified_machinery(
            "## 报告\n\n正文完毕。\n\n```text\n__vars['x'] set: 引文\n```\n\n结论正常。"
        ));
    }

    #[test]
    fn stringified_unpaired_backtick_keeps_the_rest_of_the_line() {
        // R2 edge: a stray (unclosed) backtick is prose, so what follows
        // it must stay visible to the scan — machinery there still fires.
        assert!(looks_like_stringified_machinery(
            "调试记录 `引号开始 __vars['rep'] set: 之后紧跟完整 JSON"
        ));
        // …and a properly PAIRED span still protects the quote.
        assert!(!looks_like_stringified_machinery(
            "调试发现 `__vars['rep'] set: x` 属于预期行为。"
        ));
    }

    #[test]
    fn stringified_natural_chars_phrase_does_not_fire() {
        // A natural-language "(500 chars)" phrase continues with prose
        // punctuation — only markers riding against escape/quote edges
        // count (Windows paths supply the \n density here).
        let paths = (0..10)
            .map(|i| format!("C:\\new\\node_{i}\\next\\bin"))
            .collect::<Vec<_>>()
            .join(" ");
        let line = format!("{paths} 总计... (500 chars)。");
        assert!(line.chars().count() >= 200);
        assert!(!looks_like_stringified_machinery(&line));
    }
}
