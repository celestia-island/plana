//! Shared SSE (Server-Sent Events) parsing utilities.
//!
//! All four LLM providers (OpenAI Compatible, Gemini, Anthropic, OpenAI
//! Responses) replaced `reqwest-eventsource` 0.6 (which silently drops POST
//! bodies) with manual SSE parsing.  This module centralises the common
//! buffer-and-split logic so each provider only implements the JSON
//! deserialisation specific to its API format.

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
