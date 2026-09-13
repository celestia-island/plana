//! Build identity shared by every plana-consuming backend.
//!
//! The About dialog of the panel shows one row per backend with the same
//! three facts: service version, build hash and build kind. Version and kind
//! are free (`CARGO_PKG_VERSION`, the build profile); the **hash** is the
//! only piece that needs a build script, and every service was inventing its
//! own — one repo emitted a hash of the build *timestamp* (useless as
//! identity), two emitted nothing at all and reported `null`.
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

/// Read the git revision of the current checkout and export it as the
/// `BUILD_HASH` compile-time environment variable.
///
/// Call from `build.rs`. Emits `unknown` when the sources have no git
/// metadata (vendored build, source tarball) so the field is always present
/// and honest, and suffixes `-dirty` when the tree has uncommitted changes.
/// Also registers the git HEAD as a rerun trigger, so checking out another
/// commit rebuilds the crate and the reported hash cannot go stale.
pub fn emit_build_hash() {
    if let Some(head) = git_path("HEAD") {
        println!("cargo:rerun-if-changed={head}");
    }
    let hash = git_revision().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_HASH={hash}");
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

/// Short git revision of the working tree, `-dirty` when it has uncommitted
/// changes. `None` when this is not a git checkout or git is unavailable.
fn git_revision() -> Option<String> {
    let rev = git(&["rev-parse", "--short=7", "HEAD"])?;
    let dirty = !git(&["status", "--porcelain"])
        .unwrap_or_default()
        .is_empty();
    normalize_revision(&rev, dirty)
}

/// Trim raw `git rev-parse` output into a short revision and append the dirty
/// marker. Kept separate from the process call so the shape is testable
/// without a repository.
fn normalize_revision(raw: &str, dirty: bool) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let short: String = trimmed.chars().take(7).collect();
    Some(if dirty {
        format!("{short}-dirty")
    } else {
        short
    })
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

    #[test]
    fn keeps_seven_characters_of_the_revision() {
        assert_eq!(
            normalize_revision("a8772b9d1f2e3c4b5a6978\n", false).as_deref(),
            Some("a8772b9")
        );
    }

    #[test]
    fn shorter_revisions_pass_through() {
        assert_eq!(
            normalize_revision("abc123\n", false).as_deref(),
            Some("abc123")
        );
    }

    #[test]
    fn marks_uncommitted_trees() {
        assert_eq!(
            normalize_revision("a8772b9d1f\n", true).as_deref(),
            Some("a8772b9-dirty")
        );
    }

    #[test]
    fn empty_output_is_not_a_revision() {
        assert_eq!(normalize_revision("  \n", false), None);
        assert_eq!(normalize_revision("", true), None);
    }

    #[test]
    fn the_macros_expand_to_a_usable_triple() {
        // This crate's own build has no build script calling emit_build_hash,
        // so the macro must degrade to `unknown` rather than fail to compile.
        assert!(!build_hash!().is_empty());
        assert!(matches!(build_kind!(), "dev" | "prod"));
    }
}
