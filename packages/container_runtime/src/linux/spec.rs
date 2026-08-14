use std::{collections::HashSet, path::Path};

use oci_spec::runtime::{
    Arch, Capability, LinuxBuilder, LinuxCapabilities, LinuxCapabilitiesBuilder, LinuxCpuBuilder,
    LinuxDeviceCgroup, LinuxDeviceCgroupBuilder, LinuxDeviceType, LinuxMemoryBuilder,
    LinuxNamespace, LinuxNamespaceBuilder, LinuxPidsBuilder, LinuxResources, LinuxResourcesBuilder,
    LinuxSeccomp, LinuxSeccompAction, LinuxSeccompBuilder, LinuxSyscall, LinuxSyscallBuilder,
    Mount, MountBuilder, PosixRlimit, PosixRlimitBuilder, PosixRlimitType, ProcessBuilder,
    RootBuilder, Spec, SpecBuilder, UserBuilder,
};
use tracing::{info, warn};

use _container::{
    errors::{ContainerError, ContainerResult},
    seccomp::{SeccompProfile, SeccompProfileData},
    types::ContainerCreateParams,
};

const DEFAULT_MEMORY_LIMIT: i64 = 512 * 1024 * 1024;
const DEFAULT_CPU_SHARES: u64 = 1024;
const DEFAULT_PIDS_LIMIT: i64 = 100;
const DEFAULT_UID: u32 = 0;
const DEFAULT_GID: u32 = 0;

macro_rules! oci_bail {
    ($builder:expr, $ctx:expr, $cid:expr) => {
        $builder
            .build()
            .map_err(|e| ContainerError::OperationFailed {
                container_id: ($cid).to_string(),
                message: format!("{}: {}", $ctx, e),
            })?
    };
}

pub fn generate_oci_spec(
    params: &ContainerCreateParams,
    rootfs_path: &Path,
    container_id: &str,
    run_dir: &Path,
) -> ContainerResult<Spec> {
    let env_vec: Vec<String> = params
        .env
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();

    let args = vec!["/bin/sh".to_string()];

    let mut default_env = vec![
        "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string(),
        "TERM=xterm".to_string(),
        "HOME=/root".to_string(),
        "HOSTNAME=".to_string() + container_id,
    ];
    default_env.extend(env_vec);

    let user = oci_bail!(
        UserBuilder::default()
            .uid(
                params
                    .user
                    .as_ref()
                    .and_then(|u| u.parse().ok())
                    .unwrap_or(DEFAULT_UID),
            )
            .gid(
                params
                    .user
                    .as_ref()
                    .and_then(|u| {
                        let parts: Vec<&str> = u.split(':').collect();
                        if parts.len() > 1 {
                            parts[1].parse().ok()
                        } else {
                            None
                        }
                    })
                    .unwrap_or(DEFAULT_GID),
            ),
        "failed to build process user",
        container_id
    );

    let read_only = params.read_only_rootfs.unwrap_or(false);

    let process = oci_bail!(
        ProcessBuilder::default()
            .args(args)
            .env(default_env)
            .cwd(
                params
                    .working_dir
                    .clone()
                    .unwrap_or_else(|| "/home".to_string()),
            )
            .user(user)
            .no_new_privileges(true)
            .capabilities(build_capabilities(
                params.cap_drop.as_deref(),
                params.cap_add.as_deref(),
                !params.devices.is_empty(),
                container_id
            )?)
            .rlimits(build_rlimits(container_id)?),
        "failed to build process",
        container_id
    );

    let _namespaces = build_namespaces(container_id)?;

    let namespaces = build_namespaces(container_id)?;

    let resources = build_resources(
        params.memory_limit,
        params.nano_cpus,
        params.pids_limit,
        container_id,
    )?;

    let is_rootless = !nix::unistd::geteuid().is_root();

    let mut linux_builder = LinuxBuilder::default().namespaces(namespaces);

    // Cgroup management only when running as root. In rootless mode we skip
    // cgroup entirely (standard rootless-container practice, same as rootless
    // podman): a non-root process in /user.slice/... cannot write to
    // /sys/fs/cgroup/entelecheia/... (cgroup v2 cross-hierarchy restriction),
    // so setting cgroups_path or resources there just causes EACCES. The
    // container still runs with full namespace isolation — just without
    // resource limits.
    if !is_rootless {
        linux_builder = linux_builder
            .resources(resources)
            .cgroups_path(format!("/entelecheia/{}", container_id));
    }

    if is_rootless {
        let uid = nix::unistd::geteuid().as_raw() as u32;
        let gid = nix::unistd::getegid().as_raw() as u32;
        let uid_mappings: Vec<oci_spec::runtime::LinuxIdMapping> = serde_json::from_str(&format!(
            r#"[{{"containerID":0,"hostID":{},"size":1}}]"#,
            uid
        ))
        .unwrap_or_default();
        let gid_mappings: Vec<oci_spec::runtime::LinuxIdMapping> = serde_json::from_str(&format!(
            r#"[{{"containerID":0,"hostID":{},"size":1}}]"#,
            gid
        ))
        .unwrap_or_default();
        linux_builder = linux_builder
            .uid_mappings(uid_mappings)
            .gid_mappings(gid_mappings);
    }

    if let Some(seccomp) = build_seccomp(params, container_id)? {
        linux_builder = linux_builder.seccomp(seccomp);
    }

    let linux = oci_bail!(linux_builder, "failed to build linux config", container_id);

    let root = oci_bail!(
        RootBuilder::default()
            .path(rootfs_path.to_path_buf())
            .readonly(read_only),
        "failed to build root",
        container_id
    );

    let mounts = build_mounts(&params.volumes, run_dir, container_id)?;

    Ok(oci_bail!(
        SpecBuilder::default()
            .version("1.2.0")
            .process(process)
            .linux(linux)
            .root(root)
            .mounts(mounts)
            .hostname(""),
        "failed to build OCI spec",
        container_id
    ))
}

fn build_capabilities(
    cap_drop: Option<&[String]>,
    cap_add: Option<&[String]>,
    devices_requested: bool,
    container_id: &str,
) -> ContainerResult<LinuxCapabilities> {
    // Default container capability set. `MKNOD` is deliberately absent: it lets
    // a container create device nodes (/dev/mem, /dev/sda, ...) and, combined
    // with the absence of a device allowlist, would grant unmediated device
    // access. It is re-granted only when the caller explicitly requests devices
    // (via a non-empty `devices` passthrough list or an explicit `cap_add`).
    let mut caps: HashSet<Capability> = [
        Capability::AuditWrite,
        Capability::Kill,
        Capability::NetBindService,
        Capability::Setuid,
        Capability::Setgid,
        Capability::Chown,
        Capability::Fowner,
        Capability::DacOverride,
    ]
    .into_iter()
    .collect();

    // Re-grant MKNOD before the drop loop when the caller requests device
    // passthrough; an explicit `cap_drop` entry (including `MKNOD` or `ALL`)
    // can still override this.
    if devices_requested {
        caps.insert(Capability::Mknod);
    }

    // Apply drops. `ALL` clears the entire set; otherwise each named
    // capability is removed (unknown names are ignored).
    if let Some(dropped) = cap_drop {
        for d in dropped {
            let upper = d.to_uppercase();
            let c = upper.trim_start_matches("CAP_");
            if c == "ALL" {
                caps.clear();
            } else if let Some(cap) = parse_capability(c) {
                caps.remove(&cap);
            }
        }
    }

    // Re-add explicitly requested capabilities after the drop so re-grants
    // (e.g. `cap_drop: ["ALL"]` + `cap_add: ["SYS_ADMIN"]`) take effect.
    // Unknown names are ignored with a warning.
    if let Some(added) = cap_add {
        for a in added {
            let upper = a.to_uppercase();
            let c = upper.trim_start_matches("CAP_");
            match parse_capability(c) {
                Some(cap) => {
                    caps.insert(cap);
                }
                None => {
                    warn!(
                        container_id,
                        capability = c,
                        "ignoring unknown capability in cap_add"
                    );
                }
            }
        }
    }

    Ok(oci_bail!(
        LinuxCapabilitiesBuilder::default()
            .bounding(caps.clone())
            .effective(caps.clone())
            .inheritable(caps.clone())
            .permitted(caps.clone())
            .ambient(caps),
        "failed to build capabilities",
        container_id
    ))
}

fn parse_capability(name: &str) -> Option<Capability> {
    match name {
        "AUDIT_CONTROL" => Some(Capability::AuditControl),
        "AUDIT_READ" => Some(Capability::AuditRead),
        "AUDIT_WRITE" => Some(Capability::AuditWrite),
        "BLOCK_SUSPEND" => Some(Capability::BlockSuspend),
        "BPF" => Some(Capability::Bpf),
        "CHECKPOINT_RESTORE" => Some(Capability::CheckpointRestore),
        "CHOWN" => Some(Capability::Chown),
        "DAC_OVERRIDE" => Some(Capability::DacOverride),
        "DAC_READ_SEARCH" => Some(Capability::DacReadSearch),
        "FOWNER" => Some(Capability::Fowner),
        "FSETID" => Some(Capability::Fsetid),
        "IPC_LOCK" => Some(Capability::IpcLock),
        "IPC_OWNER" => Some(Capability::IpcOwner),
        "KILL" => Some(Capability::Kill),
        "LEASE" => Some(Capability::Lease),
        "LINUX_IMMUTABLE" => Some(Capability::LinuxImmutable),
        "MAC_ADMIN" => Some(Capability::MacAdmin),
        "MAC_OVERRIDE" => Some(Capability::MacOverride),
        "MKNOD" => Some(Capability::Mknod),
        "NET_ADMIN" => Some(Capability::NetAdmin),
        "NET_BIND_SERVICE" => Some(Capability::NetBindService),
        "NET_BROADCAST" => Some(Capability::NetBroadcast),
        "NET_RAW" => Some(Capability::NetRaw),
        "PERFMON" => Some(Capability::Perfmon),
        "SETGID" => Some(Capability::Setgid),
        "SETFCAP" => Some(Capability::Setfcap),
        "SETPCAP" => Some(Capability::Setpcap),
        "SETUID" => Some(Capability::Setuid),
        "SYS_ADMIN" => Some(Capability::SysAdmin),
        "SYS_BOOT" => Some(Capability::SysBoot),
        "SYS_CHROOT" => Some(Capability::SysChroot),
        "SYS_MODULE" => Some(Capability::SysModule),
        "SYS_NICE" => Some(Capability::SysNice),
        "SYS_PACCT" => Some(Capability::SysPacct),
        "SYS_PTRACE" => Some(Capability::SysPtrace),
        "SYS_RAWIO" => Some(Capability::SysRawio),
        "SYS_RESOURCE" => Some(Capability::SysResource),
        "SYS_TIME" => Some(Capability::SysTime),
        "SYS_TTY_CONFIG" => Some(Capability::SysTtyConfig),
        "SYSLOG" => Some(Capability::Syslog),
        "WAKE_ALARM" => Some(Capability::WakeAlarm),
        _ => None,
    }
}

fn build_rlimits(container_id: &str) -> ContainerResult<Vec<PosixRlimit>> {
    Ok(vec![
        oci_bail!(
            PosixRlimitBuilder::default()
                .typ(PosixRlimitType::RlimitNofile)
                .hard(1024u64)
                .soft(1024u64),
            "failed to build nofile rlimit",
            container_id
        ),
        oci_bail!(
            PosixRlimitBuilder::default()
                .typ(PosixRlimitType::RlimitSigpending)
                .hard(512u64)
                .soft(512u64),
            "failed to build sigpending rlimit",
            container_id
        ),
    ])
}

fn build_namespaces(container_id: &str) -> ContainerResult<Vec<LinuxNamespace>> {
    let is_rootless = !nix::unistd::geteuid().is_root();

    let mut ns = vec![
        oci_bail!(
            LinuxNamespaceBuilder::default().typ(oci_spec::runtime::LinuxNamespaceType::Pid),
            "failed to build pid namespace",
            container_id
        ),
        oci_bail!(
            LinuxNamespaceBuilder::default().typ(oci_spec::runtime::LinuxNamespaceType::Mount),
            "failed to build mount namespace",
            container_id
        ),
        oci_bail!(
            LinuxNamespaceBuilder::default().typ(oci_spec::runtime::LinuxNamespaceType::Ipc),
            "failed to build ipc namespace",
            container_id
        ),
        oci_bail!(
            LinuxNamespaceBuilder::default().typ(oci_spec::runtime::LinuxNamespaceType::Uts),
            "failed to build uts namespace",
            container_id
        ),
    ];

    if is_rootless {
        info!(
            container_id,
            uid = ?nix::unistd::geteuid(),
            "running rootless — adding user namespace"
        );
        ns.push(oci_bail!(
            LinuxNamespaceBuilder::default().typ(oci_spec::runtime::LinuxNamespaceType::User),
            "failed to build user namespace",
            container_id
        ));
    }

    ns.push(oci_bail!(
        LinuxNamespaceBuilder::default().typ(oci_spec::runtime::LinuxNamespaceType::Network),
        "failed to build network namespace",
        container_id
    ));

    Ok(ns)
}

fn build_resources(
    memory_limit: Option<i64>,
    nano_cpus: Option<i64>,
    pids_limit: Option<i64>,
    container_id: &str,
) -> ContainerResult<LinuxResources> {
    let memory = oci_bail!(
        LinuxMemoryBuilder::default().limit(memory_limit.unwrap_or(DEFAULT_MEMORY_LIMIT)),
        "failed to build memory limits",
        container_id
    );

    let cpu_shares = nano_cpus
        .map(|n| (n as u64 / 1_000_000).max(1) * 1024)
        .unwrap_or(DEFAULT_CPU_SHARES);

    let cpu = oci_bail!(
        LinuxCpuBuilder::default().shares(cpu_shares),
        "failed to build cpu limits",
        container_id
    );

    let pids = oci_bail!(
        LinuxPidsBuilder::default().limit(pids_limit.unwrap_or(DEFAULT_PIDS_LIMIT)),
        "failed to build pids limits",
        container_id
    );

    // Default-deny device cgroup: deny all device access by default so a
    // container cannot create (`mknod`) or read/write host device nodes.
    let devices = vec![build_device_cgroup(container_id)?];

    Ok(oci_bail!(
        LinuxResourcesBuilder::default()
            .devices(devices)
            .memory(memory)
            .cpu(cpu)
            .pids(pids),
        "failed to build linux resources",
        container_id
    ))
}

/// Build a default-deny device cgroup rule (equivalent to `a *:* rwm` written
/// to the cgroup v1 `devices.deny` file).
///
/// Combined with dropping `CAP_MKNOD` from the default capability set, this
/// blocks both creating and accessing host device nodes. On cgroup v1 the rule
/// is applied directly by libcgroups. On cgroup v2 the OCI `devices` list maps
/// to the eBPF device controller, which libcgroups only applies when
/// libcontainer's `cgroupsv2_devices` feature is enabled (not enabled here to
/// avoid pulling in a libbpf build dependency) — that residual v2 gap is
/// covered by the dropped `CAP_MKNOD` and documented in the PR notes.
fn build_device_cgroup(container_id: &str) -> ContainerResult<LinuxDeviceCgroup> {
    Ok(oci_bail!(
        LinuxDeviceCgroupBuilder::default()
            .allow(false)
            .typ(LinuxDeviceType::A)
            .access("rwm"),
        "failed to build default-deny device cgroup",
        container_id
    ))
}

fn build_mounts(
    volumes: &[_container::types::VolumeMount],
    run_dir: &Path,
    container_id: &str,
) -> ContainerResult<Vec<Mount>> {
    let vol_dir = run_dir.join("volumes");
    let mut mounts: Vec<Mount> = vec![
        oci_bail!(
            MountBuilder::default()
                .destination(std::path::Path::new("/proc").to_path_buf())
                .typ("proc")
                .source(std::path::Path::new("proc").to_path_buf()),
            "failed to build proc mount",
            container_id
        ),
        oci_bail!(
            MountBuilder::default()
                .destination(std::path::Path::new("/dev").to_path_buf())
                .typ("tmpfs")
                .source(std::path::Path::new("tmpfs").to_path_buf())
                .options(vec![
                    "nosuid".to_string(),
                    "strictatime".to_string(),
                    "mode=755".to_string(),
                    "size=65536k".to_string(),
                ]),
            "failed to build dev mount",
            container_id
        ),
        oci_bail!(
            MountBuilder::default()
                .destination(std::path::Path::new("/dev/pts").to_path_buf())
                .typ("devpts")
                .source(std::path::Path::new("devpts").to_path_buf())
                .options(vec![
                    "nosuid".to_string(),
                    "noexec".to_string(),
                    "newinstance".to_string(),
                    "ptmxmode=0666".to_string(),
                    "mode=0620".to_string(),
                ]),
            "failed to build devpts mount",
            container_id
        ),
        oci_bail!(
            MountBuilder::default()
                .destination(std::path::Path::new("/dev/shm").to_path_buf())
                .typ("tmpfs")
                .source(std::path::Path::new("shm").to_path_buf())
                .options(vec![
                    "nosuid".to_string(),
                    "noexec".to_string(),
                    "nodev".to_string(),
                    "mode=1777".to_string(),
                    "size=65536k".to_string(),
                ]),
            "failed to build shm mount",
            container_id
        ),
        oci_bail!(
            MountBuilder::default()
                .destination(std::path::Path::new("/dev/mqueue").to_path_buf())
                .typ("mqueue")
                .source(std::path::Path::new("mqueue").to_path_buf())
                .options(vec![
                    "nosuid".to_string(),
                    "noexec".to_string(),
                    "nodev".to_string(),
                ]),
            "failed to build mqueue mount",
            container_id
        ),
        oci_bail!(
            MountBuilder::default()
                .destination(std::path::Path::new("/sys").to_path_buf())
                .typ("sysfs")
                .source(std::path::Path::new("sysfs").to_path_buf())
                .options(vec![
                    "nosuid".to_string(),
                    "noexec".to_string(),
                    "nodev".to_string(),
                    "ro".to_string(),
                ]),
            "failed to build sysfs mount",
            container_id
        ),
        oci_bail!(
            MountBuilder::default()
                .destination(std::path::Path::new("/tmp").to_path_buf())
                .typ("tmpfs")
                .source(std::path::Path::new("tmpfs").to_path_buf())
                .options(vec![
                    "nosuid".to_string(),
                    "nodev".to_string(),
                    "mode=1777".to_string(),
                    "size=8388608k".to_string(),
                ]),
            "failed to build tmp mount",
            container_id
        ),
    ];

    if vol_dir.exists() {
        mounts.push(oci_bail!(
            MountBuilder::default()
                .destination(std::path::Path::new("/mnt/volumes").to_path_buf())
                .typ("bind")
                .source(vol_dir)
                .options(vec!["rbind".to_string(), "rw".to_string()]),
            "failed to build volumes mount",
            container_id
        ));
    }

    for vol in volumes {
        let mut options = vec!["rbind".to_string()];
        if vol.read_only {
            options.push("ro".to_string());
        }
        mounts.push(oci_bail!(
            MountBuilder::default()
                .destination(std::path::Path::new(&vol.container_path).to_path_buf())
                .typ("bind")
                .source(std::path::Path::new(&vol.host_path).to_path_buf())
                .options(options),
            "failed to build volume mount",
            container_id
        ));
    }

    Ok(mounts)
}

fn parse_seccomp_action(action: &str) -> LinuxSeccompAction {
    match action {
        "SCMP_ACT_KILL" => LinuxSeccompAction::ScmpActKill,
        "SCMP_ACT_KILL_THREAD" => LinuxSeccompAction::ScmpActKillThread,
        "SCMP_ACT_KILL_PROCESS" => LinuxSeccompAction::ScmpActKillProcess,
        "SCMP_ACT_TRAP" => LinuxSeccompAction::ScmpActTrap,
        "SCMP_ACT_ERRNO" => LinuxSeccompAction::ScmpActErrno,
        "SCMP_ACT_TRACE" => LinuxSeccompAction::ScmpActTrace,
        "SCMP_ACT_LOG" => LinuxSeccompAction::ScmpActLog,
        "SCMP_ACT_ALLOW" => LinuxSeccompAction::ScmpActAllow,
        _ => LinuxSeccompAction::ScmpActErrno,
    }
}

fn parse_arch(arch: &str) -> Option<Arch> {
    match arch {
        "SCMP_ARCH_X86_64" => Some(Arch::ScmpArchX86_64),
        "SCMP_ARCH_X86" => Some(Arch::ScmpArchX86),
        "SCMP_ARCH_AARCH64" => Some(Arch::ScmpArchAarch64),
        "SCMP_ARCH_ARM" => Some(Arch::ScmpArchArm),
        "SCMP_ARCH_MIPS" => Some(Arch::ScmpArchMips),
        "SCMP_ARCH_MIPS64" => Some(Arch::ScmpArchMips64),
        "SCMP_ARCH_MIPS64N32" => Some(Arch::ScmpArchMips64n32),
        "SCMP_ARCH_MIPSEL" => Some(Arch::ScmpArchMipsel),
        "SCMP_ARCH_MIPSEL64" => Some(Arch::ScmpArchMipsel64),
        "SCMP_ARCH_MIPSEL64N32" => Some(Arch::ScmpArchMipsel64n32),
        "SCMP_ARCH_PPC" => Some(Arch::ScmpArchPpc),
        "SCMP_ARCH_PPC64" => Some(Arch::ScmpArchPpc64),
        "SCMP_ARCH_PPC64LE" => Some(Arch::ScmpArchPpc64le),
        "SCMP_ARCH_S390X" => Some(Arch::ScmpArchS390x),
        _ => None,
    }
}

fn convert_profile_to_oci(
    data: &SeccompProfileData,
    container_id: &str,
) -> ContainerResult<LinuxSeccomp> {
    let default_action = parse_seccomp_action(&data.default_action);

    let architectures: Vec<Arch> = data
        .architectures
        .iter()
        .filter_map(|a| parse_arch(a))
        .collect();

    let syscalls: Vec<LinuxSyscall> = data
        .syscalls
        .iter()
        .map(|rule| {
            Ok(oci_bail!(
                LinuxSyscallBuilder::default()
                    .names(rule.names.clone())
                    .action(parse_seccomp_action(&rule.action)),
                "failed to build syscall rule",
                container_id
            ))
        })
        .collect::<ContainerResult<Vec<_>>>()?;

    let mut builder = LinuxSeccompBuilder::default();
    builder = builder.default_action(default_action);
    if !architectures.is_empty() {
        builder = builder.architectures(architectures);
    }
    if !syscalls.is_empty() {
        builder = builder.syscalls(syscalls);
    }
    Ok(oci_bail!(
        builder,
        "failed to build seccomp profile",
        container_id
    ))
}

fn build_seccomp(
    params: &ContainerCreateParams,
    container_id: &str,
) -> ContainerResult<Option<LinuxSeccomp>> {
    if _container::seccomp::is_seccomp_disabled() {
        return Ok(None);
    }

    let profile = if params.compile_mode {
        SeccompProfile::Compile
    } else {
        SeccompProfile::EntelecheiaDefault
    };

    let data = profile.to_profile_data();
    Ok(Some(convert_profile_to_oci(&data, container_id)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use _container::types::ContainerCreateParams;
    use anyhow::{Context, Result};

    fn default_params() -> ContainerCreateParams {
        ContainerCreateParams::simple("test-container", "test:latest")
    }

    #[test]
    fn seccomp_applied_in_default_profile() -> Result<()> {
        let params = default_params();
        let seccomp = build_seccomp(&params, "test").context("test precondition")?;
        assert!(seccomp.is_some(), "seccomp should be applied by default");
        let sc = seccomp.context("test precondition")?;
        assert_eq!(sc.default_action(), LinuxSeccompAction::ScmpActErrno);
        let syscalls = sc.syscalls().as_ref().context("test precondition")?;
        assert!(syscalls.len() >= 2, "should have allow and block rules");
        Ok(())
    }

    #[test]
    fn seccomp_compile_mode_uses_compile_profile() -> Result<()> {
        let mut params = default_params();
        params.compile_mode = true;
        let seccomp = build_seccomp(&params, "test").context("test precondition")?;
        assert!(seccomp.is_some());
        let sc = seccomp.context("test precondition")?;
        let syscalls = sc.syscalls().as_ref().context("test precondition")?;
        let block_rule = syscalls
            .iter()
            .find(|r| r.action() == LinuxSeccompAction::ScmpActErrno)
            .context("test precondition")?;
        let names = block_rule.names();
        assert!(
            !names.contains(&"userfaultfd".to_string()),
            "compile profile should allow userfaultfd"
        );
        Ok(())
    }

    #[test]
    fn seccomp_default_profile_blocks_userfaultfd() -> Result<()> {
        let params = default_params();
        let seccomp = build_seccomp(&params, "test")
            .context("test precondition")?
            .context("test precondition")?;
        let syscalls = seccomp.syscalls().as_ref().context("test precondition")?;
        let block_rule = syscalls
            .iter()
            .find(|r| r.action() == LinuxSeccompAction::ScmpActErrno)
            .context("test precondition")?;
        assert!(
            block_rule.names().contains(&"userfaultfd".to_string()),
            "default profile should block userfaultfd"
        );
        Ok(())
    }

    #[test]
    fn seccomp_has_4_architectures() -> Result<()> {
        let params = default_params();
        let seccomp = build_seccomp(&params, "test")
            .context("test precondition")?
            .context("test precondition")?;
        let archs = seccomp
            .architectures()
            .as_ref()
            .context("test precondition")?;
        assert_eq!(archs.len(), 4);
        Ok(())
    }

    #[test]
    fn seccomp_default_profile_allows_basic_syscalls() -> Result<()> {
        let params = default_params();
        let seccomp = build_seccomp(&params, "test")
            .context("test precondition")?
            .context("test precondition")?;
        let syscalls = seccomp.syscalls().as_ref().context("test precondition")?;
        let allow_rule = syscalls
            .iter()
            .find(|r| r.action() == LinuxSeccompAction::ScmpActAllow)
            .context("test precondition")?;
        assert!(allow_rule.names().contains(&"read".to_string()));
        assert!(allow_rule.names().contains(&"write".to_string()));
        assert!(allow_rule.names().contains(&"open".to_string()));
        Ok(())
    }

    #[test]
    fn oci_spec_generates_with_seccomp() -> Result<()> {
        let params = default_params();
        let spec = generate_oci_spec(
            &params,
            std::path::Path::new("/tmp/test-rootfs"),
            "test-id",
            std::path::Path::new("/tmp/test-run"),
        )
        .context("generate_oci_spec should succeed with valid params")?;
        let linux = spec
            .linux()
            .as_ref()
            .context("linux section should exist")?;
        assert!(
            linux.seccomp().is_some(),
            "seccomp should be present in OCI spec"
        );
        Ok(())
    }

    #[test]
    fn default_capabilities_omit_mknod() -> Result<()> {
        let caps = build_capabilities(None, None, false, "test").context("test precondition")?;
        for set in [
            caps.bounding(),
            caps.effective(),
            caps.inheritable(),
            caps.permitted(),
            caps.ambient(),
        ] {
            let set = set.as_ref().context("test precondition")?;
            assert!(
                !set.contains(&Capability::Mknod),
                "MKNOD must be absent from the default capability set"
            );
        }
        Ok(())
    }

    #[test]
    fn mknod_regranted_only_on_explicit_device_request() -> Result<()> {
        // cap_add: MKNOD re-grants the capability.
        let caps = build_capabilities(None, Some(&["MKNOD".to_string()]), false, "test")
            .context("test precondition")?;
        assert!(
            caps.permitted()
                .as_ref()
                .context("test precondition")?
                .contains(&Capability::Mknod),
            "cap_add MKNOD must re-grant the capability"
        );

        // An empty cap_add list does not.
        let caps =
            build_capabilities(None, Some(&[]), false, "test").context("test precondition")?;
        assert!(
            !caps
                .permitted()
                .as_ref()
                .context("test precondition")?
                .contains(&Capability::Mknod),
            "empty cap_add must not grant MKNOD"
        );

        // A non-empty devices passthrough list re-grants it.
        let caps = build_capabilities(None, None, true, "test").context("test precondition")?;
        assert!(
            caps.permitted()
                .as_ref()
                .context("test precondition")?
                .contains(&Capability::Mknod),
            "requesting devices must re-grant MKNOD"
        );
        Ok(())
    }

    #[test]
    fn default_cap_drop_still_removes_mknod() -> Result<()> {
        // Even when MKNOD is requested via devices, an explicit cap_drop wins.
        let caps = build_capabilities(Some(&["MKNOD".to_string()]), None, true, "test")
            .context("test precondition")?;
        assert!(
            !caps
                .permitted()
                .as_ref()
                .context("test precondition")?
                .contains(&Capability::Mknod),
            "cap_drop MKNOD must override the device request"
        );
        Ok(())
    }

    #[test]
    fn cap_drop_all_yields_empty_set() -> Result<()> {
        let caps = build_capabilities(Some(&["ALL".to_string()]), None, false, "test")
            .context("test precondition")?;
        for set in [
            caps.bounding(),
            caps.effective(),
            caps.inheritable(),
            caps.permitted(),
            caps.ambient(),
        ] {
            let set = set.as_ref().context("test precondition")?;
            assert!(
                set.is_empty(),
                "cap_drop ALL must clear every capability set"
            );
        }
        Ok(())
    }

    #[test]
    fn cap_add_readds_after_all_drop() -> Result<()> {
        let caps = build_capabilities(
            Some(&["ALL".to_string()]),
            Some(&["SYS_ADMIN".to_string(), "NET_BIND_SERVICE".to_string()]),
            false,
            "test",
        )
        .context("test precondition")?;
        let permitted = caps.permitted().as_ref().context("test precondition")?;
        assert_eq!(
            permitted.len(),
            2,
            "only re-added capabilities should remain after an ALL drop"
        );
        assert!(permitted.contains(&Capability::SysAdmin));
        assert!(permitted.contains(&Capability::NetBindService));
        Ok(())
    }

    #[test]
    fn unknown_cap_add_is_ignored() -> Result<()> {
        // Unknown names are skipped (with a warning) rather than failing the
        // build; only the recognized re-grant survives the ALL drop.
        let caps = build_capabilities(
            Some(&["ALL".to_string()]),
            Some(&["SYS_ADMIN".to_string(), "NOT_A_REAL_CAP".to_string()]),
            false,
            "test",
        )
        .context("test precondition")?;
        let permitted = caps.permitted().as_ref().context("test precondition")?;
        assert_eq!(permitted.len(), 1, "unknown cap_add must be ignored");
        assert!(permitted.contains(&Capability::SysAdmin));
        Ok(())
    }

    #[test]
    fn resources_include_default_deny_device_cgroup() -> Result<()> {
        let resources = build_resources(None, None, None, "test").context("test precondition")?;
        let devices = resources.devices().as_ref().context("test precondition")?;
        assert!(
            !devices.is_empty(),
            "device cgroup rules must be present by default"
        );
        let deny_all = devices
            .iter()
            .find(|d| !d.allow())
            .context("test precondition")?;
        assert_eq!(
            deny_all.typ(),
            Some(LinuxDeviceType::A),
            "default-deny rule must target all device types"
        );
        assert_eq!(
            deny_all.access().as_deref(),
            Some("rwm"),
            "default-deny rule must deny read/write/mknod"
        );
        Ok(())
    }

    #[test]
    fn oci_spec_process_omits_mknod() -> Result<()> {
        let params = default_params();
        let spec = generate_oci_spec(
            &params,
            std::path::Path::new("/tmp/test-rootfs"),
            "test-id",
            std::path::Path::new("/tmp/test-run"),
        )
        .context("generate_oci_spec should succeed")?;
        let process = spec
            .process()
            .as_ref()
            .context("process section should exist")?;
        let caps = process
            .capabilities()
            .as_ref()
            .context("capabilities should exist")?;
        assert!(
            !caps
                .permitted()
                .as_ref()
                .context("test precondition")?
                .contains(&Capability::Mknod),
            "generated spec must omit MKNOD by default"
        );
        Ok(())
    }
}
