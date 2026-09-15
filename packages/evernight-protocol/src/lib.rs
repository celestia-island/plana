//! Re-export shim for the evernight Tier 3 `protocol.*` wire DTOs.
//!
//! The DTOs and their TypeScript bindings moved into
//! [`plana_celestia_types::evernight`] (and ship to npm as part of
//! `@celestia-island/plana-types`, the family's single protocol package —
//! the standalone `@celestia-island/plana-evernight-protocol` package was
//! retired before its first successful publish). This crate exists only so
//! the evernight gateway's git-dep import path
//! (`use plana_evernight_protocol::{TransportInfoDto, …}`) keeps resolving.

pub use plana_celestia_types::evernight::*;
