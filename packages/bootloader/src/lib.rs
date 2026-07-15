//! Bootloader layer: bring up the entelecheia container stack and register
//! evernight as a native OS service on Linux / Windows / macOS.
//!
//! This crate is the composition root for the "host bootstrap" responsibility.
//! It owns three concerns, kept deliberately separate so each can be exercised
//! in isolation:
//!
//! - **`identity`** — generates and persists the `celestia-XXX` instance id
//!   (000-999). The id drives container-name prefixes, port offsets, and the
//!   evernight `node_id`. Persisted to the caller-supplied config path so the
//!   same id survives restarts.
//! - **`stack`** — pulls up the three infrastructure containers
//!   (`{prefix}postgres`, `{prefix}scepter`, `{prefix}registry`), the shared
//!   docker network, and the ext4-backed socket directories. This logic was
//!   lifted out of `entelecheia-cli`'s `service.rs` so both the CLI and the
//!   evernight bootloader share a single implementation.
//! - **`platform`** — installs / uninstalls evernight as a native OS service:
//!   systemd user unit on Linux, Windows SCM service via `sc.exe`, and a
//!   launchd `LaunchDaemon` plist on macOS.
//!
//! The crate depends only on the arona shared layer (`_container`,
//! `_infra_services`, `_config`, `_infra_utils`) — never on entelecheia or
//! evernight — to avoid any circular dependency.

pub mod identity;
pub mod platform;
pub mod stack;

pub use identity::{InstanceIdentity, container_prefix_for, generate_instance_id, node_id_for};
pub use platform::{ServiceInstaller, ServiceSpec, ServiceStatus, default_service_installer};
pub use stack::{
    StackConfig, StackHandle, bring_up_stack, ensure_network, ensure_socket_dirs, teardown_stack,
};
