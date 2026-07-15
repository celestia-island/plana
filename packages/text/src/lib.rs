//! LLM text accumulation and structured streaming abstractions.
//!
//! [`LlmText`] is an immutable, reference-counted text buffer built from
//! append-only chunks via [`LlmTextBuilder`]. Zero-copy slicing is supported
//! through [`LlmTextSlice`].
//!
//! [`LlmStream`] models a streaming LLM response as a sequence of
//! [`StreamSegment`]s, each tagged with a [`StreamChunkKind`] — plain text,
//! code-fence blocks, MCP tool calls ([`StreamMcpEvent`]), think blocks,
//! or agent handoff markers. The builder merges contiguous segments of the
//! same kind for compact wire representation.
#![allow(clippy::type_complexity)]

pub mod llm_text;
pub mod stream_segment;

pub use llm_text::{LlmText, LlmTextBuilder, LlmTextSlice};
pub use stream_segment::{
    LlmStream, LlmStreamBuilder, StreamChunkKind, StreamMcpEvent, StreamSegment,
};
