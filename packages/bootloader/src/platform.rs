//! Platform service registration — install / uninstall / status for evernight
//! as a native OS service on Linux (systemd user unit), Windows (SCM via
//! `sc.exe`), and macOS (launchd `LaunchDaemon` plist).
//!
//! The trait is async so the platform impls can shell out to `systemctl` /
//! `sc.exe` / `launchctl` without blocking. All three impls are gated by
//! `#[cfg(target_os = ...)]` so only the matching one compiles per target; the
//! `default_service_installer()` constructor returns the right one.

use std::path::PathBuf;

#[allow(unused_imports)]
use anyhow::{Context, Result, anyhow, bail};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
// `warn`/`anyhow!` are used inside platform-specific impl blocks; on a given
// target only one branch compiles, so we allow the unused warnings rather than
// scatter cfg-gates across every call site.
#[allow(unused_imports)]
use tracing::{info, warn};

/// What to install. The binary path + its args form the service's ExecStart;
/// `run_at_startup` maps to systemd `WantedBy=default.target`, SCM
/// `start=auto`, or launchd `RunAtLoad=true`.
#[derive(Debug, Clone)]
pub struct ServiceSpec {
    /// Service name (without platform-specific suffix; the installer adds it).
    /// e.g. `evernight` → `evernight.service` / `Evernight` (SCM) /
    /// `io.celestia.evernight.plist`.
    pub name: String,
    /// Absolute path to the binary the service should run.
    pub bin_path: PathBuf,
    /// Args passed to the binary (e.g. `["supervise"]`).
    pub args: Vec<String>,
    /// Human description shown by the platform's status UI.
    pub description: String,
    /// Start automatically at boot / login.
    pub run_at_startup: bool,
}

impl ServiceSpec {
    pub fn new(name: impl Into<String>, bin_path: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            bin_path: bin_path.into(),
            args: vec!["supervise".to_string()],
            description: "Celestia evernight bootloader".to_string(),
            run_at_startup: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceStatus {
    Installed,
    NotInstalled,
    /// Installed but not currently running.
    Stopped,
    Running,
}

/// Install / uninstall / query a native OS service. Implementations live
/// behind `#[cfg(target_os = ...)]` below. Uses `#[async_trait]` so the trait
/// remains object-safe (`Box<dyn ServiceInstaller>`).
#[async_trait]
pub trait ServiceInstaller: Send + Sync {
    async fn install(&self, spec: &ServiceSpec) -> Result<()>;
    async fn uninstall(&self) -> Result<()>;
    async fn status(&self) -> Result<ServiceStatus>;
}

/// Return the platform-native installer for the current target.
pub fn default_service_installer() -> Box<dyn ServiceInstaller> {
    #[cfg(target_os = "linux")]
    {
        Box::new(SystemdInstaller)
    }
    #[cfg(target_os = "windows")]
    {
        Box::new(WindowsScmInstaller)
    }
    #[cfg(target_os = "macos")]
    {
        Box::new(LaunchdInstaller::default())
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Box::new(UnsupportedInstaller)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Linux: systemd user service
// ────────────────────────────────────────────────────────────────────────────
#[cfg(target_os = "linux")]
struct SystemdInstaller;

#[cfg(target_os = "linux")]
impl SystemdInstaller {
    fn unit_path(&self, name: &str) -> Result<PathBuf> {
        let home = dirs::home_dir().context("cannot determine HOME")?;
        Ok(home
            .join(".config/systemd/user")
            .join(format!("{name}.service")))
    }

    fn render_unit(&self, spec: &ServiceSpec) -> Result<String> {
        let args = if spec.args.is_empty() {
            String::new()
        } else {
            format!(" {}", spec.args.join(" "))
        };
        let wanted_by = if spec.run_at_startup {
            "WantedBy=default.target\n"
        } else {
            ""
        };
        Ok(format!(
            "[Unit]\n\
             Description={desc}\n\
             After=default.target\n\
             \n\
             [Service]\n\
             Type=simple\n\
             ExecStart={bin}{args}\n\
             Restart=on-failure\n\
             RestartSec=5\n\
             \n\
             [Install]\n\
             {wanted_by}",
            desc = spec.description,
            bin = spec.bin_path.display(),
            args = args,
            wanted_by = wanted_by,
        ))
    }
}

#[cfg(target_os = "linux")]
#[async_trait]
impl ServiceInstaller for SystemdInstaller {
    async fn install(&self, spec: &ServiceSpec) -> Result<()> {
        let unit_path = self.unit_path(&spec.name)?;
        if let Some(parent) = unit_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let unit = self.render_unit(spec)?;
        std::fs::write(&unit_path, unit)
            .with_context(|| format!("failed to write {}", unit_path.display()))?;
        info!("installed systemd unit: {}", unit_path.display());

        for cmd in [
            vec!["systemctl", "--user", "daemon-reload"],
            vec!["systemctl", "--user", "enable", &spec.name],
        ] {
            let out = tokio::process::Command::new(cmd[0])
                .args(&cmd[1..])
                .output()
                .await
                .with_context(|| format!("failed to run {cmd:?}"))?;
            if !out.status.success() {
                warn!(
                    "systemctl {:?}: {}",
                    cmd,
                    String::from_utf8_lossy(&out.stderr).trim()
                );
            }
        }
        // enable-linger so the user service survives logout.
        let user = std::env::var("USER").unwrap_or_else(|_| "evernight".into());
        let _ = tokio::process::Command::new("loginctl")
            .args(["enable-linger", &user])
            .output()
            .await;
        info!("systemd user service installed and enabled");
        Ok(())
    }

    async fn uninstall(&self) -> Result<()> {
        let name = "evernight";
        let _ = tokio::process::Command::new("systemctl")
            .args(["--user", "stop", name])
            .output()
            .await;
        let _ = tokio::process::Command::new("systemctl")
            .args(["--user", "disable", name])
            .output()
            .await;
        let unit_path = self.unit_path(name)?;
        let _ = std::fs::remove_file(&unit_path);
        let _ = tokio::process::Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .output()
            .await;
        info!("systemd user service removed");
        Ok(())
    }

    async fn status(&self) -> Result<ServiceStatus> {
        let name = "evernight";
        let unit_path = self.unit_path(name)?;
        if !unit_path.exists() {
            return Ok(ServiceStatus::NotInstalled);
        }
        let out = tokio::process::Command::new("systemctl")
            .args(["--user", "is-active", name])
            .output()
            .await;
        match out {
            Ok(o) if o.status.success() => Ok(ServiceStatus::Running),
            Ok(_) => Ok(ServiceStatus::Stopped),
            Err(_) => Ok(ServiceStatus::Installed),
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Windows: Service Control Manager via sc.exe
// ────────────────────────────────────────────────────────────────────────────
#[cfg(target_os = "windows")]
struct WindowsScmInstaller;

#[cfg(target_os = "windows")]
#[async_trait]
impl ServiceInstaller for WindowsScmInstaller {
    async fn install(&self, spec: &ServiceSpec) -> Result<()> {
        let bin_with_args = format!(
            "{} {} {}",
            spec.bin_path.display(),
            spec.args.join(" "),
            // SCM wants the binary path wrapped in quotes if it has spaces;
            // we always quote to be safe.
            ""
        );
        let bin_arg = format!("\"{}\" {}", spec.bin_path.display(), spec.args.join(" "));

        // create the service. start=auto ↔ run_at_startup.
        let start = if spec.run_at_startup {
            "auto"
        } else {
            "demand"
        };
        let out = tokio::process::Command::new("sc")
            .args([
                "create",
                &spec.name,
                "binPath=",
                &bin_arg,
                "start=",
                start,
                "DisplayName=",
                &spec.description,
            ])
            .output()
            .await
            .context("failed to run sc.exe create")?;
        if !out.status.success() {
            // service may already exist — try to reconfigure instead.
            let existing = String::from_utf8_lossy(&out.stderr);
            if existing.contains("already exists") || existing.contains("1073") {
                info!("service already exists, reconfiguring");
                let _ = tokio::process::Command::new("sc")
                    .args(["config", &spec.name, "binPath=", &bin_arg, "start=", start])
                    .output()
                    .await;
            } else {
                bail!(
                    "sc.exe create failed: {}",
                    String::from_utf8_lossy(&out.stderr).trim()
                );
            }
        }
        let _ = bin_with_args; // suppress unused warning on the unquoted form
        info!("installed Windows SCM service: {}", spec.name);
        Ok(())
    }

    async fn uninstall(&self) -> Result<()> {
        let name = "evernight";
        let _ = tokio::process::Command::new("sc")
            .args(["stop", name])
            .output()
            .await;
        let out = tokio::process::Command::new("sc")
            .args(["delete", name])
            .output()
            .await
            .context("failed to run sc.exe delete")?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            if !err.contains("does not exist") && !err.contains("1060") {
                bail!("sc.exe delete failed: {}", err.trim());
            }
        }
        info!("Windows SCM service removed");
        Ok(())
    }

    async fn status(&self) -> Result<ServiceStatus> {
        let name = "evernight";
        let out = tokio::process::Command::new("sc")
            .args(["query", name])
            .output()
            .await;
        match out {
            Err(_) => Ok(ServiceStatus::NotInstalled),
            Ok(o) if !o.status.success() => Ok(ServiceStatus::NotInstalled),
            Ok(o) => {
                let text = String::from_utf8_lossy(&o.stdout);
                if text.contains("RUNNING") {
                    Ok(ServiceStatus::Running)
                } else {
                    Ok(ServiceStatus::Stopped)
                }
            }
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// macOS: launchd LaunchDaemon plist
// ────────────────────────────────────────────────────────────────────────────
#[cfg(target_os = "macos")]
#[derive(Default)]
struct LaunchdInstaller {
    domain: Option<String>,
}

#[cfg(target_os = "macos")]
impl LaunchdInstaller {
    fn plist_path(&self, name: &str) -> Result<PathBuf> {
        // LaunchDaemons run at boot (system-wide); LaunchAgents run at login.
        // We install as a LaunchDaemon since evernight is a host service.
        Ok(PathBuf::from("/Library/LaunchDaemons").join(format!("io.celestia.{name}.plist")))
    }

    fn render_plist(&self, spec: &ServiceSpec) -> String {
        // A minimal but correct launchd plist. ProgramArguments is the binary
        // followed by each arg as its own <string>.
        let mut args = format!("<string>{}</string>\n", spec.bin_path.display());
        for a in &spec.args {
            args.push_str(&format!("\t\t<string>{}</string>\n", a));
        }
        let label = format!("io.celestia.{}", spec.name);
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\">\n\
             <dict>\n\
             \t<key>Label</key>\n\
             \t<string>{label}</string>\n\
             \t<key>ProgramArguments</key>\n\
             \t<array>\n\
             \t\t{args}\
             \t</array>\n\
             \t<key>RunAtLoad</key>\n\
             \t<{run_at_load}/>\n\
             \t<key>KeepAlive</key>\n\
             \t<true/>\n\
             \t<key>StandardOutPath</key>\n\
             \t<string>/tmp/{name}.out.log</string>\n\
             \t<key>StandardErrorPath</key>\n\
             \t<string>/tmp/{name}.err.log</string>\n\
             </dict>\n\
             </plist>\n",
            label = label,
            args = args,
            run_at_load = if spec.run_at_startup { "true" } else { "false" },
            name = spec.name,
        )
    }
}

#[cfg(target_os = "macos")]
#[async_trait]
impl ServiceInstaller for LaunchdInstaller {
    async fn install(&self, spec: &ServiceSpec) -> Result<()> {
        let plist = self.render_plist(spec);
        let path = self.plist_path(&spec.name)?;
        // /Library/LaunchDaemons requires root; write + chmod 0644 + chown root:wheel.
        std::fs::write(&path, &plist)
            .with_context(|| format!("failed to write {}", path.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644));
        }
        info!("installed launchd plist: {}", path.display());

        let label = format!("io.celestia.{}", spec.name);
        let out = tokio::process::Command::new("launchctl")
            .args(["bootstrap", "system", &path.to_string_lossy()])
            .output()
            .await
            .context("failed to run launchctl bootstrap")?;
        if !out.status.success() {
            // Already-bootstrapped is recoverable: bootout then bootstrap.
            let err = String::from_utf8_lossy(&out.stderr);
            if err.contains("already") || err.contains("Bootstrap") {
                info!("plist already bootstrapped, reloading");
                let _ = tokio::process::Command::new("launchctl")
                    .args(["bootout", &format!("system/{label}")])
                    .output()
                    .await;
                let _ = tokio::process::Command::new("launchctl")
                    .args(["bootstrap", "system", &path.to_string_lossy()])
                    .output()
                    .await;
            } else {
                bail!("launchctl bootstrap failed: {}", err.trim());
            }
        }
        info!("launchd daemon installed: {label}");
        Ok(())
    }

    async fn uninstall(&self) -> Result<()> {
        let label = "io.celestia.evernight";
        let _ = tokio::process::Command::new("launchctl")
            .args(["bootout", &format!("system/{label}")])
            .output()
            .await;
        let path = self.plist_path("evernight")?;
        let _ = std::fs::remove_file(&path);
        info!("launchd daemon removed");
        Ok(())
    }

    async fn status(&self) -> Result<ServiceStatus> {
        let path = self.plist_path("evernight")?;
        if !path.exists() {
            return Ok(ServiceStatus::NotInstalled);
        }
        let out = tokio::process::Command::new("launchctl")
            .args(["print", "system/io.celestia.evernight"])
            .output()
            .await;
        match out {
            Ok(o) if o.status.success() => Ok(ServiceStatus::Running),
            Ok(_) => Ok(ServiceStatus::Stopped),
            Err(_) => Ok(ServiceStatus::Installed),
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Unsupported target fallback
// ────────────────────────────────────────────────────────────────────────────
#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
struct UnsupportedInstaller;

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
#[async_trait]
impl ServiceInstaller for UnsupportedInstaller {
    async fn install(&self, _spec: &ServiceSpec) -> Result<()> {
        Err(anyhow!(
            "native service install is not supported on this platform"
        ))
    }
    async fn uninstall(&self) -> Result<()> {
        Err(anyhow!(
            "native service uninstall is not supported on this platform"
        ))
    }
    async fn status(&self) -> Result<ServiceStatus> {
        Ok(ServiceStatus::NotInstalled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_defaults_to_supervise() {
        let s = ServiceSpec::new("evernight", "/usr/local/bin/evernight");
        assert_eq!(s.args, vec!["supervise".to_string()]);
        assert!(s.run_at_startup);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_unit_renders_with_args() {
        let inst = SystemdInstaller;
        let spec = ServiceSpec {
            name: "evernight".into(),
            bin_path: "/usr/local/bin/evernight".into(),
            args: vec!["supervise".into(), "--once".into()],
            description: "Celestia evernight".into(),
            run_at_startup: true,
        };
        let unit = inst.render_unit(&spec).unwrap();
        assert!(unit.contains("ExecStart=/usr/local/bin/evernight supervise --once"));
        assert!(unit.contains("WantedBy=default.target"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_plist_renders() {
        let inst = LaunchdInstaller::default();
        let spec = ServiceSpec::new("evernight", "/usr/local/bin/evernight");
        let plist = inst.render_plist(&spec);
        assert!(plist.contains("<key>Label</key>"));
        assert!(plist.contains("io.celestia.evernight"));
        assert!(plist.contains("<key>RunAtLoad</key>"));
        assert!(plist.contains("<true/>"));
    }
}
