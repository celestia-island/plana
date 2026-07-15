//! OCI-native container runtime via Youki, implementing the [`ContainerOps`] trait.
//!
//! This crate is the rootless, daemonless alternative to the Docker backend. It wraps
//! [libcontainer](https://github.com/containers/youki) (the library underlying the Youki
//! OCI runtime) to create, start, and manage containers directly — no dockerd dependency.
//!
//! Key components:
//! - [`YoukiManager`] — the primary entry point; implements `ContainerOps` using
//!   `libcontainer::ContainerBuilder` for lifecycle management.
//! - [`RootfsManager`] — caches and snapshots container root filesystems so that
//!   repeated agent runs don't re-extract images.
//! - **`spec`** — builds OCI runtime config.json documents from security profiles
//!   (seccomp, landlock, namespace isolation) defined by the `container` crate.
//! - **`state`** — persists container records and status to disk so that the runtime
//!   can survive daemon restarts.
//!
//! The design philosophy is *self-contained isolation*: by using Youki in rootless mode
//! with user namespaces, the platform avoids giving the orchestrator escalated privileges.
//! Combined with the shared security profiles from `container`, this provides a consistent
//! sandbox regardless of runtime choice.
#![allow(clippy::type_complexity)]

pub mod capability;
pub mod manager;
pub mod rootfs;
pub mod spec;
pub mod state;

pub use capability::{RootfsCapability, detect_inside_container, detect_rootfs_capability};
pub use manager::YoukiManager;
pub use rootfs::RootfsManager;
