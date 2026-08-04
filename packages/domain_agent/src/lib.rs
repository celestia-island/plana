//! Agent type system — badge, descriptor, kind, and phantom-type markers.
//!
//! This crate provides compile-time agent identity used throughout the
//! Entelecheia platform for routing, tool scoping, and type-level dispatch.
//!
//! Key abstractions:
//! - [`AgentKind`] — exhaustive enum of all 17 agents (12 Layer 1 + 5 Layer 2),
//!   the single source of truth for agent names, doc paths, and UI display.
//! - [`AgentMarker`] — sealed trait implemented by zero-sized marker structs
//!   (e.g. [`KaLosMarker`], [`NeiKosMarker`]) that carry const metadata
//!   (KIND, FOLDER_NAME, FRIENDLY_NAME). Used as phantom-type parameters in
//!   [`ToolRegistry`] and other generic contexts.
//! - [`AgentDescriptor`] / [`AgentMetadataRegistry`] — runtime metadata store
//!   (friendly name, layer, containerized flag) with lazy initialization.
//! - [`AgentBadge`] — re-exported from `_core` for agent identification.
//!
//! Design philosophy: agents are known at compile time; the marker pattern
//! provides zero-cost type safety (no allocations, no virtual dispatch) while
//! the descriptor registry handles dynamic metadata queries from the TUI.
#![allow(clippy::type_complexity)]

pub mod badge;
pub mod descriptor;
pub mod kind;
pub mod markers;

pub use badge::AgentBadge;
pub use kind::{AgentKind, UnknownAgentError};
pub use markers::{
    AgentMarker, ApoRiaMarker, ClassicSoftwareEngineeringMarker, EleOsMarker, EpieiKeiaMarker,
    HapLotesMarker, HubRisMarker, IndustrialIoTMarker, KaLosMarker, NeiKosMarker, OreXisMarker,
    PhantomAgent, PhiLiaMarker, PoleMosMarker, RemoteOperationsMarker, SkeMmaMarker, SkoPeoMarker,
    WebAutomationMarker,
};
