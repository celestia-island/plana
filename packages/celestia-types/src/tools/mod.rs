//! Per-agent tool type definitions.
//!
//! Per-agent tool result/request types. These are the structured data
//! contracts for every tool exposed by the multi-agent platform's exec-only
//! microkernel architecture. Both entelecheia (Rust) and shittim-chest (TS)
//! consume these types via ts-rs code generation.
//!
//! Only per-agent tool I/O structs live here. Shared domain vocabulary enums
//! live at [`crate::enums`] and external server config at [`crate::external_mcp`].

pub mod aporia;
pub mod eleos;
pub mod epieikeia;
pub mod haplotes;
pub mod hubris;
pub mod kalos;
pub mod neikos;
pub mod orexis;
pub mod philia;
pub mod polemos;
pub mod skemma;
pub mod skopeo;
pub mod web_automation;
