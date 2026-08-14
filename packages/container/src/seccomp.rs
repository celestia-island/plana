//! Seccomp syscall-filter profile construction for container sandboxes.
//!
//! [`SeccompProfile`] describes a deny-by-default syscall filter (default
//! action `SCMP_ACT_ERRNO`) with an explicit allow-list of the syscalls a
//! sandboxed container legitimately needs. The filter is embedded as a
//! `seccomp=...` Docker `security_opt`, or converted to the OCI `LinuxSeccomp`
//! spec by the `container_runtime` crate.
//!
//! # DEV-ONLY escape hatch
//!
//! The `DISABLE_SECCOMP` environment variable (see [`is_seccomp_disabled`])
//! disables the filter entirely and MUST NEVER be set in production.

use serde::{Deserialize, Serialize};

pub fn default_seccomp_security_opt() -> String {
    "no-new-privileges:true".to_string()
}

/// Returns true when the `DISABLE_SECCOMP` escape hatch is enabled.
///
/// # DEV-ONLY escape hatch — never set in production
///
/// `DISABLE_SECCOMP=true` (or `1`) makes [`build_security_opts`] omit the
/// seccomp syscall filter entirely, leaving the container protected only by the
/// runtime's default policy (if any). The deny-by-default filter is what blocks
/// `ptrace`, `mount`, `bpf`, `kexec_load`, `unshare`, and other escape vectors,
/// so this variable MUST NEVER be set in production. It exists only to unblock
/// local development when a container image needs a syscall the profile does not
/// yet allow.
pub fn is_seccomp_disabled() -> bool {
    let disabled = seccomp_disabled_from_value(std::env::var("DISABLE_SECCOMP").ok().as_deref());

    if disabled {
        tracing::warn!(
            "DISABLE_SECCOMP is set: the seccomp syscall filter is DISABLED. \
             This is a DEV-ONLY escape hatch and must never be enabled in production."
        );
    }

    disabled
}

fn seccomp_disabled_from_value(value: Option<&str>) -> bool {
    matches!(value, Some(v) if v.eq_ignore_ascii_case("true") || v == "1")
}

pub fn build_security_opts(profile: Option<SeccompProfile>) -> Vec<String> {
    build_security_opts_with_disabled(profile, is_seccomp_disabled())
}

fn build_security_opts_with_disabled(
    profile: Option<SeccompProfile>,
    seccomp_disabled: bool,
) -> Vec<String> {
    let mut opts = Vec::new();

    opts.push("no-new-privileges:true".to_string());

    if !seccomp_disabled {
        let profile = profile.unwrap_or_default();
        if let Ok(json) = profile.to_json_string() {
            opts.push(format!("seccomp={}", json));
        }
    }

    opts
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeccompSyscallRule {
    pub names: Vec<String>,
    pub action: String,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum SeccompProfile {
    #[default]
    EntelecheiaDefault,
    Compile,
}

impl SeccompProfile {
    pub fn to_profile_data(&self) -> SeccompProfileData {
        match self {
            Self::EntelecheiaDefault => SeccompProfileData::entelecheia_default(),
            Self::Compile => SeccompProfileData::compile(),
        }
    }

    pub fn to_json_string(&self) -> serde_json::Result<String> {
        self.to_profile_data().to_json_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeccompProfileData {
    #[serde(rename = "defaultAction")]
    pub default_action: String,
    pub architectures: Vec<String>,
    pub syscalls: Vec<SeccompSyscallRule>,
}

impl SeccompProfileData {
    pub fn entelecheia_default() -> Self {
        Self {
            default_action: "SCMP_ACT_ERRNO".to_string(),
            architectures: vec![
                "SCMP_ARCH_X86_64".to_string(),
                "SCMP_ARCH_X86".to_string(),
                "SCMP_ARCH_AARCH64".to_string(),
                "SCMP_ARCH_ARM".to_string(),
            ],
            syscalls: vec![
                SeccompSyscallRule {
                    names: vec![
                        "accept".into(),
                        "accept4".into(),
                        "access".into(),
                        "arch_prctl".into(),
                        "bind".into(),
                        "brk".into(),
                        "capget".into(),
                        "capset".into(),
                        "chdir".into(),
                        "chmod".into(),
                        "chown".into(),
                        "chown32".into(),
                        "clock_getres".into(),
                        "clock_gettime".into(),
                        "clock_nanosleep".into(),
                        "clone".into(),
                        "clone3".into(),
                        "close".into(),
                        "connect".into(),
                        "copy_file_range".into(),
                        "creat".into(),
                        "dup".into(),
                        "dup2".into(),
                        "dup3".into(),
                        "epoll_create".into(),
                        "epoll_create1".into(),
                        "epoll_ctl".into(),
                        "epoll_pwait".into(),
                        "epoll_wait".into(),
                        "eventfd".into(),
                        "eventfd2".into(),
                        "execve".into(),
                        "execveat".into(),
                        "exit".into(),
                        "exit_group".into(),
                        "faccessat".into(),
                        "faccessat2".into(),
                        "fadvise64".into(),
                        "fallocate".into(),
                        "fanotify_mark".into(),
                        "fchdir".into(),
                        "fchmod".into(),
                        "fchmodat".into(),
                        "fchown".into(),
                        "fchown32".into(),
                        "fchownat".into(),
                        "fcntl".into(),
                        "fcntl64".into(),
                        "fdatasync".into(),
                        "fgetxattr".into(),
                        "flistxattr".into(),
                        "flock".into(),
                        "fork".into(),
                        "fremovexattr".into(),
                        "fsetxattr".into(),
                        "fstat".into(),
                        "fstat64".into(),
                        "fstatat64".into(),
                        "fstatfs".into(),
                        "fstatfs64".into(),
                        "fsync".into(),
                        "ftruncate".into(),
                        "ftruncate64".into(),
                        "futex".into(),
                        "futimesat".into(),
                        "getcwd".into(),
                        "getdents".into(),
                        "getdents64".into(),
                        "getegid".into(),
                        "getegid32".into(),
                        "geteuid".into(),
                        "geteuid32".into(),
                        "getgid".into(),
                        "getgid32".into(),
                        "getgroups".into(),
                        "getitimer".into(),
                        "getpeername".into(),
                        "getpgrp".into(),
                        "getpid".into(),
                        "getppid".into(),
                        "getpriority".into(),
                        "getrandom".into(),
                        "getresgid".into(),
                        "getresgid32".into(),
                        "getresuid".into(),
                        "getresuid32".into(),
                        "getrlimit".into(),
                        "getsockname".into(),
                        "getsockopt".into(),
                        "gettid".into(),
                        "gettimeofday".into(),
                        "getuid".into(),
                        "getuid32".into(),
                        "getxattr".into(),
                        "inotify_add_watch".into(),
                        "inotify_init".into(),
                        "inotify_init1".into(),
                        "inotify_rm_watch".into(),
                        "io_cancel".into(),
                        "io_destroy".into(),
                        "io_getevents".into(),
                        "io_setup".into(),
                        "io_submit".into(),
                        "ioctl".into(),
                        "ioprio_get".into(),
                        "ioprio_set".into(),
                        "ipc".into(),
                        "kill".into(),
                        "lchown".into(),
                        "lchown32".into(),
                        "lgetxattr".into(),
                        "link".into(),
                        "linkat".into(),
                        "listen".into(),
                        "listxattr".into(),
                        "llistxattr".into(),
                        "lremovexattr".into(),
                        "lseek".into(),
                        "lsetxattr".into(),
                        "lstat".into(),
                        "lstat64".into(),
                        "madvise".into(),
                        "memfd_create".into(),
                        "mincore".into(),
                        "mkdir".into(),
                        "mkdirat".into(),
                        "mknod".into(),
                        "mknodat".into(),
                        "mlock".into(),
                        "mlock2".into(),
                        "mlockall".into(),
                        "mmap".into(),
                        "mmap2".into(),
                        "mprotect".into(),
                        "mremap".into(),
                        "msgctl".into(),
                        "msgget".into(),
                        "msgrcv".into(),
                        "msgsnd".into(),
                        "msync".into(),
                        "munlock".into(),
                        "munlockall".into(),
                        "munmap".into(),
                        "nanosleep".into(),
                        "newfstatat".into(),
                        "open".into(),
                        "openat".into(),
                        "openat2".into(),
                        "pipe".into(),
                        "pipe2".into(),
                        "poll".into(),
                        "ppoll".into(),
                        "prctl".into(),
                        "pread64".into(),
                        "preadv".into(),
                        "preadv2".into(),
                        "prlimit64".into(),
                        "pselect6".into(),
                        "pwrite64".into(),
                        "pwritev".into(),
                        "pwritev2".into(),
                        "read".into(),
                        "readahead".into(),
                        "readlink".into(),
                        "readlinkat".into(),
                        "readv".into(),
                        "recv".into(),
                        "recvfrom".into(),
                        "recvmmsg".into(),
                        "recvmsg".into(),
                        "rename".into(),
                        "renameat".into(),
                        "renameat2".into(),
                        "restart_syscall".into(),
                        "rmdir".into(),
                        "rt_sigaction".into(),
                        "rt_sigprocmask".into(),
                        "rt_sigreturn".into(),
                        "rt_sigsuspend".into(),
                        "rt_sigtimedwait".into(),
                        "sched_getaffinity".into(),
                        "sched_setaffinity".into(),
                        "sched_yield".into(),
                        "seccomp".into(),
                        "select".into(),
                        "semctl".into(),
                        "semget".into(),
                        "semop".into(),
                        "semtimedop".into(),
                        "send".into(),
                        "sendfile".into(),
                        "sendfile64".into(),
                        "sendmmsg".into(),
                        "sendmsg".into(),
                        "sendto".into(),
                        "set_robust_list".into(),
                        "set_tid_address".into(),
                        "setfsgid".into(),
                        "setfsgid32".into(),
                        "setfsuid".into(),
                        "setfsuid32".into(),
                        "setgid".into(),
                        "setgid32".into(),
                        "setgroups".into(),
                        "setitimer".into(),
                        "setpgid".into(),
                        "setpriority".into(),
                        "setregid".into(),
                        "setregid32".into(),
                        "setresgid".into(),
                        "setresgid32".into(),
                        "setresuid".into(),
                        "setresuid32".into(),
                        "setrlimit".into(),
                        "setsid".into(),
                        "setsockopt".into(),
                        "setuid".into(),
                        "setuid32".into(),
                        "setxattr".into(),
                        "shmat".into(),
                        "shmctl".into(),
                        "shmdt".into(),
                        "shmget".into(),
                        "shutdown".into(),
                        "sigaltstack".into(),
                        "signalfd".into(),
                        "signalfd4".into(),
                        "socket".into(),
                        "socketpair".into(),
                        "splice".into(),
                        "stat".into(),
                        "stat64".into(),
                        "statfs".into(),
                        "statfs64".into(),
                        "statx".into(),
                        "symlink".into(),
                        "symlinkat".into(),
                        "sync".into(),
                        "sync_file_range".into(),
                        "syncfs".into(),
                        "sysinfo".into(),
                        "syslog".into(),
                        "tee".into(),
                        "tgkill".into(),
                        "timer_create".into(),
                        "timer_delete".into(),
                        "timer_getoverrun".into(),
                        "timer_gettime".into(),
                        "timer_settime".into(),
                        "timerfd_create".into(),
                        "timerfd_gettime".into(),
                        "timerfd_settime".into(),
                        "times".into(),
                        "tkill".into(),
                        "truncate".into(),
                        "truncate64".into(),
                        "umask".into(),
                        "uname".into(),
                        "unlink".into(),
                        "unlinkat".into(),
                        // `unshare` is intentionally NOT allow-listed: it creates
                        // new namespaces and becomes an escape vector when combined
                        // with SYS_ADMIN (which cosmos/scepter re-add). No
                        // in-container code demonstrably needs it — namespace
                        // creation for sub-containers is performed by the container
                        // runtime on the host, outside the sandbox — so it falls
                        // through to the deny-by-default action instead.
                        "utime".into(),
                        "utimensat".into(),
                        "utimes".into(),
                        "vfork".into(),
                        "vmsplice".into(),
                        "wait4".into(),
                        "waitid".into(),
                        "waitpid".into(),
                        "write".into(),
                        "writev".into(),
                    ],
                    action: "SCMP_ACT_ALLOW".into(),
                },
                SeccompSyscallRule {
                    names: vec![
                        "ptrace".into(),
                        "mount".into(),
                        "umount2".into(),
                        "kexec_load".into(),
                        "kexec_file_load".into(),
                        "bpf".into(),
                        "reboot".into(),
                        "add_key".into(),
                        "keyctl".into(),
                        "request_key".into(),
                        "init_module".into(),
                        "finit_module".into(),
                        "delete_module".into(),
                        "swapon".into(),
                        "swapoff".into(),
                        "sysfs".into(),
                        "uselib".into(),
                        "acct".into(),
                        "modify_ldt".into(),
                        "personality".into(),
                        "process_vm_readv".into(),
                        "process_vm_writev".into(),
                        "vhangup".into(),
                        "pivot_root".into(),
                        "ioperm".into(),
                        "iopl".into(),
                        "perf_event_open".into(),
                        "userfaultfd".into(),
                    ],
                    action: "SCMP_ACT_ERRNO".into(),
                },
            ],
        }
    }

    pub fn compile() -> Self {
        let mut profile = Self::entelecheia_default();
        if let Some(block_rule) = profile
            .syscalls
            .iter_mut()
            .find(|r| r.action == "SCMP_ACT_ERRNO")
        {
            block_rule
                .names
                .retain(|name| !matches!(name.as_str(), "userfaultfd"));
        }
        profile
    }

    pub fn to_json_string(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result, ensure};

    #[test]
    fn seccomp_profile_generates_valid_json() -> Result<()> {
        let profile = SeccompProfile::EntelecheiaDefault;
        let json = profile.to_json_string()?;
        assert!(json.contains("SCMP_ACT_ERRNO"));
        assert!(json.contains("SCMP_ACT_ALLOW"));
        assert!(json.contains("ptrace"));
        assert!(json.contains("open"));
        assert!(json.contains("read"));
        assert!(json.contains("write"));
        Ok(())
    }

    #[test]
    fn seccomp_profile_allows_basic_syscalls() -> Result<()> {
        let profile = SeccompProfile::EntelecheiaDefault.to_profile_data();
        let allow_rule = &profile.syscalls[0];
        assert_eq!(allow_rule.action, "SCMP_ACT_ALLOW");
        assert!(allow_rule.names.contains(&"read".to_string()));
        assert!(allow_rule.names.contains(&"write".to_string()));
        assert!(allow_rule.names.contains(&"open".to_string()));
        assert!(allow_rule.names.contains(&"close".to_string()));
        assert!(allow_rule.names.contains(&"fstat".to_string()));
        assert!(allow_rule.names.contains(&"mmap".to_string()));
        assert!(allow_rule.names.contains(&"brk".to_string()));
        assert!(allow_rule.names.contains(&"futex".to_string()));
        Ok(())
    }

    #[test]
    fn seccomp_profile_blocks_dangerous_syscalls() -> Result<()> {
        let profile = SeccompProfile::EntelecheiaDefault.to_profile_data();
        let block_rule = &profile.syscalls[1];
        assert_eq!(block_rule.action, "SCMP_ACT_ERRNO");
        assert!(block_rule.names.contains(&"ptrace".to_string()));
        assert!(block_rule.names.contains(&"mount".to_string()));
        assert!(block_rule.names.contains(&"kexec_load".to_string()));
        assert!(block_rule.names.contains(&"bpf".to_string()));
        assert!(block_rule.names.contains(&"reboot".to_string()));
        Ok(())
    }

    #[test]
    fn seccomp_profile_has_4_architectures() -> Result<()> {
        let profile = SeccompProfile::EntelecheiaDefault.to_profile_data();
        assert_eq!(profile.architectures.len(), 4);
        Ok(())
    }

    #[test]
    fn build_security_opts_includes_no_new_privileges() -> Result<()> {
        let opts = build_security_opts(None);
        assert!(opts.contains(&"no-new-privileges:true".to_string()));
        Ok(())
    }

    #[test]
    fn build_security_opts_includes_seccomp_profile() -> Result<()> {
        let opts = build_security_opts(None);
        let has_seccomp = opts.iter().any(|o| o.starts_with("seccomp="));
        ensure!(
            has_seccomp,
            "security opts should include seccomp profile: {:?}",
            opts
        );
        Ok(())
    }

    #[test]
    fn build_security_opts_merges_additional() -> Result<()> {
        let opts = build_security_opts(None);
        assert!(opts.contains(&"no-new-privileges:true".to_string()));
        Ok(())
    }

    #[test]
    fn build_security_opts_no_duplicates() -> Result<()> {
        let opts = build_security_opts(None);
        let count = opts
            .iter()
            .filter(|o| **o == "no-new-privileges:true")
            .count();
        assert_eq!(count, 1);
        Ok(())
    }

    #[test]
    fn seccomp_profile_present_by_default() -> Result<()> {
        let opts = build_security_opts_with_disabled(None, false);
        ensure!(
            opts.iter().any(|o| o.starts_with("seccomp=")),
            "seccomp profile must be present when not disabled: {:?}",
            opts
        );
        Ok(())
    }

    #[test]
    fn seccomp_profile_absent_only_when_explicitly_disabled() -> Result<()> {
        let opts = build_security_opts_with_disabled(None, true);
        ensure!(
            !opts.iter().any(|o| o.starts_with("seccomp=")),
            "seccomp profile must be absent only when DISABLE_SECCOMP is enabled: {:?}",
            opts
        );
        ensure!(
            opts.contains(&"no-new-privileges:true".to_string()),
            "no-new-privileges must remain even when seccomp is disabled"
        );
        Ok(())
    }

    #[test]
    fn seccomp_disabled_parses_only_truthy_values() -> Result<()> {
        assert!(seccomp_disabled_from_value(Some("true")));
        assert!(seccomp_disabled_from_value(Some("TRUE")));
        assert!(seccomp_disabled_from_value(Some("1")));
        assert!(!seccomp_disabled_from_value(Some("false")));
        assert!(!seccomp_disabled_from_value(Some("0")));
        assert!(!seccomp_disabled_from_value(Some("")));
        assert!(!seccomp_disabled_from_value(None));
        Ok(())
    }

    #[test]
    fn unshare_not_allow_or_deny_listed() -> Result<()> {
        // `unshare` is an escape vector: it must appear in NEITHER the
        // allow-list NOR the explicit deny-list of any profile, relying on the
        // deny-by-default action (SCMP_ACT_ERRNO) to block it.
        for profile in [
            SeccompProfile::EntelecheiaDefault.to_profile_data(),
            SeccompProfile::Compile.to_profile_data(),
        ] {
            let allow_rule = profile
                .syscalls
                .iter()
                .find(|r| r.action == "SCMP_ACT_ALLOW")
                .context("allow rule expected")?;
            let deny_rule = profile
                .syscalls
                .iter()
                .find(|r| r.action == "SCMP_ACT_ERRNO")
                .context("deny rule expected")?;
            ensure!(
                !allow_rule.names.contains(&"unshare".to_string()),
                "unshare must not be allow-listed"
            );
            ensure!(
                !deny_rule.names.contains(&"unshare".to_string()),
                "unshare must not be explicitly deny-listed (rely on default deny)"
            );
        }
        Ok(())
    }
}
