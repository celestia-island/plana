//! Family build identity — the single source of the `<base> <branch>:<hash7>`
//! version line (e.g. `0.1.0 master:d423747`).
//!
//! User direction 2026-10-08, base refined 2026-10-10, separator narrowed
//! 2026-10-11: the drifting numeric patch counter is retired — **the
//! branch and the exact commit ARE the identity** — while the base reports
//! the consuming crate's FULL package version (e.g. `0.1.0`), never a
//! truncated `major.minor`. Every health
//! surface and
//! About row across the family prints the same shape, and the trailing
//! hash7 is always the git short hash of the built source, so "does this
//! deployment match the branch HEAD" stays answerable at a glance. The
//! double-colon `<branch>::<hash7>` shape of the 2026-10-08..10 lines
//! stays parseable — devtools builds and binaries from the transition
//! window still speak it, and every parser here (and in the TS twin)
//! accepts both separators.
//!
//! This crate is the successor of the retired base32 build-token scheme
//! (SHA-256 over the revision, base32, last 6 characters — tokens like
//! `EDW62Q` that matched no commit anyone could check out). That scheme and
//! its `emit_build_hash` / `build_hash!` API are gone; the last consumer
//! (evernight) migrated with this crate's introduction.
//!
//! Resolution order (mirrors `celestia-devtools version-string`, the
//! build-host facility):
//! 1. `CELESTIA_BUILD_BRANCH` + `CELESTIA_BUILD_HASH` — the engine
//!    passthrough (2026-10-11 mislabel incident): a builder that checked
//!    out a known ref states it; honored only when the hash pins to this
//!    checkout's HEAD;
//! 2. `celestia-devtools version-string --dir <dir>` — the shared
//!    implementation, when the devtools binary is on the build PATH;
//! 3. direct git — branch via `rev-parse --abbrev-ref HEAD`, detached
//!    worktrees resolved through a ladder (exact default-branch tip →
//!    local heads → origin remote heads → symbolic remote HEAD →
//!    `"detached"`), hash via `rev-parse --short=7 HEAD`;
//! 4. without git metadata — the honest markers `detached` / `unknown`,
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
    /// The family version line: `<base> <branch>:<hash7>`.
    pub fn version_line(&self) -> String {
        format!("{} {}:{}", self.base, self.branch, self.hash)
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
    // The engine passthrough outranks everything (2026-10-11 mislabel
    // incident): a builder that checked out a known ref states it, so no
    // refstore re-derivation can relabel the artifact mid-build.
    if let Some(branch) = identity_from_env(
        env::var("CELESTIA_BUILD_BRANCH").ok(),
        env::var("CELESTIA_BUILD_HASH").ok(),
        git(dir, &["rev-parse", "HEAD"]),
    ) {
        let hash = short_hash(dir).unwrap_or_else(|| "unknown".to_string());
        return BuildIdentity {
            base: base.to_string(),
            branch,
            hash,
        };
    }
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

/// The engine passthrough (2026-10-11 mislabel incident): a builder that
/// KNOWS the ref it checked out — celestia-build, which synced `CB_REF` at
/// wave start — states it as `CELESTIA_BUILD_BRANCH` pinned to this
/// checkout by `CELESTIA_BUILD_HASH`. The pair is honored only together
/// and only when the hash equals the checkout's HEAD, so a stale or stray
/// branch-only env can never relabel a build; any mismatch falls through
/// to the normal resolution.
fn identity_from_env(
    branch: Option<String>,
    hash: Option<String>,
    head: Option<String>,
) -> Option<String> {
    let branch = branch.filter(|b| !b.is_empty())?;
    match hash.as_deref() {
        Some(h) if !h.is_empty() && head.as_deref() == Some(h) => Some(branch),
        _ => None,
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

/// Extract `<branch>:<hash7>` from a facility line, validating the hash
/// shape so a malformed line can never masquerade as an identity. The
/// legacy `<branch>::<hash7>` double-colon shape (devtools binaries and
/// lines from the pre-2026-10-11 transition window) stays parseable —
/// git refnames cannot contain `:`, so the LAST colon always splits
/// branch from hash.
fn parse_identity_token(line: &str) -> Option<(String, String)> {
    let token = line.split_whitespace().rev().find(|t| t.contains(':'))?;
    let (branch, hash) = token.rsplit_once(':')?;
    let branch = branch.trim_end_matches(':');
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
///
/// Before any of that: the default branch's EXACT tip wins (2026-10-11
/// mislabel incident). Deploy waves always build the default branch tip as
/// a detached checkout, while parallel lanes leave same-tip feature
/// branches in the shared refstore — `name-rev` then steals the identity
/// (`feat/model-usage-selfhosted:770f625` on a `master` build). When HEAD
/// IS the default tip, local or remote, the default name is the truth.
fn resolve_detached(dir: &str) -> Option<String> {
    if let Some(name) = default_branch_tip(dir) {
        return Some(name);
    }
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

/// The default branch's name per the remote's symbolic HEAD (e.g.
/// `origin/master` → `master`), when the clone records one. The remote
/// prefix is stripped at the FIRST slash so a slash-named default
/// (`origin/release/2026-10` → `release/2026-10`) keeps its full name —
/// taking the last component would probe a ref that never exists and
/// silently disarm the tip guard (R2 2026-10-11).
fn default_branch_name(dir: &str) -> Option<String> {
    let sym = git(
        dir,
        &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"],
    )?;
    let name = sym.split_once('/')?.1.to_string();
    if name.is_empty() || name == "HEAD" {
        return None;
    }
    Some(name)
}

/// `Some(default)` when HEAD is EXACTLY the default branch's tip, local
/// ref or remote ref — the shape every default-branch deploy wave builds.
/// A local master behind origin's (post-fetch, pre-ff) still matches the
/// remote ref; neither matching means HEAD is genuinely off-tip and the
/// ladder answers instead.
fn default_branch_tip(dir: &str) -> Option<String> {
    let name = default_branch_name(dir)?;
    let head = git(dir, &["rev-parse", "HEAD"])?;
    for r#ref in [
        format!("refs/heads/{name}"),
        format!("refs/remotes/origin/{name}"),
    ] {
        if git(dir, &["rev-parse", "--verify", &r#ref]).as_deref() == Some(head.as_str()) {
            return Some(name);
        }
    }
    None
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
    // The engine passthrough moves the identity between waves even on an
    // unchanged checkout — watch both halves of the pair (the branch alone
    // never relabels anything, but the hash pin flips the whole override).
    println!("cargo:rerun-if-env-changed=CELESTIA_BUILD_BRANCH");
    println!("cargo:rerun-if-env-changed=CELESTIA_BUILD_HASH");
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
        assert_eq!(identity.version_line(), "0.1.0 master:d423747");
    }

    #[test]
    fn devtools_line_parses_and_validates() {
        // The 2026-10-11 single-colon emit.
        assert_eq!(
            parse_identity_token("0.1.0 master:2e49260"),
            Some(("master".into(), "2e49260".into()))
        );
        assert_eq!(
            parse_identity_token("0.1.0 feat/x-y:badd902a"),
            Some(("feat/x-y".into(), "badd902a".into()))
        );
        // The legacy double-colon shape (pre-2026-10-11 devtools binaries
        // and binaries built in the transition window) stays parseable.
        assert_eq!(
            parse_identity_token("0.1 master::2e49260"),
            Some(("master".into(), "2e49260".into()))
        );
        assert_eq!(
            parse_identity_token("0.1 feat/x-y::badd902a"),
            Some(("feat/x-y".into(), "badd902a".into()))
        );
        // Malformed hashes and hashless lines never become identities.
        assert_eq!(parse_identity_token("0.1 master:zzz1234"), None);
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
        // The identity token carries `branch:hash` (either separator — the
        // devtools-first resolution may answer in the legacy shape on a
        // transition-window build host).
        let tail = line.split_whitespace().next_back().unwrap_or_default();
        let (_, hash) = tail
            .rsplit_once(':')
            .expect("line carries the branch:hash identity");
        assert!(
            is_short_hash(hash),
            "line {line} ends in a short commit hash"
        );
    }

    // ── engine passthrough (2026-10-11 mislabel incident) ──────────────

    #[test]
    fn env_passthrough_honors_only_a_hash_pinned_pair() {
        let head = "0123456789abcdef0123456789abcdef01234567";
        // The pair, pinned: the branch is stated verbatim.
        assert_eq!(
            identity_from_env(Some("master".into()), Some(head.into()), Some(head.into())),
            Some("master".into())
        );
        // Hash mismatch (stale env, different checkout): ignored.
        assert_eq!(
            identity_from_env(
                Some("master".into()),
                Some(head.into()),
                Some("fedcba9876543210fedcba9876543210fedcba98".into())
            ),
            None
        );
        // A branch alone can never relabel a build.
        assert_eq!(
            identity_from_env(Some("master".into()), None, Some(head.into())),
            None
        );
        // A hash alone names nothing.
        assert_eq!(
            identity_from_env(None, Some(head.into()), Some(head.into())),
            None
        );
        // Empty branch is no statement.
        assert_eq!(
            identity_from_env(Some(String::new()), Some(head.into()), Some(head.into())),
            None
        );
        // Empty hash pin is no pin.
        assert_eq!(
            identity_from_env(
                Some("master".into()),
                Some(String::new()),
                Some(head.into())
            ),
            None
        );
        // No git metadata at all: nothing to pin against.
        assert_eq!(
            identity_from_env(Some("master".into()), Some(head.into()), None),
            None
        );
    }

    // ── detached default-tip rule (2026-10-11 mislabel incident) ───────

    /// A throwaway git repo shaped like the mislabel incident: the default
    /// branch (`default_name`) carries a same-tip feature branch, and HEAD
    /// sits detached on that tip (the deploy wave's checkout shape). With
    /// `advance_master` the default branch moves one commit ahead first —
    /// the honest off-tip case where the ladder must keep answering.
    fn mislabel_fixture(default_name: &str, advance_master: bool) -> std::path::PathBuf {
        use std::fs;
        let dir = std::env::temp_dir().join(format!(
            "plana-build-info-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).expect("fixture dir");
        let git_args = |args: &[&str]| {
            let mut full = vec!["-C", dir.to_str().unwrap()];
            full.extend_from_slice(args);
            let out = Command::new("git").args(&full).output().expect("git runs");
            assert!(
                out.status.success(),
                "fixture git {args:?}: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        };
        git_args(&["init", "--quiet", "-b", "master"]);
        // Fixture commits bypass the org-wide core.hooksPath commit-msg
        // lint (house precedent: fixtures always run with hooksPath
        // cleared — the org hook would reject "tip"/"ahead" as subjects).
        git_args(&[
            "-c",
            "core.hooksPath=",
            "-c",
            "user.email=fixture@test",
            "-c",
            "user.name=fixture",
            "commit",
            "--quiet",
            "--allow-empty",
            "-m",
            "tip",
        ]);
        let tip = git_args(&["rev-parse", "HEAD"]);
        let remote_ref = format!("refs/remotes/origin/{default_name}");
        git_args(&["update-ref", &remote_ref, &tip]);
        git_args(&["symbolic-ref", "refs/remotes/origin/HEAD", &remote_ref]);
        git_args(&["branch", "feat/stale", &tip]);
        if advance_master {
            git_args(&[
                "-c",
                "core.hooksPath=",
                "-c",
                "user.email=fixture@test",
                "-c",
                "user.name=fixture",
                "commit",
                "--quiet",
                "--allow-empty",
                "-m",
                "ahead",
            ]);
            // The fetch has happened too: the remote-tracking ref moved with
            // the local default branch, so nothing default-shaped points at
            // the detached tip anymore.
            let ahead = git_args(&["rev-parse", "HEAD"]);
            git_args(&["update-ref", &remote_ref, &ahead]);
        }
        git_args(&["checkout", "--quiet", "--detach", &tip]);
        dir
    }

    #[test]
    fn detached_default_tip_wins_over_same_tip_feature_branch() {
        let dir = mislabel_fixture("master", false);
        // Incident shape: name-rev would answer feat/stale (and did, in
        // production) — the default tip rule must claim the identity.
        assert_eq!(
            resolve_detached(dir.to_str().unwrap()),
            Some("master".into())
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn detached_slash_named_default_tip_also_wins() {
        // R2 2026-10-11: a slash-named default (release/2026-10) must keep
        // its full name — last-component extraction would probe
        // refs/heads/2026-10, find nothing, and silently disarm the guard.
        let dir = mislabel_fixture("release/2026-10", false);
        assert_eq!(
            resolve_detached(dir.to_str().unwrap()),
            Some("release/2026-10".into())
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn detached_off_tip_keeps_the_ladder_answer() {
        let dir = mislabel_fixture("master", true);
        // master moved ahead; HEAD is genuinely only the feature branch's
        // tip — the ladder's honest answer stands.
        assert_eq!(
            resolve_detached(dir.to_str().unwrap()),
            Some("feat/stale".into())
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
