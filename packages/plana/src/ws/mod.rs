//! WebSocket TUI message domains — the variant params of the platform's
//! `SyncMessage` enum, mirroring entelecheia's `tui_types/message/types`.
//!
//! Each submodule's types are re-exported at the crate root
//! (`arona::TypeName`); the TypeScript bindings land under `ws/*.ts`, matching
//! the `#[ts(export_to = "ws/…")]` attributes on every type defined here.

pub mod agent;
pub mod services;
pub mod ui;
