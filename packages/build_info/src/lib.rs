//! Family build identity — the single source of the `<base> <branch>::<hash7>`
//! version line (e.g. `0.1.0 master::d423747`).
//!
//! User direction 2026-10-08, base refined 2026-10-10: the drifting numeric
//! patch counter is retired — **the branch and the exact commit ARE the
//! identity** — while the base reports the consuming crate's FULL package
//! version (e.g. `0.1.0`), never a truncated `major.minor`. Every health
//! surface and
//! About row across the family prints the same shape, and the trailing
//! hash7 is always the git short hash of the built source, so "does this
//! deployment match the branch HEAD" stays answerable at a glance.
//!
//! This crate is the successor of the retired base32 build-token scheme
//! (SHA-256 over the revision, base32, last 6 characters — tokens like
//! `EDW62Q` that matched no commit anyone could check out). That scheme and
//! its `emit_build_hash` / `build_hash!` API are gone; the last consumer
//! (evernight) migrated with this crate's introduction.
//!
//! Resolution order (mirrors `celestia-devtools version-string`, the
//! build-host facility):
//! 1. `celestia-devtools version-string --dir <dir>` — the shared
//!    implementation, when the devtools binary is on the build PATH;
//! 2. direct git — branch via `rev-parse --abbrev-ref HEAD`, detached
//!    worktrees resolved through a ladder (local heads → origin remote
//!    heads → symbolic remote HEAD → `"detached"`), hash via
//!    `rev-parse --short=7 HEAD`;
//! 3. without git metadata — the honest markers `detached` / `unknown`,
//!    never a fabricated token.
//!
//! Usage in a consuming crate:
//!
//! ```ignore
//! // build.rs
//! fn main() {
//!     plana_build_info::emit_version_line(env!("CARGO_PKG_VERSION"));
//! }
//!
//! // anywhere in that crate
//! pub fn version_line() -> &'static str { env!("VERSION") }
//! pub fn version_hash() -> &'static str { env!("VERSION_HASH") }
//! ```
//!
//! The TS twin of this facility is exported from
//! `@celestia-island/plana-rpc-client/build-info` for build-time webui
//! tooling (a subpath export, so webview consumers of the main entry
//! never pull the node-only module into their bundles).

use std::env;
use std::path::Path;
use std::process::Command;

/// The resolved identity of one build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildIdentity {
    /// Version base — the consuming crate's full package version,
    /// e.g. `0.1.0`.
    pub base: String,
    /// Branch name, or `"detached"` when no name resolves.
    pub branch: String,
    /// Commit short hash (`rev-parse --short=7`), or `"unknown"` without git.
    pub hash: String,
}

impl BuildIdentity {
    /// The family version line: `<base> <branch>::<hash7>`.
    pub fn version_line(&self) -> String {
        format!("{} {}::{}", self.base, self.branch, self.hash)
    }
}

/// The identity base: the consuming crate's FULL package version.
///
/// The 2026-10-08 direction collapsed this to `major.minor` (`0.1.284`
/// and `0.1.7` both identifying as `0.1`); the 2026-10-10 user direction
/// reversed that — the base is the honest full version (e.g. `0.1.0`),
/// passed through verbatim.
pub fn base_from_pkg_version(pkg_version: &str) -> String {
    pkg_version.to_string()
}

/// Resolve the build identity of the git worktree containing `dir`.
///
/// See the crate docs for the resolution order and the fallback markers.
pub fn resolve(base: &str, dir: &str) -> BuildIdentity {
    if let Some(identity) = from_devtools(base, dir) {
        return identity;
    }
    let branch = branch_of(dir).unwrap_or_else(|| "detached".to_string());
    let hash = short_hash(dir).unwrap_or_else(|| "unknown".to_string());
    BuildIdentity {
        base: base.to_string(),
        branch,
        hash,
    }
}

/// Read the identity from the shared build-host facility, keeping the
/// caller's authoritative base.
fn from_devtools(base: &str, dir: &str) -> Option<BuildIdentity> {
    let output = Command::new("celestia-devtools")
        .args(["version-string", "--dir", dir])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let line = String::from_utf8_lossy(&output.stdout).trim().to_string();
    parse_identity_token(&line).map(|(branch, hash)| BuildIdentity {
        base: base.to_string(),
        branch,
        hash,
    })
}

/// Extract `<branch>::<hash7>` from a facility line, validating the hash
/// shape so a malformed line can never masquerade as an identity.
fn parse_identity_token(line: &str) -> Option<(String, String)> {
    let token = line.split_whitespace().rev().find(|t| t.contains("::"))?;
    let (branch, hash) = token.split_once("::")?;
    if branch.is_empty() || !is_short_hash(hash) {
        return None;
    }
    Some((branch.to_string(), hash.to_lowercase()))
}

/// Whether a string is a 7..=40 char lowercase hex commit hash.
fn is_short_hash(value: &str) -> bool {
    (7..=40).contains(&value.len())
        && value
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
}

/// Branch name of `dir`'s HEAD, resolving detached worktrees through the
/// ladder documented in the crate docs. `None` when git is unavailable.
fn branch_of(dir: &str) -> Option<String> {
    let branch = git(dir, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    if branch.is_empty() || branch == "HEAD" {
        return resolve_detached(dir);
    }
    Some(branch)
}

/// The detached ladder: local heads, then origin's remote heads (a freshly
/// fetched squash sha may live only on the remote), then the symbolic
/// remote HEAD. `name-rev` prints the literal `undefined` when its ref set
/// cannot name the commit — that must not leak into the identity.
fn resolve_detached(dir: &str) -> Option<String> {
    for refs in ["refs/heads/*", "refs/remotes/origin/*"] {
        if let Ok(out) = Command::new("git")
            .args([
                "-C",
                dir,
                "name-rev",
                "--name-only",
                &format!("--refs={refs}"),
                "HEAD",
            ])
            .output()
        {
            let named = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !named.is_empty() && named != "undefined" && named != "remotes/origin/HEAD" {
                // Git refnames cannot contain '~', so this drops any depth
                // suffix (`master~2`); remote names keep their last self.
                let stripped = named
                    .split('~')
                    .next()
                    .unwrap_or("detached")
                    .trim_start_matches("remotes/origin/");
                return Some(stripped.to_string());
            }
        }
    }
    if let Ok(out) = Command::new("git")
        .args([
            "-C",
            dir,
            "symbolic-ref",
            "--short",
            "refs/remotes/origin/HEAD",
        ])
        .output()
    {
        let sym = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !sym.is_empty() {
            return Some(sym.rsplit('/').next().unwrap_or("detached").to_string());
        }
    }
    Some("detached".to_string())
}

/// Short commit hash of `dir`'s HEAD, or `None` without git metadata.
fn short_hash(dir: &str) -> Option<String> {
    let hash = git(dir, &["rev-parse", "--short=7", "HEAD"])?;
    if !is_short_hash(&hash) {
        return None;
    }
    Some(hash)
}

fn git(dir: &str, args: &[&str]) -> Option<String> {
    let mut full = vec!["-C", dir];
    full.extend_from_slice(args);
    let output = Command::new("git").args(&full).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

/// Resolve and export the identity for the crate whose build script calls
/// this: sets `VERSION` (the full line) and `VERSION_HASH` (the bare
/// hash7), and registers the git state that may move the identity as rerun
/// triggers — a warm target directory must not ship a stale line.
///
/// `pkg_version` is the consuming crate's `CARGO_PKG_VERSION`; the base is
/// the full package version per the 2026-10-10 direction.
pub fn emit_version_line(pkg_version: &str) {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    emit_version_line_at(pkg_version, &manifest_dir);
}

/// Directory-explicit twin of [`emit_version_line`] (test seam).
pub fn emit_version_line_at(pkg_version: &str, dir: &str) {
    let identity = resolve(&base_from_pkg_version(pkg_version), dir);
    println!("cargo:rustc-env=VERSION={}", identity.version_line());
    // The bare hash7 for consumers that want the commit alone (probes,
    // machine facts) without re-parsing the version line.
    println!("cargo:rustc-env=VERSION_HASH={}", identity.hash);
    watch_git_state(dir);
    println!("cargo:rerun-if-changed=build.rs");
}

/// Watch the git state that can move the identity. Declaring ANY
/// rerun-if opts out of cargo's any-file default, so the watch must be
/// complete: `HEAD` (checkout switches), `refs/heads` (branch advances on
/// a worktree, where `HEAD` is a symref that keeps its shape), and
/// `packed-refs` (gc moves loose refs into the pack).
fn watch_git_state(dir: &str) {
    let Some(git_dir) = git(dir, &["rev-parse", "--absolute-git-dir"]) else {
        return;
    };
    for relative in ["HEAD", "refs/heads", "packed-refs"] {
        let path = Path::new(&git_dir).join(relative);
        println!("cargo:rerun-if-changed={}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_is_the_full_package_version() {
        // The 2026-10-10 direction: the base is the honest full version,
        // never a truncated major.minor.
        assert_eq!(base_from_pkg_version("0.1.284"), "0.1.284");
        assert_eq!(base_from_pkg_version("0.1.0"), "0.1.0");
        assert_eq!(base_from_pkg_version("2.7.9-rc.3"), "2.7.9-rc.3");
        assert_eq!(base_from_pkg_version("0.1"), "0.1");
        assert_eq!(base_from_pkg_version("weird"), "weird");
    }

    #[test]
    fn identity_line_is_the_family_shape() {
        let identity = BuildIdentity {
            base: "0.1.0".into(),
            branch: "master".into(),
            hash: "d423747".into(),
        };
        assert_eq!(identity.version_line(), "0.1.0 master::d423747");
    }

    #[test]
    fn devtools_line_parses_and_validates() {
        assert_eq!(
            parse_identity_token("0.1 master::2e49260"),
            Some(("master".into(), "2e49260".into()))
        );
        assert_eq!(
            parse_identity_token("0.1 feat/x-y::badd902a"),
            Some(("feat/x-y".into(), "badd902a".into()))
        );
        // Malformed hashes and hashless lines never become identities.
        assert_eq!(parse_identity_token("0.1 master::zzz1234"), None);
        assert_eq!(parse_identity_token("0.1.284"), None);
        assert_eq!(parse_identity_token(""), None);
    }

    #[test]
    fn short_hashes_are_seven_to_forty_hex() {
        assert!(is_short_hash("d423747"));
        assert!(is_short_hash("d4237476f9428618dc14d57de98fb58aa208db3f"));
        assert!(!is_short_hash("d42374"));
        assert!(!is_short_hash("d42374z"));
        assert!(
            !is_short_hash("EDW62Q"),
            "retired base32 tokens are not hashes"
        );
    }

    /// Inside a git checkout the resolver must produce a real identity —
    /// including in CI's detached checkouts, where the ladder answers.
    /// Without git metadata (vendored source) the honest markers appear.
    #[test]
    fn resolve_answers_for_this_checkout() {
        let identity = resolve("0.1.0", env!("CARGO_MANIFEST_DIR"));
        assert_eq!(identity.base, "0.1.0");
        assert!(
            !identity.branch.is_empty(),
            "branch resolves or degrades to detached"
        );
        if identity.hash != "unknown" {
            assert!(
                is_short_hash(&identity.hash),
                "hash {hash} must be a short commit hash",
                hash = identity.hash
            );
        }
        let line = identity.version_line();
        assert!(line.starts_with("0.1.0 "), "line {line} keeps the base");
        assert!(line.contains("::"), "line {line} carries the identity");
    }
}
