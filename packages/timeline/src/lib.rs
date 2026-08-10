//! Rich CLI timeline visualization for agent dialogues and skill execution.
//!
//! This crate renders structured conversation/skill-execution histories as
//! formatted terminal output, giving operators a compact, scrollable view of
//! agent activity.
//!
//! Key abstractions:
//! - [`TimelineGroup`] et al. — data model for human messages, skill invocations,
//!   and ask-human prompts, with nested tool-call blocks.
//! - [`TimelineRenderer`] / [`TimelineOutput`] — trait-based rendering pipeline
//!   that decouples the data model from the output format.
//! - [`CliTimelineRenderer`] — concrete ASCII renderer producing buffered line
//!   output suitable for raw terminal display.
//! - [`chars`] module — glyph constants (ASCII and Unicode box-drawing) shared
//!   across renderers.
//!
//! Design philosophy: the timeline is pure presentation — it consumes
//! immutable event data and produces formatted text without side effects,
//! enabling easy testing and alternative frontends.
#![allow(clippy::type_complexity)]

pub mod chars;
pub mod cli_renderer;
pub mod renderer;
pub mod types;

pub use chars::{
    ARROW_DOWN, ARROW_SWAP, ARROW_UP, BD_DL, BD_DR, BD_H, BD_RND_BL, BD_RND_BR, BD_RND_TL,
    BD_RND_TR, BD_T_LEFT, BD_UL, BD_UR, BD_V, CHECK, CROSS, DOT_ALT, DOT_EMPTY, DOT_FILLED, HLINE,
    TL_BODY, TL_CLOSE, TL_HEADER, TL_SEP, TL_SEP_CHAR, TL_TOOL_CLOSE, TL_TOOL_INNER, TL_TOOL_OPEN,
};
pub use cli_renderer::{
    CliTimelineBuffer, CliTimelineRenderer, fmt_cont_line, fmt_tool_line, fmt_truncate,
};
pub use renderer::{TimelineOutput, TimelineRenderer};
pub use types::{
    GroupState, GroupStats, SkillBlockStatus, TimelineAskHumanGroup, TimelineContentBlock,
    TimelineContentKind, TimelineGroup, TimelineGroupData, TimelineHumanGroup,
    TimelineSegmentBlock, TokenSource, ToolBlockData, ToolBlockState, ToolCloseLabel,
    format_number,
};
