//! Shared SSE (Server-Sent Events) parsing utilities.
//!
//! All four LLM providers (OpenAI Compatible, Gemini, Anthropic, OpenAI
//! Responses) replaced `reqwest-eventsource` 0.6 (which silently drops POST
//! bodies) with manual SSE parsing.  This module centralises the common
//! buffer-and-split logic so each provider only implements the JSON
//! deserialisation specific to its API format.
//!
//! # Stream hygiene (audit R1-LP-1/2)
//!
//! A user-configured endpoint can send bytes that never form valid UTF-8,
//! CRLF-only framing, or one endless event. The helpers here keep such a
//! stream from wedging a provider:
//!
//! - [`decode_complete_utf8_prefix`] resynchronises on permanently
//!   invalid bytes (U+FFFD, per the WHATWG decoding the SSE spec
//!   mandates) instead of leaving them stuck at the head of the buffer
//!   forever — the pre-fix behavior stalled the stream AND grew the
//!   buffer without bound.
//! - [`normalize_sse_buffer`] folds CRLF/CR line endings to LF so
//!   CRLF-framed events still split.
//! - [`MAX_SSE_BUFFER_BYTES`] is the cap providers enforce on their
//!   buffers; beyond it the stream is an error, not an allocation
//!   growth chart.

/// The cap on one provider's SSE decode buffers (`state.buffer` +
/// `state.raw_bytes` together). Real LLM events are kilobytes; 8 MiB is
/// generosity with a margin.
pub const MAX_SSE_BUFFER_BYTES: usize = 8 * 1024 * 1024;

/// Fold CRLF and bare-CR line endings to LF, in place.
///
/// The SSE spec allows CRLF, CR and LF as line terminators; this parser
/// splits events on `\n\n`. A CRLF-only endpoint therefore never forms
/// the separator and the buffer grows forever — normalise before
/// splitting. Runs at most one allocation per call and only when the
/// chunk actually contains a CR.
pub fn normalize_sse_buffer(buffer: &mut String) {
    if buffer.contains('\r') {
        *buffer = buffer.replace("\r\n", "\n").replace('\r', "\n");
    }
}

/// Whether `len` bytes of pending decode buffer are over the cap.
#[must_use]
pub fn over_buffer_cap(len: usize) -> bool {
    len > MAX_SSE_BUFFER_BYTES
}

/// Extract complete SSE data events from a buffer.
///
/// SSE events are delimited by `\n\n`.  Each event may contain multiple
/// lines; lines starting with `data: ` carry the payload.  This function
/// scans `buffer` for complete events, collects their concatenated `data:`
/// payloads, and returns them along with the remaining (incomplete) buffer
/// tail.
///
/// Returns `(events, remaining_buffer)` where `events` is a list of payload
/// strings (one per complete event) and `remaining_buffer` is the leftover
/// text after the last `\n\n` (to be fed more bytes on the next call).
pub fn extract_sse_events(buffer: &str) -> (Vec<String>, &str) {
    let mut events = Vec::new();
    let mut remaining = buffer;

    while let Some(pos) = remaining.find("\n\n") {
        let event_block = &remaining[..pos];
        remaining = &remaining[pos + 2..];

        let mut data_parts: Vec<&str> = Vec::new();
        for line in event_block.lines() {
            if let Some(d) = line
                .strip_prefix("data: ")
                .or_else(|| line.strip_prefix("data:"))
            {
                data_parts.push(d.trim());
            }
        }

        if !data_parts.is_empty() {
            events.push(data_parts.join("\n"));
        }
    }

    (events, remaining)
}

/// Marker for the end of an SSE stream.
pub const DONE: &str = "[DONE]";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_event() {
        let buf = "data: {\"hello\":\"world\"}\n\n";
        let (events, remaining) = extract_sse_events(buf);
        assert_eq!(events, vec!["{\"hello\":\"world\"}"]);
        assert!(remaining.is_empty());
    }

    #[test]
    fn multiple_events() {
        let buf = "data: first\n\ndata: second\n\ndata: third\n\n";
        let (events, remaining) = extract_sse_events(buf);
        assert_eq!(events.len(), 3);
        assert_eq!(events[0], "first");
        assert_eq!(events[2], "third");
        assert!(remaining.is_empty());
    }

    #[test]
    fn incomplete_event_kept_in_buffer() {
        let buf = "data: complete\n\ndata: incomplete";
        let (events, remaining) = extract_sse_events(buf);
        assert_eq!(events, vec!["complete"]);
        assert_eq!(remaining, "data: incomplete");
    }

    #[test]
    fn done_marker_preserved() {
        let buf = "data: [DONE]\n\n";
        let (events, _) = extract_sse_events(buf);
        assert_eq!(events, vec!["[DONE]"]);
    }

    #[test]
    fn no_data_prefix_ignored() {
        let buf = "event: ping\ndata: payload\n\n";
        let (events, _) = extract_sse_events(buf);
        assert_eq!(events, vec!["payload"]);
    }

    #[test]
    fn empty_buffer() {
        let (events, remaining) = extract_sse_events("");
        assert!(events.is_empty());
        assert!(remaining.is_empty());
    }

    #[test]
    fn bare_data_prefix() {
        let buf = "data:no_space\n\n";
        let (events, _) = extract_sse_events(buf);
        assert_eq!(events, vec!["no_space"]);
    }
}

/// Decode the longest complete UTF-8 prefix; returns (safe, consumed).
///
/// Audit R1-LP-1: a permanently invalid byte sequence (not a truncated
/// multi-byte tail) used to be left at the head of the caller's buffer
/// forever — `consumed` stayed 0, the stream stalled, and the buffer
/// grew on every read. Per the WHATWG decoding the SSE spec mandates,
/// the invalid run is now consumed and surfaced as U+FFFD, so the
/// stream resynchronises instead of wedging.
pub fn decode_complete_utf8_prefix(bytes: &[u8]) -> (String, usize) {
    match std::str::from_utf8(bytes) {
        Ok(s) => (s.to_string(), bytes.len()),
        Err(e) => {
            let valid = e.valid_up_to();
            let mut safe = String::from_utf8_lossy(&bytes[..valid]).into_owned();
            match e.error_len() {
                // Definitively invalid run: consume it, resynchronise.
                Some(bad_len) => {
                    safe.push('\u{FFFD}');
                    (safe, valid + bad_len)
                }
                // Truncated multi-byte tail: wait for more bytes.
                None => (safe, valid),
            }
        }
    }
}

#[cfg(test)]
mod tail_and_utf8_tests {
    use super::*;

    /// Round-35's F-35-1 shape check: both the tail event and DONE must
    /// extract from a single buffer (the old DONE handler dropped the
    /// tail's parsed chunks when they shared a read).
    #[test]
    fn tail_and_done_extract_from_one_buffer() {
        let buf = "data: {\"choices\":[{\"delta\":{\"content\":\"TAIL\"}}]}\n\ndata: [DONE]\n\n";
        let (events, rest) = extract_sse_events(buf);
        assert_eq!(events.len(), 2);
        assert_eq!(events[1], DONE);
        assert!(rest.is_empty());
    }

    /// Round-35's F-35-2: a multi-byte char split across reads must
    /// reassemble without replacement characters.
    #[test]
    fn utf8_split_across_reads_reassembles() {
        let full = "你好".as_bytes();
        let (first, second) = full.split_at(4);
        let mut raw: Vec<u8> = first.to_vec();
        let (s1, c1) = decode_complete_utf8_prefix(&raw);
        raw.drain(..c1);
        raw.extend_from_slice(second);
        let (s2, c2) = decode_complete_utf8_prefix(&raw);
        raw.drain(..c2);
        assert_eq!(format!("{s1}{s2}"), "你好");
    }

    /// Audit R1-LP-1: a permanently invalid byte used to stay at the
    /// buffer head forever (consumed = 0 ⇒ stall + unbounded growth). It
    /// must be consumed as U+FFFD so the stream resynchronises.
    #[test]
    fn invalid_bytes_resynchronise_instead_of_stalling() {
        let mut raw: Vec<u8> = vec![0xFF, b'a', b'b'];
        let (s, consumed) = decode_complete_utf8_prefix(&raw);
        raw.drain(..consumed);
        // Exactly the invalid run (1 byte) is consumed — the valid ASCII
        // tail stays for the next decode. Pre-fix this was 0: stall.
        assert_eq!(consumed, 1, "the invalid byte must be consumed");
        assert!(s.contains('\u{FFFD}'));
        assert_eq!(raw, b"ab");
        // The stream keeps flowing after the resynchronisation.
        raw.extend_from_slice(b"c");
        let (s2, c2) = decode_complete_utf8_prefix(&raw);
        assert_eq!(s2, "abc");
        assert_eq!(c2, 3);
    }

    /// Audit R1-LP-2: CRLF-only and bare-CR framing must still split.
    #[test]
    fn crlf_and_cr_framing_normalize() {
        let mut buf = String::from("data: a\r\n\r\ndata: b\r\rdata: c");
        normalize_sse_buffer(&mut buf);
        let (events, rest) = extract_sse_events(&buf);
        assert_eq!(events, vec!["a", "b"]);
        assert_eq!(rest, "data: c");
    }

    #[test]
    fn cap_helper_flags_only_over() {
        assert!(!over_buffer_cap(MAX_SSE_BUFFER_BYTES));
        assert!(over_buffer_cap(MAX_SSE_BUFFER_BYTES + 1));
    }
}
