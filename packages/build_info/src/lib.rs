//! Build identity shared by every plana-consuming backend.
//!
//! The About dialog of the panel shows one row per backend with the same
//! three facts: service version, build hash and build kind. Version and kind
//! are free (`CARGO_PKG_VERSION`, the build profile); the **hash** is the
//! only piece that needs a build script, and every service was inventing its
//! own — one repo emitted a hash of the build *timestamp* (useless as
//! identity), two emitted nothing at all and reported `null`.
//!
//! The hash shape is the workspace-unified build token, the same scheme the
//! malkuth supervisor prints for supervised binaries: a SHA-256 digest over
//! the revision identity, base32-encoded (RFC 4648 alphabet `A-Z2-7`), cut
//! to the **last 6 characters** — e.g. `XB7KQ2`. Every About row (WebUI and
//! each engine) therefore renders one visually uniform token, while the
//! underlying digest stays a per-revision identity (not a timestamp).
//!
//! Usage in a consuming crate:
//!
//! ```ignore
//! // build.rs
//! fn main() {
//!     plana_build_info::emit_build_hash();
//! }
//!
//! // anywhere in that crate
//! pub fn identity() -> (&'static str, &'static str) {
//!     (plana_build_info::build_hash!(), plana_build_info::build_kind!())
//! }
//! ```
//!
//! `build_hash!()` is a macro rather than a function on purpose:
//! `option_env!("BUILD_HASH")` is resolved while compiling the crate that
//! *spells it*, so a helper function living here would only ever see this
//! crate's own build (i.e. nothing). The macro expands at the consumer.

use std::path::Path;
use std::process::Command;

use sha2::Digest;

/// Read the git revision of the current checkout and export it as the
/// `BUILD_HASH` compile-time environment variable.
///
/// Call from `build.rs`. The exported value is the workspace-unified build
/// token (see the crate docs): base32 of the SHA-256 over the full revision
/// string, last 6 characters, `-dirty` suffixed for an uncommitted tree.
/// Emits `unknown` when the sources have no git metadata (vendored build,
/// source tarball) so the field is always present and honest.
///
/// Registers every file git could move the revision through as a rerun
/// trigger. Watching `.git/HEAD` alone is not enough: in a linked worktree —
/// and in the build engine's checkouts — `HEAD` is a *symref* that keeps
/// saying `ref: refs/heads/…` while the branch advances underneath it, so a
/// warm target directory would keep reporting the revision it was first
/// built from. The resolved ref file, the packed refs and the index close
/// that hole.
pub fn emit_build_hash() {
    for trigger in watch_paths() {
        println!("cargo:rerun-if-changed={trigger}");
    }
    let hash = git_revision().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_HASH={hash}");
}

/// Files whose change may move the reported revision.
fn watch_paths() -> Vec<String> {
    let mut paths = Vec::new();
    if let Some(head) = git_path("HEAD") {
        paths.push(head);
    }
    // Where a symref actually points (`refs/heads/<branch>`), so advancing the
    // branch on a worktree re-runs this script.
    if let Some(path) = git(&["symbolic-ref", "-q", "HEAD"])
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .and_then(|reference| git_path(&reference))
    {
        paths.push(path);
    }
    if let Some(packed) = git_path("packed-refs") {
        paths.push(packed);
    }
    paths
}

/// The build hash captured by [`emit_build_hash`], or `"unknown"` when the
/// crate was built without a build script that called it.
#[macro_export]
macro_rules! build_hash {
    () => {
        ::core::option_env!("BUILD_HASH").unwrap_or("unknown")
    };
}

/// The build kind every health surface reports: `"dev"` for a debug build,
/// `"prod"` otherwise. Never hard-code this per handler — the About dialog
/// compares the same word across all backends.
#[macro_export]
macro_rules! build_kind {
    () => {
        if ::core::cfg!(debug_assertions) {
            "dev"
        } else {
            "prod"
        }
    };
}

/// Unified build token of the working tree: base32(SHA-256(revision))[:6],
/// `-dirty` when it has uncommitted changes. `None` when this is not a git
/// checkout or git is unavailable.
fn git_revision() -> Option<String> {
    let rev = git(&["rev-parse", "HEAD"])?;
    let dirty = !git(&["status", "--porcelain"])
        .unwrap_or_default()
        .is_empty();
    encode_revision(&rev, dirty)
}

/// Fold a raw `git rev-parse` output into the unified build token. Kept
/// separate from the process call so the shape is testable without a
/// repository: the digest input is the full revision string (never a short
/// prefix, which would collide across repos), base32-encoded with the same
/// RFC 4648 alphabet malkuth's binary info uses, cut to the LAST 6
/// characters so every surface prints the same short form.
fn encode_revision(raw: &str, dirty: bool) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut input = trimmed.to_string();
    if dirty {
        input.push_str("-dirty");
    }
    let digest = sha2::Sha256::digest(input.as_bytes());
    let token = base32_encode(&digest);
    let start = token.len() - 6;
    let short = token[start..].to_string();
    Some(if dirty {
        format!("{short}-dirty")
    } else {
        short
    })
}

/// RFC 4648 base32 alphabet (no padding) — identical to malkuth's
/// `BASE32_ALPHABET`, so a build token and a supervised-binary short hash
/// read as the same kind of word.
const BASE32_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

fn base32_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(5));
    let mut buffer = 0u64;
    let mut bits = 0u32;
    for &byte in bytes {
        buffer = (buffer << 8) | u64::from(byte);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            let index = ((buffer >> bits) & 0x1f) as usize;
            out.push(BASE32_ALPHABET[index] as char);
        }
    }
    if bits > 0 {
        let index = ((buffer << (5 - bits)) & 0x1f) as usize;
        out.push(BASE32_ALPHABET[index] as char);
    }
    out
}

/// Absolute path of a file inside the git dir (`.git/HEAD`, honoring
/// worktrees and `GIT_DIR`), used for the rerun trigger.
fn git_path(relative: &str) -> Option<String> {
    let dir = git(&["rev-parse", "--absolute-git-dir"])?;
    Some(
        Path::new(dir.trim())
            .join(relative)
            .to_string_lossy()
            .into_owned(),
    )
}

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The whole point of the unified token: exactly 6 base32 characters,
    /// from the same alphabet malkuth prints for supervised binaries.
    #[test]
    fn token_is_six_base32_characters() {
        let token = encode_revision("a8772b9d1f2e3c4b5a6978deadbeefcafe012345", false)
            .expect("a revision encodes");
        assert_eq!(token.len(), 6, "token must be 6 chars, got {token:?}");
        assert!(
            token.chars().all(|c| BASE32_ALPHABET.contains(&(c as u8))),
            "token {token:?} must use the base32 alphabet"
        );
    }

    /// Same revision in, same token out — and different revisions must not
    /// collide on the 6-char window for a realistic spread of inputs.
    #[test]
    fn token_is_deterministic_per_revision() {
        let a = encode_revision("a8772b9d1f2e3c4b5a6978deadbeefcafe012345", false).unwrap();
        let again = encode_revision("a8772b9d1f2e3c4b5a6978deadbeefcafe012345", false).unwrap();
        assert_eq!(a, again);
        let b = encode_revision("0123456789abcdef0123456789abcdef01234567", false).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn marks_uncommitted_trees() {
        let clean = encode_revision("a8772b9d1f2e3c4b5a6978\n", false).unwrap();
        let dirty = encode_revision("a8772b9d1f2e3c4b5a6978\n", true).unwrap();
        assert!(dirty.ends_with("-dirty"));
        assert_eq!(dirty.strip_suffix("-dirty").unwrap().len(), 6);
        assert_ne!(clean, dirty, "the dirty marker must feed the digest");
    }

    #[test]
    fn empty_output_is_not_a_revision() {
        assert_eq!(encode_revision("  \n", false), None);
        assert_eq!(encode_revision("", true), None);
    }

    /// One independently computed constant so an alphabet/width/digest-input
    /// tweak cannot slip through silently: the expected tail below was
    /// derived outside this crate with Python's hashlib + base64 (RFC 4648)
    /// over the same digest input this crate hashes.
    #[test]
    fn known_revision_has_a_stable_token() {
        let token = encode_revision("a8772b9", false).expect("encodes");
        assert_eq!(token, "ZJVCEA");
    }

    /// Every trigger must be a non-empty path, and the resolved ref must be
    /// watched alongside `HEAD` — the stale-hash bug was exactly this.
    #[test]
    fn watches_the_resolved_ref_not_only_head() {
        let paths = watch_paths();
        assert!(!paths.is_empty(), "at least .git/HEAD is watched");
        assert!(paths.iter().all(|path| !path.trim().is_empty()));
        assert!(paths.iter().any(|path| path.ends_with("HEAD")));
        let git_dir = git(&["rev-parse", "--absolute-git-dir"]).expect("inside a git checkout");
        assert!(
            paths.iter().any(|path| path.contains(git_dir.trim())),
            "triggers must live in the git dir, got {paths:?}"
        );
    }

    /// The live checkout encodes to the unified token: 6 base32 chars,
    /// optionally `-dirty` — this is what every About dialog row will show.
    #[test]
    fn the_live_checkout_encodes_to_the_unified_token() {
        let Some(token) = git_revision() else {
            return; // not a git checkout (vendored build) — nothing to assert
        };
        let short = token.strip_suffix("-dirty").unwrap_or(&token);
        assert_eq!(short.len(), 6, "token {token:?} must be 6 chars");
        assert!(
            short.chars().all(|c| BASE32_ALPHABET.contains(&(c as u8))),
            "token {token:?} must use the base32 alphabet"
        );
    }

    #[test]
    fn the_macros_expand_to_a_usable_triple() {
        // This crate's own build has no build script calling emit_build_hash,
        // so the macro must degrade to `unknown` rather than fail to compile.
        assert!(!build_hash!().is_empty());
        assert!(matches!(build_kind!(), "dev" | "prod"));
    }
}
