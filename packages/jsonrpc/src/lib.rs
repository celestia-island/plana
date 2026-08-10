//! JSON-RPC 2.0 wire layer.
//!
//! Two transport backends:
//! - **Unix socket** — Unix-domain-socket bridge for local IPC between
//!   separate OS processes (abstract namespace on Linux, file-system socket
//!   elsewhere).
//! - **HTTP / WS** — [`rpc_router::RpcMethodMap`] provides an axum-compatible
//!   method registry that can be mounted at any path, accepting both HTTP
//!   POST and WebSocket upgrade, suitable for web frontends.
//!
//! Architecture:
//! - **`unix_socket`** — low-level socket lifecycle: bind, connect, chmod,
//!   stale-socket cleanup. Defines well-known socket paths and
//!   abstract-namespace helpers for Linux.
//! - **`unix_transport`** — [`JsonRpcTransport`], [`JsonRpcSender`],
//!   [`JsonRpcReceiver`] wrap the raw byte stream with newline-delimited JSON
//!   framing and request/response correlation (using JSON-RPC `id` fields).
//! - **`bridge`** — the [`GatewayMethod`] enum maps wire method strings into
//!   typed variants (`Sync.Ping`, `Base.Heartbeat`, `Tool.CallTool`, …) and
//!   provides serialization/deserialization helpers that work with any
//!   serde-serializable message type.
//! - **`json_keys`** — typed parameter-key enums that replace raw `&str` key
//!   lookups with compile-time-checked variants, reducing debugging surface
//!   when the protocol evolves.
//! - **`pending`** — pending-request registry with `Method` catalog,
//!   one-shot completion handles, and the [`namespace!`] macro used to
//!   declare typed method namespaces (built-in protocol families only — the
//!   `Method` enum is a closed set; third-party method names register
//!   directly into [`rpc_router::RpcMethodMap`]).
//!
//! Design philosophy: the bridge is *intentionally narrow*. Both sides agree
//! on an exhaustive message catalog; clients never open a raw socket — they
//! go through the transport abstractions. This makes it straightforward to
//! add a third process (e.g. a web frontend) that speaks the same well-typed
//! JSON-RPC dialect.
#![allow(clippy::type_complexity)]

pub mod bridge;
pub mod json_keys;
pub mod pending;
pub mod rpc_router;
pub mod session;
pub mod types;
pub mod unix_socket;
pub mod unix_transport;

pub use bridge::{
    core_message_to_method_and_params, deserialize_from_jsonrpc, from_jsonrpc_method,
    serialize_to_jsonrpc, GatewayMethod, UnknownGatewayMethodError,
};
pub use json_keys::{
    AuthParamKey, BridgeKey, ContainerCreateParamKey, ContainerForkParamKey, ContainerVolumeKey,
    ReplParamKey, ResponseKey, ToolCallParamKey, ToolListToolsResultKey,
};
pub use pending::{MessageKind, Method, PendingHandle, PendingRegistry};
pub use rpc_router::{rpc_axum_router, RpcMethodMap};
pub use types::{UnixMethod, *};
pub use unix_socket::{
    bind_interprocess, bind_log_socket, chmod_socket, connect_interprocess, connect_log_socket,
    cosmos_bridge_socket_path, cosmos_socket_dir, cosmos_socket_path, ensure_socket_dir,
    log_socket_dir, log_socket_path, remove_stale_socket, tui_socket_path, InterprocessAccept,
    InterprocessListener, InterprocessStream, ListenerNonblockingMode, COSMOS_BRIDGE_SOCKET,
    COSMOS_SOCKET_DIR, COSMOS_SOCKET_SUFFIX, DEFAULT_SOCKET_DIR, LOG_SOCKET_FILENAME,
};
pub use unix_transport::{
    IncomingMessage, JsonRpcReceiver, JsonRpcSender, JsonRpcServer, JsonRpcTransport, TimeoutPolicy,
};
