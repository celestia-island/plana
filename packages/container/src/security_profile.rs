use crate::{egress::EgressPolicy, seccomp};

pub struct ContainerSecurity {
    pub cap_drop: Option<Vec<String>>,
    pub cap_add: Option<Vec<String>>,
    pub security_opt: Option<Vec<String>>,
    pub egress_policy: Option<EgressPolicy>,
}

pub fn postgres() -> ContainerSecurity {
    ContainerSecurity {
        cap_drop: Some(vec![]),
        cap_add: None,
        security_opt: Some(vec!["no-new-privileges:true".to_string()]),
        egress_policy: Some(EgressPolicy::deny_all()),
    }
}

pub fn scepter() -> ContainerSecurity {
    ContainerSecurity {
        cap_drop: Some(vec!["ALL".to_string()]),
        cap_add: Some(vec![
            "NET_BIND_SERVICE".to_string(),
            "SYS_ADMIN".to_string(),
        ]),
        // AppArmor blocks FUSE mounts (fuse-overlayfs) inside the container.
        // Without this, cosmos sub-container rootfs isolation fails.
        security_opt: Some(vec!["apparmor=unconfined".to_string()]),
        egress_policy: Some(EgressPolicy::entelecheia_default()),
    }
}

pub fn scepter_readonly() -> ContainerSecurity {
    ContainerSecurity {
        cap_drop: Some(vec!["ALL".to_string()]),
        cap_add: Some(vec!["NET_BIND_SERVICE".to_string()]),
        security_opt: None,
        egress_policy: Some(EgressPolicy::entelecheia_default()),
    }
}

pub fn cosmos() -> ContainerSecurity {
    ContainerSecurity {
        // Drop ALL then add back the minimal set cosmos sub-containers need:
        // - NET_BIND_SERVICE: bind to ports < 1024 (if scepter binds low)
        // - SYS_ADMIN: needed if sub-container probes fuse/overlay at startup
        // - CHOWN / FOWNER / SETGID / SETUID: file ownership on shared volumes
        cap_drop: Some(vec!["ALL".to_string()]),
        cap_add: Some(vec![
            "NET_BIND_SERVICE".to_string(),
            "SYS_ADMIN".to_string(),
            "CHOWN".to_string(),
            "FOWNER".to_string(),
            "SETGID".to_string(),
            "SETUID".to_string(),
            "DAC_OVERRIDE".to_string(),
        ]),
        // Relax seccomp — the EntelecheiaDefault whitelist blocks `mount` which
        // scepter may probe during startup. AppArmor=unconfined matches the
        // scepter orchestrator's own profile.
        security_opt: Some(vec!["apparmor=unconfined".to_string()]),
        egress_policy: Some(EgressPolicy::entelecheia_default()),
    }
}

pub fn compile() -> ContainerSecurity {
    ContainerSecurity {
        cap_drop: None,
        cap_add: None,
        security_opt: Some(seccomp::build_security_opts(Some(
            seccomp::SeccompProfile::Compile,
        ))),
        egress_policy: None,
    }
}

pub fn apply_to_host_config(
    sec: &ContainerSecurity,
    host_config: &mut bollard::service::HostConfig,
) {
    let default_egress = EgressPolicy::entelecheia_default();
    let egress = sec.egress_policy.as_ref().unwrap_or(&default_egress);
    egress.apply_to_docker_config(host_config);

    if let Some(ref dns) = host_config.dns
        && dns.is_empty()
    {
        host_config.dns = None;
    }

    host_config.cap_drop = Some(
        sec.cap_drop
            .clone()
            .unwrap_or_else(|| vec!["ALL".to_string()]),
    );
    host_config.cap_add = sec.cap_add.clone();
    host_config.security_opt = Some(
        sec.security_opt
            .clone()
            .unwrap_or_else(|| seccomp::build_security_opts(None)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result, ensure};

    #[test]
    fn postgres_no_cap_drop() -> Result<()> {
        let sec = postgres();
        assert_eq!(sec.cap_drop, Some(vec![]));
        assert!(
            sec.egress_policy
                .as_ref()
                .context("egress policy expected")?
                .is_restricted()
        );
        Ok(())
    }

    #[test]
    fn scepter_restricts_caps_and_egress() -> Result<()> {
        let sec = scepter();
        assert_eq!(sec.cap_drop, Some(vec!["ALL".to_string()]));
        let added = sec.cap_add.as_ref().context("cap_add expected")?;
        assert!(
            added.contains(&"SYS_ADMIN".to_string()),
            "must have SYS_ADMIN for container orchestration"
        );
        assert!(added.contains(&"NET_BIND_SERVICE".to_string()));
        assert!(!added.contains(&"ALL".to_string()), "must not wildcard-add");
        // scepter uses apparmor=unconfined (needed for fuse-overlayfs)
        assert!(sec.security_opt.is_some());
        assert!(
            sec.egress_policy
                .as_ref()
                .context("egress policy expected")?
                .is_restricted()
        );
        Ok(())
    }

    #[test]
    fn cosmos_uses_functional_profile() -> Result<()> {
        let sec = cosmos();
        // Must explicitly drop ALL then add back needed caps
        assert_eq!(sec.cap_drop, Some(vec!["ALL".to_string()]));
        let added = sec.cap_add.as_ref().context("cap_add expected")?;
        assert!(
            added.contains(&"SYS_ADMIN".to_string()),
            "cosmos needs SYS_ADMIN"
        );
        assert!(added.contains(&"NET_BIND_SERVICE".to_string()));
        // Must have explicit security_opt (not None → strict seccomp)
        assert!(sec.security_opt.is_some());
        // Must have egress filtering
        assert!(
            sec.egress_policy
                .as_ref()
                .context("egress policy expected")?
                .is_restricted()
        );
        Ok(())
    }

    #[test]
    fn compile_has_relaxed_seccomp() -> Result<()> {
        let sec = compile();
        assert_eq!(sec.cap_drop, None);
        assert!(sec.security_opt.is_some());
        assert!(sec.egress_policy.is_none());
        Ok(())
    }

    #[test]
    fn apply_postgres_to_host_config() -> Result<()> {
        let sec = postgres();
        let mut hc = bollard::service::HostConfig::default();
        apply_to_host_config(&sec, &mut hc);
        assert_eq!(hc.cap_drop, Some(vec![]));
        assert!(hc.security_opt.is_some());
        assert_eq!(hc.dns, Some(vec!["0.0.0.0".to_string()]));
        Ok(())
    }

    #[test]
    fn apply_scepter_to_host_config() -> Result<()> {
        let sec = scepter();
        let mut hc = bollard::service::HostConfig::default();
        apply_to_host_config(&sec, &mut hc);
        assert_eq!(hc.cap_drop, Some(vec!["ALL".to_string()]));
        let added = hc.cap_add.as_ref().context("cap_add expected")?;
        assert!(
            added.contains(&"SYS_ADMIN".to_string()),
            "scepter needs SYS_ADMIN"
        );
        let sec_opt = hc.security_opt.context("security_opt expected")?;
        assert!(
            sec_opt.iter().any(|o| o == "apparmor=unconfined"),
            "scepter uses apparmor=unconfined for fuse-overlayfs"
        );
        assert!(hc.dns.is_some());
        Ok(())
    }

    #[test]
    fn apply_cosmos_to_host_config() -> Result<()> {
        let sec = cosmos();
        let mut hc = bollard::service::HostConfig::default();
        apply_to_host_config(&sec, &mut hc);
        assert_eq!(hc.cap_drop, Some(vec!["ALL".to_string()]));
        let added = hc.cap_add.as_ref().context("cap_add expected")?;
        assert!(added.contains(&"SYS_ADMIN".to_string()));
        let sec_opt = hc.security_opt.context("security_opt expected")?;
        assert!(sec_opt.iter().any(|o| o == "apparmor=unconfined"));
        assert!(hc.dns.is_some());
        Ok(())
    }

    #[test]
    fn scepter_drops_all_then_adds_orchestrator_caps() -> Result<()> {
        let sec = scepter();
        ensure!(
            sec.cap_drop == Some(vec!["ALL".to_string()]),
            "Scepter must drop ALL capabilities first"
        );
        let added = sec.cap_add.as_ref().context("cap_add expected")?;
        ensure!(
            added.contains(&"SYS_ADMIN".to_string()),
            "Scepter must have SYS_ADMIN for fuse-overlayfs fallback"
        );
        ensure!(
            !added.iter().any(|c| c == "ALL" || c == "*" || c.is_empty()),
            "Scepter must not wildcard-add capabilities"
        );
        Ok(())
    }

    #[test]
    fn scepter_readonly_has_no_sys_admin() -> Result<()> {
        let sec = scepter_readonly();
        if let Some(ref caps) = sec.cap_add {
            ensure!(
                !caps.iter().any(|c| c == "SYS_ADMIN"),
                "Scepter read-only profile must never have SYS_ADMIN"
            );
        }
        ensure!(
            sec.cap_drop == Some(vec!["ALL".to_string()]),
            "Scepter read-only must drop ALL capabilities first"
        );
        Ok(())
    }

    #[test]
    fn scepter_has_restricted_egress() -> Result<()> {
        let sec = scepter();
        ensure!(
            sec.egress_policy
                .as_ref()
                .context("egress policy expected")?
                .is_restricted(),
            "Scepter must use restricted egress policy"
        );
        Ok(())
    }

    #[test]
    fn all_profiles_drop_or_override_caps() -> Result<()> {
        // postgres and scepter_readonly must drop caps and never add SYS_ADMIN
        for (name, sec) in [
            ("postgres", postgres()),
            ("scepter_readonly", scepter_readonly()),
        ] {
            let mut hc = bollard::service::HostConfig::default();
            apply_to_host_config(&sec, &mut hc);
            ensure!(
                hc.cap_drop.is_some(),
                "{} profile must explicitly drop capabilities",
                name
            );
            if let Some(ref added) = hc.cap_add {
                ensure!(
                    !added.iter().any(|c| c == "SYS_ADMIN"),
                    "{} profile must never add SYS_ADMIN",
                    name
                );
            }
        }
        Ok(())
    }

    #[test]
    fn all_profiles_have_security_opts() -> Result<()> {
        // All profiles must produce some security_opt after apply_to_host_config
        for (name, sec) in [
            ("scepter", scepter()),
            ("cosmos", cosmos()),
            ("compile", compile()),
        ] {
            let mut hc = bollard::service::HostConfig::default();
            apply_to_host_config(&sec, &mut hc);
            let sec_opt = hc.security_opt.unwrap_or_default();
            ensure!(
                !sec_opt.is_empty(),
                "{} profile must produce some security_opt",
                name
            );
        }
        Ok(())
    }

    #[test]
    fn postgres_has_no_new_privileges() -> Result<()> {
        let sec = postgres();
        let mut hc = bollard::service::HostConfig::default();
        apply_to_host_config(&sec, &mut hc);
        let sec_opt = hc.security_opt.unwrap_or_default();
        ensure!(
            sec_opt.iter().any(|o| o.contains("no-new-privileges")),
            "postgres profile must have no-new-privileges"
        );
        Ok(())
    }
}
