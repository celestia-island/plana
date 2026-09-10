//! Re-export shim: the generic protocol core now lives in the `plana`
//! protocol foundation (`plana::protocol_core`). This crate exists only so
//! git-pin consumers of `plana-protocol-core` keep compiling; new code
//! should depend on `plana` directly.

pub use plana::protocol_core::*;
