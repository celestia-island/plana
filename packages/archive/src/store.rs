//! The archive store: create, list, inspect, verify, restore and purge
//! content-addressed archives under one root directory.
//!
//! Storage layout: `<root>/<kind>/<id>/<stamp>/` where `stamp` is a UTC
//! timestamp plus a random suffix (same-second archives never collide).
//! The locator's "latest" resolution is the lexicographically greatest
//! stamp — which is why the stamp is fixed-width.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use chrono::Utc;
use rand::RngCore;
use sha2::{Digest, Sha256};

use super::ArchiveError;
use super::manifest::{ARCHIVE_MANIFEST_SCHEMA, ArchiveManifest, Encryption};

/// The payload file name inside one archive directory.
const PAYLOAD_FILE: &str = "payload.tar";
/// The manifest file name inside one archive directory.
const MANIFEST_FILE: &str = "manifest.json";
/// The nonce length for AES-256-GCM.
const NONCE_LEN: usize = 12;
/// The AEAD magic prefix distinguishing an encrypted payload file from
/// a plain one at a glance.
const ENC_MAGIC: &[u8; 4] = b"PEA1";

/// Which archive set a locator resolves within.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveLocator {
    /// The archive namespace (e.g. `plugins`).
    pub kind: String,
    /// The archived subject's stable id.
    pub id: String,
    /// The exact stamp; `None` resolves to the latest.
    pub stamp: Option<String>,
}

impl ArchiveLocator {
    /// A locator resolving to the latest archive of `kind`/`id`.
    pub fn latest(kind: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            id: id.into(),
            stamp: None,
        }
    }

    /// A locator pinning one exact stamp.
    pub fn at(kind: impl Into<String>, id: impl Into<String>, stamp: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            id: id.into(),
            stamp: Some(stamp.into()),
        }
    }

    fn dir(&self, root: &Path) -> Result<PathBuf, ArchiveError> {
        // A locator is rebuildable from external input (RPC parameters,
        // config); every read/purge path funnels through here, so the
        // kind/id/stamp charset is re-checked before anything is joined
        // under the root.
        validate_kind(&self.kind)?;
        validate_id(&self.id)?;
        if let Some(stamp) = &self.stamp {
            validate_stamp(stamp)?;
        }
        let base = root.join(&self.kind).join(&self.id);
        match &self.stamp {
            Some(stamp) => Ok(base.join(stamp)),
            None => {
                if !base.exists() {
                    return Err(ArchiveError::NotFound {
                        kind: self.kind.clone(),
                        id: self.id.clone(),
                    });
                }
                let mut best: Option<PathBuf> = None;
                for entry in fs::read_dir(&base).map_err(|e| ArchiveError::Io {
                    path: base.clone(),
                    source: e,
                })? {
                    let entry = entry.map_err(|e| ArchiveError::Io {
                        path: base.clone(),
                        source: e,
                    })?;
                    let path = entry.path();
                    if path.is_dir() {
                        match &best {
                            Some(current) if current.file_name() >= path.file_name() => {}
                            _ => best = Some(path),
                        }
                    }
                }
                best.ok_or_else(|| ArchiveError::NotFound {
                    kind: self.kind.clone(),
                    id: self.id.clone(),
                })
            }
        }
    }
}

/// One archive-to-be: what to store and how.
#[derive(Debug, Clone)]
pub struct ArchiveRequest {
    /// The archive namespace (e.g. `plugins`, `configs`, `deploy`).
    pub kind: String,
    /// The archived subject's stable id (e.g. the plugin id).
    pub id: String,
    /// Files or directories to capture, in tar order. Directories are
    /// walked recursively; entry paths are stored relative to each
    /// source's parent so a restore reproduces the source's own name.
    pub sources: Vec<PathBuf>,
    /// Who took the archive (a host/role label — never a credential).
    pub source: String,
    /// Secret-free metadata carried beside the manifest.
    pub meta: BTreeMap<String, String>,
    /// How the payload is protected at rest.
    pub encryption: Encryption,
}

/// What one successful [`ArchiveStore::archive`] produced.
#[derive(Debug, Clone)]
pub struct ArchiveReceipt {
    /// The exact locator (stamp included) of the new archive.
    pub locator: ArchiveLocator,
    /// The written manifest.
    pub manifest: ArchiveManifest,
    /// Payload bytes as stored on disk.
    pub payload_bytes: u64,
}

/// A one-line summary of a stored archive (what [`ArchiveStore::list`]
/// answers with).
#[derive(Debug, Clone)]
pub struct ArchiveEntrySummary {
    /// The exact locator of the stored archive.
    pub locator: ArchiveLocator,
    /// The stored manifest (metadata + digests).
    pub manifest: ArchiveManifest,
}

/// Restore behaviour knobs.
#[derive(Debug, Clone, Default)]
pub struct RestoreOptions {
    /// Report what would happen without touching the target.
    pub dry_run: bool,
    /// Allow extraction into existing paths (tar semantics: overwrite
    /// per file). Without it, any existing target path is a conflict.
    pub overwrite: bool,
    /// Before extracting, move an existing target directory to
    /// `<target>.pre-restore-<stamp>` (the undo path for a bad
    /// restore).
    pub backup_existing: bool,
}

/// What one restore (real or dry-run) did or would do.
#[derive(Debug, Clone)]
pub struct RestoreReport {
    /// Paths written (or that would be written).
    pub restored: Vec<PathBuf>,
    /// The backup path when [`RestoreOptions::backup_existing`] moved
    /// an existing target aside.
    pub backup_path: Option<PathBuf>,
    /// Whether this run actually wrote (false for a dry run).
    pub applied: bool,
}

/// What [`ArchiveStore::verify`] checked.
#[derive(Debug, Clone)]
pub struct VerifyReport {
    /// Whether the manifest digest matches the stored payload bytes.
    pub payload_digest_ok: bool,
    /// Whether the payload decrypted/decoded as tar (when a key is
    /// available for encrypted payloads).
    pub payload_decodes: Option<bool>,
    /// The manifest as read.
    pub manifest: ArchiveManifest,
}

/// The facility handle. `open` creates the root (0700) if missing.
#[derive(Debug, Clone)]
pub struct ArchiveStore {
    root: PathBuf,
}

impl ArchiveStore {
    /// Open (creating if needed) the store rooted at `root`.
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, ArchiveError> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(|e| ArchiveError::Io {
            path: root.clone(),
            source: e,
        })?;
        set_private_dir(&root)?;
        Ok(Self { root })
    }

    /// The store root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Capture `request.sources` into a new archive under
    /// `<root>/<kind>/<id>/<stamp>/`, returning the receipt.
    pub fn archive(
        &self,
        request: &ArchiveRequest,
        key: Option<&[u8; 32]>,
    ) -> Result<ArchiveReceipt, ArchiveError> {
        validate_kind(&request.kind)?;
        validate_id(&request.id)?;
        if request.sources.is_empty() {
            return Err(ArchiveError::Invalid("no sources to archive".to_string()));
        }
        if matches!(request.encryption, Encryption::Aes256Gcm) && key.is_none() {
            return Err(ArchiveError::Invalid(
                "encryption aes-256-gcm requires a 32-byte key".to_string(),
            ));
        }
        for source in &request.sources {
            if !source.exists() {
                return Err(ArchiveError::Invalid(format!(
                    "source `{}` does not exist",
                    source.display()
                )));
            }
        }

        let base = self.root.join(&request.kind).join(&request.id);
        fs::create_dir_all(&base).map_err(|e| ArchiveError::Io {
            path: base.clone(),
            source: e,
        })?;
        // Atomically claim a unique stamp directory: `create_dir` fails
        // with AlreadyExists when a concurrent archiver of the same id
        // claimed the same name first, and the loop re-reads the winner
        // and steps past it. (`create_dir_all` would let two
        // same-millisecond writers interleave inside one directory and
        // corrupt each other's payloads.)
        let (stamp, dir) = loop {
            let stamp = fresh_stamp(max_stamp(&base).as_deref());
            let dir = base.join(&stamp);
            match fs::create_dir(&dir) {
                Ok(()) => break (stamp, dir),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => {
                    return Err(ArchiveError::Io {
                        path: dir,
                        source: e,
                    });
                }
            }
        };
        set_private_dir(&dir)?;

        let captured = self.capture(request, key, &dir, &stamp);
        if captured.is_err() {
            // The stamp directory was exclusively claimed above, so
            // everything inside is ours: a failed capture must not leave
            // a half-written directory behind — it would resolve as
            // `latest` and poison both inspect and `list` for the id.
            let _ = fs::remove_dir_all(&dir);
        }
        captured
    }

    /// The post-claim half of [`ArchiveStore::archive`]: write the tar
    /// payload, seal it, anchor it with the manifest.
    fn capture(
        &self,
        request: &ArchiveRequest,
        key: Option<&[u8; 32]>,
        dir: &Path,
        stamp: &str,
    ) -> Result<ArchiveReceipt, ArchiveError> {
        let payload_path = dir.join(PAYLOAD_FILE);
        let entry_count = write_tar(&request.sources, &payload_path)?;
        let payload_bytes = seal_payload(&payload_path, request.encryption, key)?;

        let digest = file_sha256(&payload_path)?;
        let manifest = ArchiveManifest {
            schema: ARCHIVE_MANIFEST_SCHEMA,
            kind: request.kind.clone(),
            id: request.id.clone(),
            created_at: Utc::now(),
            source: request.source.clone(),
            payload_sha256: hex::encode(digest),
            payload_bytes,
            encryption: request.encryption,
            entry_count,
            meta: request.meta.clone(),
        };
        let manifest_path = dir.join(MANIFEST_FILE);
        let manifest_json = manifest
            .canonical_json()
            .map_err(|e| ArchiveError::Corrupt {
                locator: format!("{}/{}", request.kind, request.id),
                reason: e.to_string(),
            })?;
        write_private_file(&manifest_path, manifest_json.as_bytes())?;

        let locator = ArchiveLocator {
            kind: request.kind.clone(),
            id: request.id.clone(),
            stamp: Some(stamp.to_string()),
        };
        tracing::info!(
            archive = %manifest.locator(stamp),
            bytes = payload_bytes,
            entries = entry_count,
            "archived"
        );
        Ok(ArchiveReceipt {
            locator,
            manifest,
            payload_bytes,
        })
    }

    /// Every stored archive under the root, ordered by `kind`, `id`,
    /// then stamp (lexicographic = chronological within one id).
    pub fn list(&self) -> Result<Vec<ArchiveEntrySummary>, ArchiveError> {
        let mut out = Vec::new();
        let kinds = match fs::read_dir(&self.root) {
            Ok(entries) => entries,
            Err(e) => {
                return Err(ArchiveError::Io {
                    path: self.root.clone(),
                    source: e,
                });
            }
        };
        let mut kind_names: Vec<PathBuf> = kinds
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        kind_names.sort();
        for kind_dir in kind_names {
            let mut id_dirs: Vec<PathBuf> = match fs::read_dir(&kind_dir) {
                Ok(entries) => entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.is_dir())
                    .collect(),
                Err(e) => {
                    return Err(ArchiveError::Io {
                        path: kind_dir.clone(),
                        source: e,
                    });
                }
            };
            id_dirs.sort();
            for id_dir in id_dirs {
                let mut stamps: Vec<PathBuf> = match fs::read_dir(&id_dir) {
                    Ok(entries) => entries
                        .filter_map(|e| e.ok())
                        .map(|e| e.path())
                        .filter(|p| p.is_dir())
                        .collect(),
                    Err(e) => {
                        return Err(ArchiveError::Io {
                            path: id_dir.clone(),
                            source: e,
                        });
                    }
                };
                stamps.sort();
                for stamp_dir in stamps {
                    let manifest = read_manifest(&stamp_dir)?;
                    let stamp = stamp_dir
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or_default()
                        .to_string();
                    out.push(ArchiveEntrySummary {
                        locator: ArchiveLocator {
                            kind: manifest.kind.clone(),
                            id: manifest.id.clone(),
                            stamp: Some(stamp),
                        },
                        manifest,
                    });
                }
            }
        }
        Ok(out)
    }

    /// Read one archive's manifest (latest when the locator carries no
    /// stamp).
    pub fn inspect(&self, locator: &ArchiveLocator) -> Result<ArchiveManifest, ArchiveError> {
        read_manifest(&locator.dir(&self.root)?)
    }

    /// Re-verify one archive's integrity: payload digest against the
    /// manifest, and (when a key is available for an encrypted payload)
    /// a full decrypt + tar-decode pass.
    pub fn verify(
        &self,
        locator: &ArchiveLocator,
        key: Option<&[u8; 32]>,
    ) -> Result<VerifyReport, ArchiveError> {
        let dir = locator.dir(&self.root)?;
        let manifest = read_manifest(&dir)?;
        let payload_path = dir.join(PAYLOAD_FILE);
        let actual = hex::encode(file_sha256(&payload_path)?);
        let digest_ok = actual == manifest.payload_sha256;
        let decodes = match manifest.encryption {
            Encryption::Aes256Gcm => match key {
                Some(key) => {
                    let plain = open_payload(&payload_path, &manifest, Some(key))?;
                    Some(tar_decodes(&plain))
                }
                None => {
                    return Err(ArchiveError::KeyRequired {
                        locator: manifest.locator(locator.stamp.as_deref().unwrap_or_default()),
                    });
                }
            },
            Encryption::None => {
                let bytes = fs::read(&payload_path).map_err(|e| ArchiveError::Io {
                    path: payload_path.clone(),
                    source: e,
                })?;
                Some(tar_decodes(&bytes))
            }
        };
        Ok(VerifyReport {
            payload_digest_ok: digest_ok,
            payload_decodes: decodes,
            manifest,
        })
    }

    /// Restore one archive into `target`.
    pub fn restore(
        &self,
        locator: &ArchiveLocator,
        target: impl AsRef<Path>,
        options: &RestoreOptions,
        key: Option<&[u8; 32]>,
    ) -> Result<RestoreReport, ArchiveError> {
        let dir = locator.dir(&self.root)?;
        let manifest = read_manifest(&dir)?;
        let locator_label = manifest.locator(locator.stamp.as_deref().unwrap_or_default());
        let payload_path = dir.join(PAYLOAD_FILE);
        let plain = open_payload(&payload_path, &manifest, key)?;

        // Pre-flight: walk the tar, collect the entry paths, and check
        // confinement + conflicts. Nothing is written on a dry run, and
        // a conflict without `overwrite` aborts before any write.
        let target = target.as_ref().to_path_buf();
        let mut planned: Vec<PathBuf> = Vec::new();
        let mut conflicts: Vec<PathBuf> = Vec::new();
        {
            let mut cursor = tar::Archive::new(&plain[..]);
            for entry in cursor.entries().map_err(|e| ArchiveError::Corrupt {
                locator: locator_label.clone(),
                reason: e.to_string(),
            })? {
                let entry = entry.map_err(|e| ArchiveError::Corrupt {
                    locator: locator_label.clone(),
                    reason: e.to_string(),
                })?;
                let header_path = entry.path().map_err(|e| ArchiveError::Corrupt {
                    locator: locator_label.clone(),
                    reason: e.to_string(),
                })?;
                // v1 archives regular files; a link entry inside a
                // payload means tampering or a hostile payload — refuse
                // it fail-closed BEFORE anything is written, so a
                // symlink can never sit in the target for a later entry
                // to write through.
                let entry_type = entry.header().entry_type();
                if entry_type.is_symlink() || entry_type.is_hard_link() {
                    return Err(ArchiveError::Corrupt {
                        locator: locator_label.clone(),
                        reason: format!(
                            "tar entry `{}` is a symlink/hard link — payloads carry regular files only",
                            header_path.display()
                        ),
                    });
                }
                let relative = confined_relative(header_path.as_ref()).ok_or_else(|| {
                    ArchiveError::Corrupt {
                        locator: locator_label.clone(),
                        reason: format!(
                            "tar entry `{}` escapes the restore target",
                            header_path.display()
                        ),
                    }
                })?;
                let destination = target.join(&relative);
                if destination.exists() {
                    conflicts.push(destination.clone());
                }
                planned.push(destination);
            }
        }

        if options.dry_run {
            if !options.overwrite && !conflicts.is_empty() {
                return Err(ArchiveError::Conflict {
                    locator: locator_label,
                    count: conflicts.len(),
                    first: conflicts[0].clone(),
                });
            }
            return Ok(RestoreReport {
                restored: planned,
                backup_path: None,
                applied: false,
            });
        }

        if !options.overwrite && !conflicts.is_empty() {
            return Err(ArchiveError::Conflict {
                locator: locator_label.clone(),
                count: conflicts.len(),
                first: conflicts[0].clone(),
            });
        }

        let mut backup_path = None;
        if target.exists() && options.backup_existing {
            let stamp = fresh_stamp(None);
            let backup = target.with_file_name(format!(
                "{}.pre-restore-{stamp}",
                target
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("target")
            ));
            fs::rename(&target, &backup).map_err(|e| ArchiveError::Io {
                path: target.clone(),
                source: e,
            })?;
            backup_path = Some(backup);
        }
        fs::create_dir_all(&target).map_err(|e| ArchiveError::Io {
            path: target.clone(),
            source: e,
        })?;
        set_private_dir(&target)?;

        let mut restored = Vec::new();
        let mut cursor = tar::Archive::new(&plain[..]);
        for entry in cursor.entries().map_err(|e| ArchiveError::Corrupt {
            locator: locator_label.clone(),
            reason: e.to_string(),
        })? {
            let mut entry = entry.map_err(|e| ArchiveError::Corrupt {
                locator: locator_label.clone(),
                reason: e.to_string(),
            })?;
            let header_path = entry.path().map_err(|e| ArchiveError::Corrupt {
                locator: locator_label.clone(),
                reason: e.to_string(),
            })?;
            let relative =
                confined_relative(header_path.as_ref()).ok_or_else(|| ArchiveError::Corrupt {
                    locator: locator_label.clone(),
                    reason: format!(
                        "tar entry `{}` escapes the restore target",
                        header_path.display()
                    ),
                })?;
            // `unpack_in` joins the entry's own stored path under the
            // target. Every class it could refuse (`..` walks, links,
            // canonical escapes) was already refused above, so the
            // bool is unwritten-to-disk honesty rather than a live
            // guard — the pre-flight is the enforcement point.
            entry.unpack_in(&target).map_err(|e| ArchiveError::Io {
                path: target.clone(),
                source: e,
            })?;
            restored.push(target.join(&relative));
        }
        tracing::info!(
            archive = %locator_label,
            restored = restored.len(),
            "archive restored"
        );
        Ok(RestoreReport {
            restored,
            backup_path,
            applied: true,
        })
    }

    /// Delete one archive directory: the exact stamp, or the LATEST
    /// single stamp when the locator carries none (repeat to drain the
    /// set). There is no undo — callers are expected to have restored
    /// or re-archived.
    pub fn purge(&self, locator: &ArchiveLocator) -> Result<(), ArchiveError> {
        let dir = locator.dir(&self.root)?;
        fs::remove_dir_all(&dir).map_err(|e| ArchiveError::Io {
            path: dir,
            source: e,
        })
    }
}

// ── internals ────────────────────────────────────────────────────────────

fn validate_kind(kind: &str) -> Result<(), ArchiveError> {
    let ok = !kind.is_empty()
        && kind.len() <= 64
        && kind
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if ok {
        Ok(())
    } else {
        Err(ArchiveError::Invalid(format!(
            "kind '{kind}' must match ^[a-z0-9-]+$ (<= 64 chars)"
        )))
    }
}

fn validate_id(id: &str) -> Result<(), ArchiveError> {
    let ok = id != "."
        && id != ".."
        && !id.is_empty()
        && id.len() <= 128
        && id.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_' || c == '.'
        });
    if ok {
        Ok(())
    } else {
        Err(ArchiveError::Invalid(format!(
            "id '{id}' must match ^[a-z0-9._-]+$ (<= 128 chars)"
        )))
    }
}

/// Locator stamps are produced by [`fresh_stamp`] (`YYYYmmddTHHMMSS.mmm`)
/// and must never act as a traversal vector when a locator is rebuilt
/// from external input: fixed charset, bounded length, no `.`/`..`.
fn validate_stamp(stamp: &str) -> Result<(), ArchiveError> {
    let ok = !stamp.is_empty()
        && stamp.len() <= 32
        && stamp != "."
        && stamp != ".."
        && stamp
            .bytes()
            .all(|b| b.is_ascii_digit() || b == b'T' || b == b'.');
    if ok {
        Ok(())
    } else {
        Err(ArchiveError::Invalid(format!(
            "stamp '{stamp}' must be a UTC stamp like 20261021T085618.123"
        )))
    }
}

/// A fresh directory stamp: fixed-width UTC to millisecond precision,
/// bumped past every existing stamp in the same `id` set — so
/// lexicographic order IS chronological order, same-millisecond writes
/// never collide, and `latest` resolution is honest about creation
/// order.
fn fresh_stamp(existing_max: Option<&str>) -> String {
    let mut t = Utc::now();
    let mut stamp = t.format("%Y%m%dT%H%M%S%.3f").to_string();
    if let Some(max) = existing_max {
        while stamp.as_str() <= max {
            t += chrono::Duration::milliseconds(1);
            stamp = t.format("%Y%m%dT%H%M%S%.3f").to_string();
        }
    }
    stamp
}

/// The greatest stamp directory under an `kind/id` set, if any.
fn max_stamp(base: &Path) -> Option<String> {
    let mut stamps: Vec<String> = match fs::read_dir(base) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .filter_map(|p| p.file_name().and_then(|n| n.to_str().map(str::to_string)))
            .collect(),
        Err(_) => return None,
    };
    stamps.sort();
    stamps.pop()
}

/// Write every source into one tar at `payload_path`, returning the
/// carried entry count (directories and files).
///
/// The tree is walked here (rather than delegated to
/// `tar::Builder::append_path`) so every stored name is constructed
/// relative to the source's own file name — absolute source paths can
/// never leak into entry names, and the entry count falls out of the
/// same walk.
fn write_tar(sources: &[PathBuf], payload_path: &Path) -> Result<u64, ArchiveError> {
    let file = fs::File::create(payload_path).map_err(|e| ArchiveError::Io {
        path: payload_path.to_path_buf(),
        source: e,
    })?;
    set_private_file(&file)?;
    let mut builder = tar::Builder::new(file);
    let mut count: u64 = 0;
    for source in sources {
        let name = source.file_name().ok_or_else(|| {
            ArchiveError::Invalid(format!(
                "source `{}` has no file name to store under",
                source.display()
            ))
        })?;
        count += write_tree(&mut builder, source, Path::new(name))?;
    }
    builder.finish().map_err(|e| ArchiveError::Io {
        path: payload_path.to_path_buf(),
        source: e,
    })?;
    Ok(count)
}

/// Append one source subtree as `stored` (dir or file), returning the
/// entries written. Children are visited in sorted order for
/// deterministic archives; symlinks and special files are skipped with
/// a warning (v1 archives regular files only).
fn write_tree(
    builder: &mut tar::Builder<fs::File>,
    source: &Path,
    stored: &Path,
) -> Result<u64, ArchiveError> {
    let meta = fs::symlink_metadata(source).map_err(|e| ArchiveError::Io {
        path: source.to_path_buf(),
        source: e,
    })?;
    if !meta.is_dir() {
        if meta.is_file() {
            let mut header = tar::Header::new_gnu();
            header.set_entry_type(tar::EntryType::Regular);
            header.set_size(meta.len());
            header.set_mode(0o600);
            set_header_mtime(&mut header, &meta);
            let file = fs::File::open(source).map_err(|e| ArchiveError::Io {
                path: source.to_path_buf(),
                source: e,
            })?;
            builder
                .append_data(&mut header, stored, file)
                .map_err(|e| ArchiveError::Io {
                    path: source.to_path_buf(),
                    source: e,
                })?;
            return Ok(1);
        }
        tracing::warn!(
            source = %source.display(),
            "symlink/special file skipped by plana_archive (v1 archives regular files only)"
        );
        return Ok(0);
    }

    let mut header = tar::Header::new_gnu();
    header.set_entry_type(tar::EntryType::Directory);
    header.set_size(0);
    header.set_mode(0o750);
    set_header_mtime(&mut header, &meta);
    // A directory entry's stored name carries the trailing slash so
    // every tar consumer reads it as a directory.
    builder
        .append_data(&mut header, stored.join(""), std::io::empty())
        .map_err(|e| ArchiveError::Io {
            path: source.to_path_buf(),
            source: e,
        })?;
    let mut count: u64 = 1;

    let mut children: Vec<PathBuf> = fs::read_dir(source)
        .map_err(|e| ArchiveError::Io {
            path: source.to_path_buf(),
            source: e,
        })?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();
    children.sort();
    for child in children {
        count += write_tree(
            builder,
            &child,
            &stored.join(child.file_name().unwrap_or_default()),
        )?;
    }
    Ok(count)
}

/// Hash a file with SHA-256.
fn file_sha256(path: &Path) -> Result<[u8; 32], ArchiveError> {
    let mut file = fs::File::open(path).map_err(|e| ArchiveError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(|e| ArchiveError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    Ok(hasher.finalize().into())
}

/// Post-process a freshly written plain payload: encrypt in place when
/// requested, and answer the stored byte count either way.
fn seal_payload(
    payload_path: &Path,
    encryption: Encryption,
    key: Option<&[u8; 32]>,
) -> Result<u64, ArchiveError> {
    match encryption {
        Encryption::None => {}
        Encryption::Aes256Gcm => {
            let key = key.expect("caller validated key presence for aes-256-gcm");
            let plain = fs::read(payload_path).map_err(|e| ArchiveError::Io {
                path: payload_path.to_path_buf(),
                source: e,
            })?;
            let mut nonce_bytes = [0u8; NONCE_LEN];
            rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
            let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
            // Integrity rides the manifest digest pairing; AEAD here
            // defends confidentiality only.
            let ciphertext = cipher
                .encrypt(
                    Nonce::from_slice(&nonce_bytes),
                    Payload {
                        msg: &plain,
                        aad: &[],
                    },
                )
                .map_err(|_| ArchiveError::Invalid("payload encryption failed".to_string()))?;
            let mut sealed = Vec::with_capacity(ENC_MAGIC.len() + NONCE_LEN + ciphertext.len());
            sealed.extend_from_slice(ENC_MAGIC);
            sealed.extend_from_slice(&nonce_bytes);
            sealed.extend_from_slice(&ciphertext);
            write_private_file(payload_path, &sealed)?;
        }
    }
    let size = fs::metadata(payload_path)
        .map_err(|e| ArchiveError::Io {
            path: payload_path.to_path_buf(),
            source: e,
        })?
        .len();
    Ok(size)
}

/// Read the stored payload back as plain tar bytes (decrypting when the
/// manifest says so), after re-verifying the stored digest.
fn open_payload(
    payload_path: &Path,
    manifest: &ArchiveManifest,
    key: Option<&[u8; 32]>,
) -> Result<Vec<u8>, ArchiveError> {
    let stored = fs::read(payload_path).map_err(|e| ArchiveError::Io {
        path: payload_path.to_path_buf(),
        source: e,
    })?;
    let actual = hex::encode(Sha256::digest(&stored));
    if actual != manifest.payload_sha256 {
        return Err(ArchiveError::Corrupt {
            locator: format!("{}/{}", manifest.kind, manifest.id),
            reason: "stored payload digest does not match the manifest".to_string(),
        });
    }
    match manifest.encryption {
        Encryption::None => Ok(stored),
        Encryption::Aes256Gcm => {
            let key = key.ok_or_else(|| ArchiveError::KeyRequired {
                locator: format!("{}/{}", manifest.kind, manifest.id),
            })?;
            if stored.len() < ENC_MAGIC.len() + NONCE_LEN || &stored[..ENC_MAGIC.len()] != ENC_MAGIC
            {
                return Err(ArchiveError::Corrupt {
                    locator: format!("{}/{}", manifest.kind, manifest.id),
                    reason: "encrypted payload is missing its magic/nonce header".to_string(),
                });
            }
            let nonce = Nonce::from_slice(&stored[ENC_MAGIC.len()..ENC_MAGIC.len() + NONCE_LEN]);
            let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
            let plain = cipher
                .decrypt(
                    nonce,
                    Payload {
                        msg: &stored[ENC_MAGIC.len() + NONCE_LEN..],
                        aad: &[],
                    },
                )
                .map_err(|_| ArchiveError::Decrypt {
                    locator: format!("{}/{}", manifest.kind, manifest.id),
                })?;
            Ok(plain)
        }
    }
}

/// Whether the bytes decode as a tar stream (used by `verify`).
fn tar_decodes(bytes: &[u8]) -> bool {
    let mut archive = tar::Archive::new(bytes);
    let mut entries = match archive.entries() {
        Ok(entries) => entries,
        Err(_) => return false,
    };
    loop {
        match entries.next() {
            Some(Ok(_)) => continue,
            Some(Err(_)) => return false,
            None => return true,
        }
    }
}

/// Read and schema-check one archive directory's manifest.
fn read_manifest(dir: &Path) -> Result<ArchiveManifest, ArchiveError> {
    let manifest_path = dir.join(MANIFEST_FILE);
    let raw = fs::read_to_string(&manifest_path).map_err(|e| ArchiveError::Io {
        path: manifest_path.clone(),
        source: e,
    })?;
    let manifest: ArchiveManifest =
        serde_json::from_str(&raw).map_err(|e| ArchiveError::Corrupt {
            locator: dir.display().to_string(),
            reason: e.to_string(),
        })?;
    if manifest.schema != ARCHIVE_MANIFEST_SCHEMA {
        return Err(ArchiveError::Corrupt {
            locator: dir.display().to_string(),
            reason: format!(
                "manifest schema {} is not supported (want {ARCHIVE_MANIFEST_SCHEMA})",
                manifest.schema
            ),
        });
    }
    Ok(manifest)
}

/// The tar entry's path confined under the restore target: relative,
/// no `..` components, non-empty.
fn confined_relative(entry_path: &Path) -> Option<PathBuf> {
    if entry_path.is_absolute() {
        return None;
    }
    let mut out = PathBuf::new();
    for component in entry_path.components() {
        match component {
            std::path::Component::Normal(seg) => out.push(seg),
            std::path::Component::CurDir => {}
            _ => return None,
        }
    }
    if out.as_os_str().is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Stamp a header with the file's mtime (0 when the clock is off).
fn set_header_mtime(header: &mut tar::Header, meta: &fs::Metadata) {
    let secs = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    header.set_mtime(secs);
}

fn set_private_dir(path: &Path) -> Result<(), ArchiveError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|e| {
            ArchiveError::Io {
                path: path.to_path_buf(),
                source: e,
            }
        })?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

fn set_private_file(file: &fs::File) -> Result<(), ArchiveError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|e| ArchiveError::Io {
                path: PathBuf::from("<archive file>"),
                source: e,
            })?;
    }
    #[cfg(not(unix))]
    {
        let _ = file;
    }
    Ok(())
}

fn write_private_file(path: &Path, bytes: &[u8]) -> Result<(), ArchiveError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map_err(|e| ArchiveError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
    set_private_file(&file)?;
    file.write_all(bytes).map_err(|e| ArchiveError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    fn sample_request(root: &Path, name: &str) -> (ArchiveRequest, PathBuf) {
        // A source tree: <root>/src-<name>/config.toml + nested/tool.json
        let source = root.join(format!("src-{name}"));
        let nested = source.join("nested");
        fs::create_dir_all(&nested).unwrap();
        fs::write(source.join("config.toml"), "id = \"demo\"\n").unwrap();
        fs::write(nested.join("tool.json"), "{\"ok\":true}").unwrap();
        (
            ArchiveRequest {
                kind: "plugins".into(),
                id: format!("acme-{name}"),
                sources: vec![source.clone()],
                source: "uninstall".into(),
                meta: BTreeMap::from([("version".into(), "0.1.0".into())]),
                encryption: Encryption::None,
            },
            source,
        )
    }

    fn key32() -> [u8; 32] {
        let mut key = [7u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut key);
        key
    }

    #[test]
    fn archive_list_inspect_round_trip_in_plaintext() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ArchiveStore::open(tmp.path().join("store")).unwrap();
        let (request, source) = sample_request(tmp.path(), "plain");

        let receipt = store.archive(&request, None).unwrap();
        assert_eq!(receipt.manifest.kind, "plugins");
        assert_eq!(receipt.manifest.encryption, Encryption::None);
        assert_eq!(
            receipt.manifest.entry_count, 4,
            "dir + config + nested dir + tool.json"
        );
        assert!(receipt.payload_bytes > 0);

        let listed = store.list().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].locator.id, "acme-plain");

        let manifest = store.inspect(&receipt.locator).unwrap();
        assert_eq!(manifest, receipt.manifest);
        assert!(
            manifest
                .locator(receipt.locator.stamp.as_deref().unwrap())
                .contains("plugins/acme-plain/")
        );

        let _unused = source;
    }

    #[test]
    fn restore_reproduces_the_sources_under_the_target() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ArchiveStore::open(tmp.path().join("store")).unwrap();
        let (request, source) = sample_request(tmp.path(), "restore");
        let receipt = store.archive(&request, None).unwrap();

        let target = tmp.path().join("restored");
        let report = store
            .restore(&receipt.locator, &target, &RestoreOptions::default(), None)
            .unwrap();
        assert!(report.applied);
        // The archive keeps the source's own name: dir + file entries
        // unpack under target/src-restore/.
        assert_eq!(
            report.restored.len(),
            4,
            "dir, config, nested dir, tool.json"
        );
        assert!(
            !report
                .restored
                .iter()
                .any(|p| p.to_string_lossy().contains(".."))
        );

        let config = fs::read_to_string(target.join("src-restore").join("config.toml")).unwrap();
        assert_eq!(config, "id = \"demo\"\n");
        let tool = fs::read_to_string(target.join("src-restore").join("nested").join("tool.json"))
            .unwrap();
        assert_eq!(tool, "{\"ok\":true}");
        // Directory permissions were set private.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&target).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o700);
        }
        let _unused = source;
    }

    #[test]
    fn encrypted_archives_round_trip_and_a_wrong_key_fails_closed() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ArchiveStore::open(tmp.path().join("store")).unwrap();
        let (mut request, _source) = sample_request(tmp.path(), "enc");
        request.encryption = Encryption::Aes256Gcm;
        let key = key32();

        // No key with an encrypted request = invalid, before any write.
        assert!(matches!(
            store.archive(&request, None),
            Err(ArchiveError::Invalid(_))
        ));

        let receipt = store.archive(&request, Some(&key)).unwrap();
        assert_eq!(receipt.manifest.encryption, Encryption::Aes256Gcm);

        // Verify without a key: the payload digest works, decryption needs the key.
        let report = store.verify(&receipt.locator, None).unwrap_err();
        assert!(matches!(report, ArchiveError::KeyRequired { .. }));

        let report = store.verify(&receipt.locator, Some(&key)).unwrap();
        assert!(report.payload_digest_ok);
        assert_eq!(report.payload_decodes, Some(true));

        // Wrong key = fail closed (digest pairing passes — same bytes —
        // but AEAD authentication rejects).
        let wrong = key32();
        let wrong_key_result = store.restore(
            &receipt.locator,
            tmp.path().join("nope"),
            &RestoreOptions::default(),
            Some(&wrong),
        );
        assert!(matches!(
            wrong_key_result,
            Err(ArchiveError::Decrypt { .. })
        ));

        // The right key restores byte-identical content.
        let target = tmp.path().join("restored");
        store
            .restore(
                &receipt.locator,
                &target,
                &RestoreOptions::default(),
                Some(&key),
            )
            .unwrap();
        let config = fs::read_to_string(target.join("src-enc").join("config.toml")).unwrap();
        assert_eq!(config, "id = \"demo\"\n");
    }

    #[test]
    fn a_tampered_payload_fails_the_digest_pairing() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ArchiveStore::open(tmp.path().join("store")).unwrap();
        let (request, _source) = sample_request(tmp.path(), "tamper");
        let receipt = store.archive(&request, None).unwrap();

        let dir = receipt.locator.dir(store.root()).unwrap();
        let payload = dir.join(PAYLOAD_FILE);
        let mut bytes = fs::read(&payload).unwrap();
        bytes[0] ^= 0xff;
        fs::write(&payload, &bytes).unwrap();

        let err = store
            .restore(
                &receipt.locator,
                tmp.path().join("out"),
                &RestoreOptions::default(),
                None,
            )
            .unwrap_err();
        assert!(matches!(err, ArchiveError::Corrupt { .. }), "got {err:?}");
    }

    #[test]
    fn an_in_place_tamper_that_keeps_the_tar_valid_is_caught() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ArchiveStore::open(tmp.path().join("store")).unwrap();
        let (request, _source) = sample_request(tmp.path(), "pairing");
        let receipt = store.archive(&request, None).unwrap();

        let dir = receipt.locator.dir(store.root()).unwrap();
        let payload = dir.join(PAYLOAD_FILE);
        let mut bytes = fs::read(&payload).unwrap();
        // Layout: [dir hdr][file hdr][config data][dir hdr][file hdr]
        // [tool data]. Flip a byte inside the first file's data section
        // (1024 + 6): the tar stays structurally valid, so only the
        // manifest↔payload digest pairing can notice — a parse error
        // would not.
        assert!(bytes.len() > 2048, "fixture too small: {}", bytes.len());
        bytes[1024 + 6] ^= 0xff;
        fs::write(&payload, &bytes).unwrap();

        let report = store.verify(&receipt.locator, None).unwrap();
        assert!(!report.payload_digest_ok, "verify must flag the pairing");

        let err = store.restore(
            &receipt.locator,
            tmp.path().join("out"),
            &RestoreOptions::default(),
            None,
        );
        assert!(
            matches!(err, Err(ArchiveError::Corrupt { .. })),
            "got {err:?}"
        );
    }

    #[test]
    fn tar_entries_that_escape_the_target_are_refused() {
        // Hand-craft tars carrying an absolute path and a `..` walk.
        let tmp = tempfile::tempdir().unwrap();
        let store = ArchiveStore::open(tmp.path().join("store")).unwrap();

        // tar-rs refuses absolute/..-paths at set_path time — so the
        // attacks are modeled the way a hostile payload actually
        // arrives: a valid tar whose header name bytes were rewritten
        // afterwards.
        let mut builder = tar::Builder::new(Vec::new());
        let mut header = tar::Header::new_gnu();
        header.set_size(3);
        header.set_entry_type(tar::EntryType::Regular);
        header.set_cksum();
        header.set_path("benign.txt").unwrap();
        builder.append(&header, b"pwd".as_slice()).unwrap();
        let mut evil: Vec<u8> = builder.into_inner().unwrap();
        rewrite_tar_name(&mut evil, b"/etc/passwd");

        let mut builder = tar::Builder::new(Vec::new());
        let mut header = tar::Header::new_gnu();
        header.set_size(4);
        header.set_entry_type(tar::EntryType::Regular);
        header.set_cksum();
        header.set_path("deep/inner.txt").unwrap();
        builder.append(&header, b"evil".as_slice()).unwrap();
        let mut walker: Vec<u8> = builder.into_inner().unwrap();
        rewrite_tar_name(&mut walker, b"deep/../evil.txt");

        for (id, payload) in [("evil", evil), ("walker", walker)] {
            let (_dir, receipt) = store_hostile_payload(&store, tmp.path(), id, &payload);
            let target = tmp.path().join(format!("out-{id}"));
            let err = store
                .restore(&receipt.locator, &target, &RestoreOptions::default(), None)
                .unwrap_err()
                .to_string();
            assert!(err.contains("escape"), "got {err}");
            assert!(!target.join("etc").exists(), "nothing escaped");
            assert!(!target.join("evil.txt").exists(), "no `..` walk landed");
            assert!(!tmp.path().join("evil.txt").exists());
        }
    }

    #[test]
    fn hard_link_entries_pointing_outside_the_target_are_refused() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ArchiveStore::open(tmp.path().join("store")).unwrap();

        // A hostile payload can carry entry shapes write_tar never
        // produces: a hard link whose link target points outside the
        // restore target.
        let mut builder = tar::Builder::new(Vec::new());
        let mut header = tar::Header::new_gnu();
        header.set_size(0);
        header.set_entry_type(tar::EntryType::Link);
        header.set_cksum();
        header.set_path("benign.txt").unwrap();
        header.set_link_name(Path::new("/etc/passwd")).unwrap();
        builder.append(&header, std::io::empty()).unwrap();
        let payload: Vec<u8> = builder.into_inner().unwrap();

        let (_dir, receipt) = store_hostile_payload(&store, tmp.path(), "hardlink", &payload);
        let target = tmp.path().join("out");
        let result = store.restore(&receipt.locator, &target, &RestoreOptions::default(), None);
        assert!(result.is_err(), "hard link to /etc/passwd must be refused");
        assert!(!target.join("benign.txt").exists(), "no link was created");
    }

    #[test]
    fn symlink_then_write_through_is_refused() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ArchiveStore::open(tmp.path().join("store")).unwrap();

        // Entry 1 plants a symlink pointing outside the target; entry 2
        // is a regular file written *through* it. Both entry names are
        // confined — the guard must come from the unpack layer.
        let outside = tmp.path().join("outside");
        let mut builder = tar::Builder::new(Vec::new());
        let mut jump = tar::Header::new_gnu();
        jump.set_size(0);
        jump.set_entry_type(tar::EntryType::Symlink);
        jump.set_cksum();
        jump.set_path("jump").unwrap();
        jump.set_link_name(&outside).unwrap();
        builder.append(&jump, std::io::empty()).unwrap();
        let mut file = tar::Header::new_gnu();
        file.set_size(4);
        file.set_entry_type(tar::EntryType::Regular);
        file.set_cksum();
        file.set_path("jump/evil.txt").unwrap();
        builder.append(&file, b"evil".as_slice()).unwrap();
        let payload: Vec<u8> = builder.into_inner().unwrap();

        let (_dir, receipt) = store_hostile_payload(&store, tmp.path(), "symlink", &payload);
        let target = tmp.path().join("out");
        let result = store.restore(&receipt.locator, &target, &RestoreOptions::default(), None);
        assert!(result.is_err(), "write-through a symlink must be refused");
        assert!(!outside.join("evil.txt").exists(), "nothing landed outside");
    }

    #[test]
    fn write_tree_edge_shapes_round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ArchiveStore::open(tmp.path().join("store")).unwrap();
        let source = tmp.path().join("src-edge");
        fs::create_dir_all(source.join("empty dir")).unwrap();
        fs::create_dir_all(source.join("a/b/c/d/e")).unwrap();
        fs::write(source.join("a/b/c/d/e/leaf with space.txt"), "deep").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            fs::write(
                source.join(std::ffi::OsStr::from_bytes(b"raw-\xff-name")),
                "bytes",
            )
            .unwrap();
        }

        let request = ArchiveRequest {
            kind: "plugins".into(),
            id: "acme-edge".into(),
            sources: vec![source],
            source: "test".into(),
            meta: BTreeMap::new(),
            encryption: Encryption::None,
        };
        let receipt = store.archive(&request, None).unwrap();
        #[cfg(unix)]
        let want_entries = 9u64; // 7 dirs + 2 files
        #[cfg(not(unix))]
        let want_entries = 8u64; // the raw non-UTF-8 name is unix-only
        assert_eq!(receipt.manifest.entry_count, want_entries);

        let target = tmp.path().join("out");
        let report = store
            .restore(&receipt.locator, &target, &RestoreOptions::default(), None)
            .unwrap();
        assert_eq!(report.restored.len(), want_entries as usize);
        assert_eq!(
            fs::read_to_string(
                target
                    .join("src-edge")
                    .join("a/b/c/d/e")
                    .join("leaf with space.txt")
            )
            .unwrap(),
            "deep"
        );
        assert!(target.join("src-edge").join("empty dir").is_dir());
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            let raw = target
                .join("src-edge")
                .join(std::ffi::OsStr::from_bytes(b"raw-\xff-name"));
            assert_eq!(fs::read_to_string(raw).unwrap(), "bytes");
        }
    }

    #[test]
    fn stamps_are_fixed_width_and_sort_chronologically() {
        let s = fresh_stamp(None);
        assert_eq!(s.len(), 19, "YYYYmmddTHHMMSS.mmm: {s}");
        assert!(
            s.bytes()
                .all(|b| b.is_ascii_digit() || b == b'T' || b == b'.'),
            "unexpected stamp shape: {s}"
        );
        // Millisecond precision with fixed width: lexicographic order is
        // chronological order, across a year boundary too.
        assert!("20261231T235959.999" < "20270101T000000.000");
        assert!("20270101T000000.000" < "20270101T000000.001");
    }

    /// Rewrite a GNU tar header's 100-byte name field in place and
    /// recompute the checksum the way a hostile payload author would
    /// (checksum field 148..156 treated as spaces, octal, NUL-space
    /// terminated).
    fn rewrite_tar_name(tar: &mut [u8], name: &[u8]) {
        assert!(name.len() <= 100);
        let len = name.len();
        tar[..len].copy_from_slice(name);
        tar[len..100].fill(0);
        tar[148..156].copy_from_slice(b"        ");
        let sum: u32 = tar[..512].iter().map(|b| *b as u32).sum();
        let chksum = format!("{sum:06o}\0 ");
        tar[148..156].copy_from_slice(chksum.as_bytes());
    }

    /// Store hostile payload bytes as a well-formed archive (manifest
    /// digest refreshed to vouch for them), so only the restore-level
    /// guards — not digest pairing — can catch the attack.
    fn store_hostile_payload(
        store: &ArchiveStore,
        tmp: &Path,
        id: &str,
        payload: &[u8],
    ) -> (PathBuf, ArchiveReceipt) {
        let seed = tmp.join(format!("seed-{id}"));
        fs::write(&seed, "seed").unwrap();
        let request = ArchiveRequest {
            kind: "plugins".into(),
            id: id.into(),
            sources: vec![seed],
            source: "test".into(),
            meta: BTreeMap::new(),
            encryption: Encryption::None,
        };
        let receipt = store.archive(&request, None).unwrap();
        let dir = receipt.locator.dir(store.root()).unwrap();
        fs::write(dir.join(PAYLOAD_FILE), payload).unwrap();
        let manifest: ArchiveManifest = read_manifest(&dir).unwrap();
        let manifest = ArchiveManifest {
            payload_sha256: hex::encode(Sha256::digest(payload)),
            payload_bytes: payload.len() as u64,
            ..manifest
        };
        write_private_file(
            &dir.join(MANIFEST_FILE),
            manifest.canonical_json().unwrap().as_bytes(),
        )
        .unwrap();
        (dir, receipt)
    }

    #[test]
    fn a_symlink_entry_in_the_payload_is_refused_before_any_write() {
        // A hostile payload plants a symlink and then a file "through"
        // it: the file's own name is clean, so only the link-entry
        // refusal keeps the write out of the pointed-to directory.
        let tmp = tempfile::tempdir().unwrap();
        let store = ArchiveStore::open(tmp.path().join("store")).unwrap();

        let mut builder = tar::Builder::new(Vec::new());
        let mut link_header = tar::Header::new_gnu();
        link_header.set_entry_type(tar::EntryType::Symlink);
        link_header.set_size(0);
        link_header.set_path("outside").unwrap();
        link_header.set_link_name("/etc").unwrap();
        // The checksum is computed over the finished header — set it last.
        link_header.set_cksum();
        builder.append(&link_header, std::io::empty()).unwrap();
        let mut file_header = tar::Header::new_gnu();
        file_header.set_entry_type(tar::EntryType::Regular);
        file_header.set_size(4);
        file_header.set_path("outside/evil.txt").unwrap();
        file_header.set_cksum();
        builder.append(&file_header, b"evil".as_slice()).unwrap();
        let evil: Vec<u8> = builder.into_inner().unwrap();

        let request = ArchiveRequest {
            kind: "plugins".into(),
            id: "symlink".into(),
            sources: vec![tmp.path().join("seed")],
            source: "test".into(),
            meta: BTreeMap::new(),
            encryption: Encryption::None,
        };
        fs::write(tmp.path().join("seed"), "seed").unwrap();
        let receipt = store.archive(&request, None).unwrap();
        let dir = receipt.locator.dir(store.root()).unwrap();
        let manifest: ArchiveManifest = read_manifest(&dir).unwrap();
        fs::write(dir.join(PAYLOAD_FILE), &evil).unwrap();
        let digest = hex::encode(Sha256::digest(&evil));
        let manifest = ArchiveManifest {
            payload_sha256: digest,
            payload_bytes: evil.len() as u64,
            ..manifest
        };
        write_private_file(
            &dir.join(MANIFEST_FILE),
            manifest.canonical_json().unwrap().as_bytes(),
        )
        .unwrap();

        let target = tmp.path().join("out");
        let err = store
            .restore(&receipt.locator, &target, &RestoreOptions::default(), None)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("symlink/hard link"),
            "the link refusal must fire: {err}"
        );
        assert!(
            !target.exists(),
            "nothing was written on the refused restore"
        );
    }

    #[test]
    fn dry_run_writes_nothing_and_conflicts_are_reported() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ArchiveStore::open(tmp.path().join("store")).unwrap();
        let (request, _source) = sample_request(tmp.path(), "dry");
        let receipt = store.archive(&request, None).unwrap();

        // Dry run into a fresh target: applied=false, no files written.
        let target = tmp.path().join("target");
        let report = store
            .restore(
                &receipt.locator,
                &target,
                &RestoreOptions {
                    dry_run: true,
                    ..Default::default()
                },
                None,
            )
            .unwrap();
        assert!(!report.applied);
        assert_eq!(report.restored.len(), 4);
        assert!(!target.exists(), "dry run must not create the target");

        // Real restore, then a second restore without overwrite = conflict.
        store
            .restore(&receipt.locator, &target, &RestoreOptions::default(), None)
            .unwrap();
        let conflict = store.restore(&receipt.locator, &target, &RestoreOptions::default(), None);
        assert!(matches!(
            conflict,
            Err(ArchiveError::Conflict { count: 4, .. })
        ));

        // With overwrite the same restore succeeds.
        store
            .restore(
                &receipt.locator,
                &target,
                &RestoreOptions {
                    overwrite: true,
                    ..Default::default()
                },
                None,
            )
            .unwrap();
    }

    #[test]
    fn backup_existing_moves_the_old_target_aside() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ArchiveStore::open(tmp.path().join("store")).unwrap();
        let (request, _source) = sample_request(tmp.path(), "backup");
        let receipt = store.archive(&request, None).unwrap();

        let target = tmp.path().join("target");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("old-marker"), "old").unwrap();

        let report = store
            .restore(
                &receipt.locator,
                &target,
                &RestoreOptions {
                    backup_existing: true,
                    overwrite: true,
                    ..Default::default()
                },
                None,
            )
            .unwrap();
        let backup = report.backup_path.expect("backup recorded");
        assert!(backup.exists());
        assert!(
            backup.join("old-marker").exists(),
            "the old content rode along"
        );
        assert!(
            target.join("src-backup").join("config.toml").exists(),
            "the restore landed"
        );
    }

    #[test]
    fn latest_locator_resolves_and_purge_removes_the_set() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ArchiveStore::open(tmp.path().join("store")).unwrap();
        let (mut request, _source) = sample_request(tmp.path(), "latest");
        request.id = "acme-latest".into();

        let first = store.archive(&request, None).unwrap();
        let second = store.archive(&request, None).unwrap();
        assert_ne!(
            first.locator.stamp, second.locator.stamp,
            "stamps are unique"
        );

        let latest = ArchiveLocator::latest("plugins", "acme-latest");
        let latest_manifest = store.inspect(&latest).unwrap();
        assert_eq!(latest_manifest.payload_bytes, second.manifest.payload_bytes);

        let pinned = ArchiveLocator::at(
            "plugins",
            "acme-latest",
            first.locator.stamp.clone().unwrap(),
        );
        assert_eq!(
            store.inspect(&pinned).unwrap().payload_bytes,
            first.manifest.payload_bytes
        );

        // Purging the latest drops the set to the first archive, which
        // `latest` now resolves to; the pinned first stamp is untouched
        // and survives.
        store.purge(&latest).unwrap();
        assert_eq!(
            store.inspect(&latest).unwrap().payload_bytes,
            first.manifest.payload_bytes,
            "latest falls back to the remaining archive"
        );
        assert!(store.inspect(&pinned).is_ok(), "the pinned stamp survived");

        // Purge the whole remaining set: now latest is NotFound.
        store
            .purge(&ArchiveLocator::latest("plugins", "acme-latest"))
            .unwrap();
        assert!(matches!(
            store.inspect(&ArchiveLocator::latest("plugins", "acme-latest")),
            Err(ArchiveError::NotFound { .. })
        ));
    }

    #[test]
    fn request_validation_rejects_empty_and_traversal_inputs() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ArchiveStore::open(tmp.path().join("store")).unwrap();
        let mut request = sample_request(tmp.path(), "valid").0;

        request.kind = "Bad_Kind".into();
        assert!(matches!(
            store.archive(&request, None),
            Err(ArchiveError::Invalid(_))
        ));

        request.kind = "plugins".into();
        request.id = "..".into();
        assert!(matches!(
            store.archive(&request, None),
            Err(ArchiveError::Invalid(_))
        ));

        request.id = "acme-ok".into();
        request.sources.clear();
        assert!(matches!(
            store.archive(&request, None),
            Err(ArchiveError::Invalid(_))
        ));

        request.sources = vec![tmp.path().join("does-not-exist")];
        assert!(matches!(
            store.archive(&request, None),
            Err(ArchiveError::Invalid(_))
        ));

        // A source with no file name to store under (`/`, `.`, `..`)
        // cannot be named inside a tar.
        request.sources = vec![PathBuf::from("/")];
        assert!(matches!(
            store.archive(&request, None),
            Err(ArchiveError::Invalid(_))
        ));
    }

    #[test]
    fn unknown_locators_answer_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ArchiveStore::open(tmp.path().join("store")).unwrap();
        assert!(matches!(
            store.inspect(&ArchiveLocator::latest("plugins", "ghost")),
            Err(ArchiveError::NotFound { .. })
        ));
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn locator_fields_rebuilt_from_external_input_cannot_traverse() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ArchiveStore::open(tmp.path().join("store")).unwrap();

        let hostile_stamp = ArchiveLocator::at("plugins", "x", "../../escape");
        assert!(matches!(
            store.inspect(&hostile_stamp),
            Err(ArchiveError::Invalid(_))
        ));
        assert!(matches!(
            store.inspect(&ArchiveLocator::latest("../outside", "x")),
            Err(ArchiveError::Invalid(_))
        ));
        assert!(matches!(
            store.inspect(&ArchiveLocator::latest("plugins", "..")),
            Err(ArchiveError::Invalid(_))
        ));
        // A stamp rebuilt from external input must not smuggle foreign
        // characters through (the charset guard fires before any join).
        let smuggled = ArchiveLocator::at("plugins", "x", "20261021T085618.12;");
        assert!(matches!(
            store.inspect(&smuggled),
            Err(ArchiveError::Invalid(_))
        ));

        // purge must refuse to act outside the root.
        let outside = tmp.path().join("precious");
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("keep"), "x").unwrap();
        let hostile = ArchiveLocator {
            kind: "plugins".into(),
            id: "x".into(),
            stamp: Some("..".into()),
        };
        assert!(matches!(
            store.purge(&hostile),
            Err(ArchiveError::Invalid(_))
        ));
        assert!(
            outside.join("keep").exists(),
            "nothing outside the root was touched"
        );
    }

    #[test]
    fn concurrent_archives_of_one_id_claim_distinct_stamps() {
        let tmp = tempfile::tempdir().unwrap();
        let store = Arc::new(ArchiveStore::open(tmp.path().join("store")).unwrap());
        let (request, _source) = sample_request(tmp.path(), "race");
        let request = Arc::new(request);
        let barrier = Arc::new(std::sync::Barrier::new(8));

        let mut handles = Vec::new();
        for _ in 0..8 {
            let store = store.clone();
            let request = request.clone();
            let barrier = barrier.clone();
            handles.push(std::thread::spawn(move || {
                barrier.wait();
                store.archive(&request, None)
            }));
        }
        let mut stamps = Vec::new();
        for handle in handles {
            stamps.push(handle.join().unwrap().unwrap().locator.stamp.unwrap());
        }
        let unique: std::collections::BTreeSet<&String> = stamps.iter().collect();
        assert_eq!(
            unique.len(),
            stamps.len(),
            "same-millisecond writers must claim distinct stamps: {stamps:?}"
        );

        // Every archive independently verifies — no two writers ever
        // shared a payload file.
        for stamp in &stamps {
            let report = store
                .verify(
                    &ArchiveLocator::at("plugins", "acme-race", stamp.clone()),
                    None,
                )
                .unwrap();
            assert!(report.payload_digest_ok);
            assert_eq!(report.payload_decodes, Some(true));
        }
        assert_eq!(store.list().unwrap().len(), 8);
    }

    #[test]
    #[cfg(unix)]
    fn a_failed_capture_leaves_no_partial_stamp_directory() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let store = ArchiveStore::open(tmp.path().join("store")).unwrap();
        let source = tmp.path().join("src-vanish");
        fs::create_dir_all(&source).unwrap();
        let unreadable = source.join("secret.txt");
        fs::write(&unreadable, "s").unwrap();
        fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o000)).unwrap();

        let request = ArchiveRequest {
            kind: "plugins".into(),
            id: "acme-vanish".into(),
            sources: vec![source],
            source: "test".into(),
            meta: BTreeMap::new(),
            encryption: Encryption::None,
        };
        let result = store.archive(&request, None);
        fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(result.is_err(), "the unreadable file must fail the capture");

        let base = store.root().join("plugins").join("acme-vanish");
        let leftovers = fs::read_dir(&base)
            .map(|entries| entries.count())
            .unwrap_or(0);
        assert_eq!(leftovers, 0, "no half-written stamp directory may survive");
        assert!(matches!(
            store.inspect(&ArchiveLocator::latest("plugins", "acme-vanish")),
            Err(ArchiveError::NotFound { .. })
        ));
        assert!(store.list().unwrap().is_empty());
    }
}
