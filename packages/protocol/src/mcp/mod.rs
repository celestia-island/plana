//! MCP tool type definitions.
//!
//! Per-agent MCP tool result/request types. These are the structured data
//! contracts for every tool exposed by the multi-agent platform's exec-only
//! microkernel architecture. Both entelecheia (Rust) and shittim-chest (TS)
//! consume these types via ts-rs code generation.
//!
//! Only per-agent tool I/O structs live here. Shared domain vocabulary enums
//! live at [`crate::enums`] and external server config at [`crate::external_mcp`].
//! The re-exports below keep the historical `arona::mcp::enums` /
//! `arona::mcp::external` paths working for downstream consumers.

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

// Backward-compat re-exports. Canonical homes were hoisted out of `mcp/`:
//   - vocabulary enums  -> `crate::enums`
//   - external servers  -> `crate::external_mcp`
// These keep `arona::mcp::enums::*` / `arona::mcp::external::*` resolvable so
// existing consumers compile unchanged. New code should import from the root.
pub use crate::enums;
pub use crate::external_mcp as external;
