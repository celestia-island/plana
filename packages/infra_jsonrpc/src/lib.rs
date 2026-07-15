//! Unix-domain-socket JSON-RPC bridge connecting the TUI to the orchestration backend.
//!
//! This crate is the *wire layer* of the entelecheia IPC architecture. The TUI process
//! (a terminal application) and the backend process (the agent orchestrator) run in
//! separate OS processes and communicate exclusively through Unix abstract-namespace
//! sockets using JSON-RPC 2.0 semantics.
//!
//! Architecture:
//! - **`unix_socket`** — low-level socket lifecycle: bind, connect, chmod, stale-socket
//!   cleanup. Defines well-known socket paths (cosmos bridge, TUI bridge, log socket)
//!   and abstract-namespace helpers for Linux.
//! - **`unix_transport`** — [`JsonRpcTransport`], [`JsonRpcSender`], [`JsonRpcReceiver`]
//!   wrap the raw byte stream with newline-delimited JSON framing and request/response
//!   correlation (using JSON-RPC `id` fields).
//! - **`bridge`** — the [`GatewayMethod`] enum maps every cross-process message
//!   (agent reports, snapshots, user commands, auth tokens) into typed variants
//!   (`Tui::AgentReport`, `Tui::ContainerSnapshot`, etc.) and provides
//!   serialization/deserialization helpers.
//! - **`json_keys`** — typed parameter-key enums that replace raw `&str` key lookups
//!   with compile-time-checked variants, reducing debugging surface when the protocol
//!   evolves.
//!
//! Design philosophy: the bridge is *intentionally narrow*. Both sides agree on an
//! exhaustive message catalog; the TUI never opens a raw socket — it goes through
//! the transport abstractions. This makes it straightforward to add a third process
//! (e.g. a web frontend) that speaks the same well-typed JSON-RPC dialect.
#![allow(clippy::type_complexity)]

pub mod bridge;
pub mod json_keys;
pub mod pending;
pub mod types;
pub mod unix_socket;
pub mod unix_transport;

pub use bridge::{
    GatewayMethod, UnknownGatewayMethodError, core_message_to_method_and_params,
    deserialize_from_jsonrpc, from_jsonrpc_method, serialize_to_jsonrpc,
};
pub use json_keys::{
    AuthParamKey, BridgeKey, ContainerCreateParamKey, ContainerForkParamKey, ContainerVolumeKey,
    McpCallParamKey, McpListToolsResultKey, ReplParamKey, ResponseKey,
};
pub use pending::{MessageKind, PendingHandle, PendingRegistry, methods};
pub use types::{UnixMethod, *};
pub use unix_socket::{
    COSMOS_BRIDGE_SOCKET, COSMOS_SOCKET_DIR, COSMOS_SOCKET_SUFFIX, DEFAULT_SOCKET_DIR,
    InterprocessAccept, InterprocessListener, InterprocessStream, LOG_SOCKET_FILENAME,
    ListenerNonblockingMode, bind_interprocess, bind_log_socket, chmod_socket,
    connect_interprocess, connect_log_socket, cosmos_bridge_socket_path, cosmos_socket_dir,
    cosmos_socket_path, ensure_socket_dir, log_socket_dir, log_socket_path, remove_stale_socket,
    tui_socket_path,
};
pub use unix_transport::{
    IncomingMessage, JsonRpcReceiver, JsonRpcSender, JsonRpcServer, JsonRpcTransport, TimeoutPolicy,
};
