//! Named AppArmor profile for FUSE-capable containers (scepter / cosmos).
//!
//! Docker's default AppArmor profile (`docker-default`) denies FUSE mounts,
//! which `fuse-overlayfs` needs to isolate cosmos sub-container root
//! filesystems. Instead of disabling AppArmor entirely with
//! `apparmor=unconfined`, we apply the named [`FUSE_PROFILE_NAME`] profile,
//! which mirrors the docker-default baseline and additionally allows FUSE
//! mounts plus the pivot_root/overlay operations nested container
//! orchestration requires.
//!
//! ## Loading the profile (host, root)
//!
//! [`FUSE_PROFILE`] must be installed and loaded on the Docker host before
//! containers are created with it:
//!
//! ```text
//! # install
//! install -m 0644 celestia-plana-fuse /etc/apparmor.d/celestia-plana-fuse
//! # load into the running kernel
//! apparmor_parser -r /etc/apparmor.d/celestia-plana-fuse
//! ```
//!
//! If the profile is not loaded, Docker rejects container creation with a
//! clear error — this is the intended fail-closed path: the profile name is
//! set unconditionally (unless the dev escape hatch is enabled), so a missing
//! profile cannot silently fall back to `unconfined`.
//!
//! # DEV-ONLY escape hatch
//!
//! The `PLANA_APPARMOR_UNCONFINED` environment variable (see
//! [`is_apparmor_unconfined`]) falls back to `apparmor=unconfined` and MUST
//! NEVER be set in production.

use tracing::warn;

/// Name of the AppArmor profile that permits FUSE mounts (fuse-overlayfs).
pub const FUSE_PROFILE_NAME: &str = "celestia-plana-fuse";

/// Dev-mode escape hatch: set `PLANA_APPARMOR_UNCONFINED=1` to fall back to
/// `apparmor=unconfined`.
pub const UNCONFINED_ENV: &str = "PLANA_APPARMOR_UNCONFINED";

/// The AppArmor profile to load on the host via `apparmor_parser -r`.
pub const FUSE_PROFILE: &str = include_str!("apparmor/celestia-plana-fuse");

/// Returns true when the `PLANA_APPARMOR_UNCONFINED` escape hatch is enabled.
///
/// # DEV-ONLY escape hatch — never set in production
///
/// `PLANA_APPARMOR_UNCONFINED=true` (or `1`) makes [`fuse_security_opts`] emit
/// `apparmor=unconfined`, disabling the named FUSE AppArmor profile. This
/// mirrors the `DISABLE_SECCOMP` escape hatch in [`crate::seccomp`] and exists
/// only to unblock local development on hosts where the named profile has not
/// yet been installed/loaded. It MUST NEVER be set in production.
pub fn is_apparmor_unconfined() -> bool {
    let unconfined = unconfined_from_value(std::env::var(UNCONFINED_ENV).ok().as_deref());

    if unconfined {
        warn!(
            "PLANA_APPARMOR_UNCONFINED is set: the named AppArmor profile is DISABLED \
             and apparmor=unconfined is used instead. This is a DEV-ONLY escape hatch \
             and must never be enabled in production."
        );
    }

    unconfined
}

fn unconfined_from_value(value: Option<&str>) -> bool {
    matches!(value, Some(v) if v.eq_ignore_ascii_case("true") || v == "1")
}

/// Security opts for scepter/cosmos: the named FUSE AppArmor profile (or the
/// `unconfined` fallback) plus `no-new-privileges`.
pub fn fuse_security_opts() -> Vec<String> {
    fuse_security_opts_with_unconfined(is_apparmor_unconfined())
}

fn fuse_security_opts_with_unconfined(unconfined: bool) -> Vec<String> {
    let mut opts = vec!["no-new-privileges:true".to_string()];

    if unconfined {
        opts.push("apparmor=unconfined".to_string());
    } else {
        opts.push(format!("apparmor={}", FUSE_PROFILE_NAME));
    }

    opts
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Result, ensure};

    #[test]
    fn named_profile_security_opts() -> Result<()> {
        let opts = fuse_security_opts_with_unconfined(false);
        ensure!(
            opts.contains(&"no-new-privileges:true".to_string()),
            "must keep no-new-privileges: {:?}",
            opts
        );
        ensure!(
            opts.contains(&format!("apparmor={}", FUSE_PROFILE_NAME)),
            "must use the named profile by default: {:?}",
            opts
        );
        ensure!(
            !opts.iter().any(|o| o == "apparmor=unconfined"),
            "must not use apparmor=unconfined by default: {:?}",
            opts
        );
        Ok(())
    }

    #[test]
    fn unconfined_fallback_security_opts() -> Result<()> {
        let opts = fuse_security_opts_with_unconfined(true);
        ensure!(
            opts.contains(&"apparmor=unconfined".to_string()),
            "escape hatch must fall back to apparmor=unconfined: {:?}",
            opts
        );
        ensure!(
            opts.contains(&"no-new-privileges:true".to_string()),
            "no-new-privileges must remain even when unconfined: {:?}",
            opts
        );
        Ok(())
    }

    #[test]
    fn unconfined_parses_only_truthy_values() -> Result<()> {
        assert!(unconfined_from_value(Some("true")));
        assert!(unconfined_from_value(Some("TRUE")));
        assert!(unconfined_from_value(Some("1")));
        assert!(!unconfined_from_value(Some("false")));
        assert!(!unconfined_from_value(Some("0")));
        assert!(!unconfined_from_value(Some("")));
        assert!(!unconfined_from_value(None));
        Ok(())
    }

    #[test]
    fn profile_content_permits_fuse_and_pivot_root() -> Result<()> {
        ensure!(
            FUSE_PROFILE.contains("mount fstype=fuse.*"),
            "profile must allow FUSE mounts"
        );
        ensure!(
            FUSE_PROFILE.contains("pivot_root"),
            "profile must keep docker-default pivot_root allowance"
        );
        ensure!(
            FUSE_PROFILE.contains(&format!("profile {}", FUSE_PROFILE_NAME)),
            "profile block must be named after FUSE_PROFILE_NAME"
        );
        ensure!(
            FUSE_PROFILE.contains("#include <tunables/global>"),
            "profile must include tunables/global"
        );
        Ok(())
    }
}
