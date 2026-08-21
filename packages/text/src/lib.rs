//! LLM text accumulation and structured streaming abstractions.
//!
//! [`LlmText`] is an immutable, reference-counted text buffer built from
//! append-only chunks via [`LlmTextBuilder`]. Zero-copy slicing is supported
//! through [`LlmTextSlice`].
//!
//! [`LlmStream`] models a streaming LLM response as a sequence of
//! [`StreamSegment`]s, each tagged with a [`StreamChunkKind`] — plain text,
//! code-fence blocks, tool calls ([`StreamToolEvent`]), think blocks,
//! or agent handoff markers. The builder merges contiguous segments of the
//! same kind for compact wire representation.
//!
//! [`report_text`] holds the markdown-aware hygiene layer (extractive
//! summaries, LLM meta-text detection, tool-payload classification) shared
//! by the report pipeline consumers.
#![allow(clippy::type_complexity)]

pub mod llm_text;
pub mod report_text;
pub mod stream_segment;

pub use llm_text::{LlmText, LlmTextBuilder, LlmTextSlice};
pub use report_text::{
    ReportJsonShape, classify_report_json, is_markdown_structured, looks_like_llm_meta_text,
    plain_text_summary,
};
pub use stream_segment::{
    LlmStream, LlmStreamBuilder, StreamChunkKind, StreamSegment, StreamToolEvent,
};
