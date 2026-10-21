//! Content-addressed archive/restore facility, shared by every celestia
//! project (the 2026-10-21 L2-plugin-platform ruling: plugin uninstall
//! moves configuration into an archive instead of deleting it, and the
//! same facility serves deploy `.prev` snapshots, config migrations and
//! service rollbacks).
//!
//! One archive = a directory
//! `<root>/<kind>/<id>/<stamp>/` holding `manifest.json` (the metadata,
//! integrity anchor) plus `payload.tar` (optionally encrypted).
//!
//! - **Integrity**: the manifest records the SHA-256 of the stored
//!   payload bytes and re-verifies it on every open/restore/verify —
//!   swapping either side fails loudly. (The manifest/payload pairing,
//!   not AEAD, is the tamper evidence; AES-GCM defends
//!   confidentiality.)
//! - **Encryption**: AES-256-GCM; the key lives with the caller
//!   (`_secure` / `LoadCredential`), never inside the archive. A wrong
//!   key or a tampered header fails closed.
//! - **Restore discipline**: path-confinement on every tar entry
//!   (no absolute paths, no `..` walks), dry-run, overwrite and
//!   backup-before-restore options.
//!
//! The v1 API is deliberately synchronous (`std::fs`), matching the
//! registry crate's consumability stance. Compression is a forward
//! field (`Encryption::Plaintext` payloads are plain tar today); GC is
//! manual via [`ArchiveStore::purge`].

pub mod manifest;
pub mod store;

pub use manifest::{ARCHIVE_MANIFEST_SCHEMA, ArchiveManifest, Encryption};
pub use store::{
    ArchiveEntrySummary, ArchiveLocator, ArchiveReceipt, ArchiveRequest, ArchiveStore,
    RestoreOptions, RestoreReport, VerifyReport,
};

use std::path::PathBuf;
use thiserror::Error;

/// Every way the facility can fail. Storage paths are echoed back so an
/// operator can act, but secrets never travel through this type.
#[derive(Debug, Error)]
pub enum ArchiveError {
    /// Underlying filesystem failure at `path`.
    #[error("archive I/O at {path}: {source}")]
    Io {
        /// The path the failed operation touched.
        path: PathBuf,
        /// The OS error.
        #[source]
        source: std::io::Error,
    },
    /// Digest mismatch, malformed payload or a manifest that contradicts
    /// the bytes on disk.
    #[error("archive `{locator}` is corrupt: {reason}")]
    Corrupt {
        /// Human-facing locator (`kind/id/stamp`).
        locator: String,
        /// What specifically failed.
        reason: String,
    },
    /// The payload is encrypted but no key was supplied.
    #[error("archive `{locator}` is encrypted — supply the decryption key")]
    KeyRequired {
        /// Human-facing locator.
        locator: String,
    },
    /// The key did not decrypt the payload (wrong key, or the
    /// archived bytes were tampered with).
    #[error("archive `{locator}` decryption failed (wrong key or tampered manifest)")]
    Decrypt {
        /// Human-facing locator.
        locator: String,
    },
    /// Restoring would clobber existing paths and `overwrite` was not
    /// requested.
    #[error("restore of `{locator}` would overwrite {count} existing path(s), first: {first}")]
    Conflict {
        /// Human-facing locator.
        locator: String,
        /// How many existing paths collide.
        count: usize,
        /// One colliding path (the first in tar order).
        first: PathBuf,
    },
    /// No archive directory matches the locator.
    #[error("no archive found for kind `{kind}` id `{id}`")]
    NotFound {
        /// The kind that was searched.
        kind: String,
        /// The id that was searched.
        id: String,
    },
    /// A request that cannot name a valid archive (empty kind/id, bad
    /// source path, traversal attempt).
    #[error("invalid archive request: {0}")]
    Invalid(String),
}
