//! Ed25519 package signature verification for subscribed Layer-3 agents.
//!
//! Every subscription from a third-party source must carry an Ed25519
//! signature over a **package manifest** (`agent.sig`, base64) covering every
//! file of the agent package (path + sha256), not just `agent.toml`. The
//! verifying public key comes from the trusted-source registry
//! (`CELESTIA_AGENT_SIGNING_PUBKEY_<SOURCE>` env var, source dashes mapped to
//! underscores); a source without a registered key cannot be verified and is
//! rejected.
//!
//! Local agents (loaded from `.amphoreus/` or the local agents directory) are
//! exempt: they never pass through the subscription path.
//!
//! Manifest format (deterministic, shared with `celestia-devtools
//! sign-agent`):
//!
//! ```json
//! {"version":1,"files":[{"path":"agent.toml","sha256":".."},{"path":"plugin.ts","sha256":".."}]}
//! ```
//!
//! `files` is sorted by path; hashes are lowercase hex SHA-256; `agent.sig`
//! itself and any `.git` directories are excluded.

use anyhow::{Context, Result, anyhow};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::Path;

/// Signature file name inside an agent package.
pub const SIGNATURE_FILE: &str = "agent.sig";

/// Verifies a detached base64 signature over `manifest_bytes`.
///
/// Pure function, unit-testable without touching the registry.
pub fn verify_detached(manifest_bytes: &[u8], sig_b64: &str, pubkey_b64: &str) -> Result<()> {
    let pubkey_bytes: [u8; 32] = decode_b64(pubkey_b64)
        .context("public key is not valid base64")?
        .try_into()
        .map_err(|_| anyhow!("public key must be 32 bytes"))?;
    let pubkey =
        VerifyingKey::from_bytes(&pubkey_bytes).context("public key is not a valid Ed25519 key")?;

    let sig = Signature::from_slice(&decode_b64(sig_b64).context("signature is not valid base64")?)
        .context("signature is not a valid Ed25519 signature")?;

    pubkey
        .verify(manifest_bytes, &sig)
        .map_err(|_| anyhow!("agent package signature verification failed"))
}

/// Rebuilds the canonical manifest for an agent package directory.
///
/// Every regular file under `agent_root` (excluding `.git` and the signature
/// file itself) contributes a `{path, sha256}` entry; entries are sorted by
/// path for determinism.
pub fn build_manifest(agent_root: &Path) -> Result<Vec<u8>> {
    let mut files = Vec::new();
    let mut stack = vec![agent_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)
            .with_context(|| format!("failed to read directory: {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            let rel = path
                .strip_prefix(agent_root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            // Any path component named .git is excluded (covers both the
            // .git directory and .git file forms of submodules/worktrees).
            let is_git = rel.split('/').any(|c| c == ".git");
            let is_sig = rel == SIGNATURE_FILE;
            if is_git || is_sig {
                continue;
            }
            // Symlinks are never followed: they can escape the package root
            // and Python's rglob does not traverse them either.
            let meta = std::fs::symlink_metadata(&path)
                .with_context(|| format!("failed to stat file: {}", path.display()))?;
            if meta.file_type().is_symlink() {
                continue;
            }
            if meta.is_dir() {
                stack.push(path);
                continue;
            }
            let bytes = std::fs::read(&path)
                .with_context(|| format!("failed to read file: {}", path.display()))?;
            let digest = hex::encode(Sha256::digest(&bytes));
            files.push(FileEntry {
                path: rel,
                sha256: digest,
            });
        }
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let manifest = PackageManifest { version: 1, files };
    serde_json::to_vec(&manifest).context("failed to serialize package manifest")
}

/// Verifies an entire agent package against its `agent.sig`.
///
/// Rebuilds the manifest from the current tree, then checks the detached
/// signature over it. Any modification to any package file (or the removal of
/// the signature) fails the verification.
pub fn verify_agent_package(agent_root: &Path, pubkey_b64: &str) -> Result<()> {
    let manifest_bytes = build_manifest(agent_root)?;
    let sig_b64 = std::fs::read_to_string(agent_root.join(SIGNATURE_FILE)).with_context(|| {
        format!(
            "missing {} (required when verify_signature=true)",
            SIGNATURE_FILE
        )
    })?;
    verify_detached(&manifest_bytes, sig_b64.trim(), pubkey_b64)
}

/// Looks up the registered Ed25519 public key for a trusted source.
///
/// Sources in `SubscribeSettings.trusted_sources` are allowed to subscribe,
/// but only sources with a registered key can actually be verified;
/// unverifiable sources are rejected when `verify_signature` is enabled.
///
/// Dashes in source names map to underscores (environment variables cannot
/// carry dashes), e.g. `amphoreus-agents` →
/// `CELESTIA_AGENT_SIGNING_PUBKEY_AMPHOREUS_AGENTS`.
pub fn pubkey_for(source: &str) -> Option<String> {
    let key = source.to_uppercase().replace('-', "_");
    std::env::var(format!("CELESTIA_AGENT_SIGNING_PUBKEY_{}", key))
        .ok()
        .filter(|s| !s.is_empty())
}

/// Extracts the "owner" identity of a subscription target for the
/// trusted-source allow-list. For github-style `owner/repo` entries the owner
/// is the first path segment; for URL sources it is the URL host.
pub fn subscription_owner(repo: &str) -> String {
    let repo = repo.trim();
    if let Some(rest) = repo.split("://").nth(1) {
        rest.split('/').next().unwrap_or("").to_string()
    } else {
        repo.split('/').next().unwrap_or("").to_string()
    }
}

#[derive(Serialize)]
struct PackageManifest {
    version: u32,
    files: Vec<FileEntry>,
}

#[derive(Serialize)]
struct FileEntry {
    path: String,
    sha256: String,
}

fn decode_b64(s: &str) -> Result<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(s.trim())
        .map_err(|e| anyhow!("base64 decode failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use std::fs;

    fn keypair(seed: u8) -> (SigningKey, Vec<u8>) {
        let sk = SigningKey::from_bytes(&[seed; 32]);
        let pk = sk.verifying_key().to_bytes().to_vec();
        (sk, pk)
    }

    fn b64(bytes: &[u8]) -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(bytes)
    }

    fn sign_package(agent_root: &Path, sk: &SigningKey) {
        let manifest = build_manifest(agent_root).unwrap();
        let sig = sk.sign(&manifest);
        fs::write(agent_root.join(SIGNATURE_FILE), b64(&sig.to_bytes())).unwrap();
    }

    fn sample_package(root: &Path) {
        fs::create_dir_all(root.join("sub")).unwrap();
        fs::write(
            root.join("agent.toml"),
            b"[agent]\nid = \"demo\"\nlayer = 3\n",
        )
        .unwrap();
        fs::write(
            root.join("plugin.ts"),
            b"globalThis.handleRequest = null;\n",
        )
        .unwrap();
        fs::write(root.join("sub/util.js"), b"export const x = 1;\n").unwrap();
    }

    #[test]
    fn verify_accepts_genuine_package() -> Result<()> {
        let (sk, pk) = keypair(1);
        let root = std::env::temp_dir().join(format!("sigtest-ok-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        sample_package(&root);
        sign_package(&root, &sk);
        verify_agent_package(&root, &b64(&pk))?;
        let _ = fs::remove_dir_all(&root);
        Ok(())
    }

    #[test]
    fn verify_rejects_tampered_code_file() {
        let (sk, pk) = keypair(2);
        let root = std::env::temp_dir().join(format!("sigtest-tamper-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        sample_package(&root);
        sign_package(&root, &sk);
        // Tamper with a code file (not agent.toml) — must be detected.
        fs::write(root.join("plugin.ts"), b"evil code\n").unwrap();
        assert!(verify_agent_package(&root, &b64(&pk)).is_err());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn verify_rejects_added_file() {
        let (sk, pk) = keypair(3);
        let root = std::env::temp_dir().join(format!("sigtest-add-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        sample_package(&root);
        sign_package(&root, &sk);
        fs::write(root.join("backdoor.sh"), b"#!/bin/sh\n").unwrap();
        assert!(verify_agent_package(&root, &b64(&pk)).is_err());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn verify_rejects_missing_signature() {
        let (_, pk) = keypair(4);
        let root = std::env::temp_dir().join(format!("sigtest-nosig-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        sample_package(&root);
        assert!(verify_agent_package(&root, &b64(&pk)).is_err());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn manifest_is_deterministic() -> Result<()> {
        let root = std::env::temp_dir().join(format!("sigtest-det-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        sample_package(&root);
        let a = build_manifest(&root)?;
        let b = build_manifest(&root)?;
        assert_eq!(a, b);
        let _ = fs::remove_dir_all(&root);
        Ok(())
    }

    #[test]
    fn subscription_owner_parses_github_and_url() {
        assert_eq!(
            subscription_owner("amphoreus-agents/code-reviewer"),
            "amphoreus-agents"
        );
        assert_eq!(
            subscription_owner("https://example.com/team/agent.git"),
            "example.com"
        );
    }

    #[test]
    fn pubkey_for_maps_dashes_to_underscores() {
        // SAFETY: tests run single-threaded per case; env mutation is isolated.
        unsafe {
            std::env::set_var("CELESTIA_AGENT_SIGNING_PUBKEY_AMPHOREUS_AGENTS", "k");
        }
        assert_eq!(pubkey_for("amphoreus-agents"), Some("k".to_string()));
        unsafe {
            std::env::remove_var("CELESTIA_AGENT_SIGNING_PUBKEY_AMPHOREUS_AGENTS");
        }
    }

    #[test]
    fn verify_rejects_wrong_key() {
        let (sk, _) = keypair(5);
        let (_, other_pk) = keypair(6);
        let root = std::env::temp_dir().join(format!("sigtest-key-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        sample_package(&root);
        sign_package(&root, &sk);
        assert!(verify_agent_package(&root, &b64(&other_pk)).is_err());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn manifest_supports_non_ascii_paths() -> Result<()> {
        let (sk, pk) = keypair(7);
        let root = std::env::temp_dir().join(format!("sigtest-utf8-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("子目录")).unwrap();
        fs::write(
            root.join("agent.toml"),
            b"[agent]\nid = \"demo\"\nlayer = 3\n",
        )
        .unwrap();
        fs::write(root.join("子目录/说明.txt"), "你好\n".as_bytes()).unwrap();
        sign_package(&root, &sk);
        // Round-trip must survive raw UTF-8 paths (sign-agent uses
        // ensure_ascii=False to match serde_json's raw UTF-8 output).
        verify_agent_package(&root, &b64(&pk))?;
        let _ = fs::remove_dir_all(&root);
        Ok(())
    }

    #[test]
    fn manifest_excludes_git_file_entries() -> Result<()> {
        let root = std::env::temp_dir().join(format!("sigtest-gitfile-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        sample_package(&root);
        // .git as a FILE (submodule/worktree form) must be excluded too.
        fs::write(root.join(".git"), b"gitdir: ../foo\n").unwrap();
        fs::create_dir_all(root.join("sub/.git")).unwrap();
        fs::write(root.join("sub/.git/config"), b"[core]\n").unwrap();
        let manifest = build_manifest(&root)?;
        let text = String::from_utf8(manifest)?;
        assert!(
            !text.contains(".git"),
            "manifest must exclude .git entries: {text}"
        );
        let _ = fs::remove_dir_all(&root);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn manifest_skips_symlinks() -> Result<()> {
        use std::os::unix::fs::symlink;
        let root = std::env::temp_dir().join(format!("sigtest-symlink-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        sample_package(&root);
        let outside = std::env::temp_dir().join(format!("sigtest-outside-{}", std::process::id()));
        fs::write(&outside, b"secret").unwrap();
        symlink(&outside, root.join("leak.txt")).unwrap();
        symlink(&outside, root.join("leakdir")).unwrap();
        let manifest = build_manifest(&root)?;
        let text = String::from_utf8(manifest)?;
        assert!(!text.contains("leak"), "symlinks must be skipped: {text}");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_file(&outside);
        Ok(())
    }
}
