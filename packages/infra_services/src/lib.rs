//! Service-wiring layer: assembles infrastructure components into a running system.
//!
//! This crate bridges the gap between *what* the platform needs (defined by `core` /
//! `container` / `storage`) and *how* those needs are satisfied at startup. It is the
//! composition root for the backend process — it owns decisions about runtime selection,
//! credential storage format, log persistence, and WebSocket transport.
//!
//! Key responsibilities:
//! - **`container_factory`** — the single decision point that picks Docker or Youki based
//!   on user config; returns a `Box<dyn ContainerOps>` so callers are runtime-agnostic.
//! - **`server_manager`** — manages the lifecycle of the long-running "scepter" server
//!   container (health checks, startup, teardown).
//! - **`connection_token`** — generates and verifies shared-secret tokens that gate
//!   access to the Unix-domain-socket JSON-RPC channel.
//! - **`persistence`** / **`pg_log_*`** — schema initialisation and tracing-layer hooks
//!   that stream structured logs into PostgreSQL.
//! - **`file_credential_storage`** — on-disk, permissions-gated API key store.
//!
//! The design follows the *Factory pattern at the crate boundary*: `infra_services` knows
//! about all concrete implementations, but exposes only facades and factory functions.
//! Downstream code (the TUI, the agent runtime) never binds to a specific backend.
#![allow(clippy::type_complexity)]

pub mod connection_token;
pub mod container_factory;
pub mod file_credential_storage;
pub mod persistence;
pub mod pg_log_layer;
pub mod pg_log_writer;
pub mod server_manager;
pub mod types;
pub mod ws_transport;

pub use connection_token::{
    connection_token_path, generate_token, read_token, remove_token_file, socket_token_path,
    write_token_to_file,
};
pub use container_factory::{
    cosmos_runtime_type, create_container_backend, create_container_backend_from_config,
    outer_runtime_type,
};
pub use file_credential_storage::{CredentialFile, CredentialRecord, FileCredentialStorage};
pub use persistence::{OnlineAgentInfo, Persistence};
pub use pg_log_layer::PgLogLayer;
pub use pg_log_writer::{LogEntry, PgLogWriter, cleanup_old_logs};
pub use server_manager::{ServerManager, ServerStatus, inject_docker_client, set_cluster_prefix};
pub use types::{AgentMetadata, LogContext};
pub use ws_transport::{
    Message as WsTransportMessage, WsReceiver, WsSender, WsTransport, WsTransportConfig,
};

pub use arona_container_runtime::{RootfsCapability, detect_rootfs_capability};
