//! Re-export shim: the JSON-RPC 2.0 wire layer now lives in the `plana`
//! protocol foundation (`plana::jsonrpc`). This crate exists only so
//! git-pin consumers of `plana-jsonrpc` keep compiling; new code should
//! depend on `plana` directly.

pub use plana::jsonrpc::*;
