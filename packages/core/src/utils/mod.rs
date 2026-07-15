pub mod bytes_base64;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::constants::strings::*;

fn windows_version_label(info: &os_info::Info) -> String {
    let version = info.version();
    // Windows 10 build >= 22000 is Windows 11
    if let os_info::Version::Semantic(major, minor, build) = version
        && *major == 10
        && *minor == 0
    {
        if *build >= 22000 {
            return "Windows 11".to_string();
        }
        return "Windows 10".to_string();
    }
    format!("Windows {version}")
}

fn detect_linux_distro_label() -> Option<String> {
    std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|text| {
            text.lines()
                .find_map(|line| line.strip_prefix("PRETTY_NAME="))
                .map(|value| value.trim_matches('"').trim().to_string())
                .filter(|value| !value.is_empty())
        })
}

pub fn simplify_platform_title(platform: &str) -> String {
    fn simplify_linux_label(label: &str) -> String {
        let mut normalized = label.trim().replace("GNU/Linux", "");
        normalized = normalized.replace("Linux", " ");
        normalized = normalized.replace("LTS", "");
        normalized = normalized.replace("rolling", "Rolling");

        if let Some(paren_start) = normalized.find('(') {
            normalized.truncate(paren_start);
        }

        let normalized = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
        let normalized = normalized.trim();
        if normalized.is_empty() {
            return "Linux".to_string();
        }

        let replacements = [
            ("Ubuntu", "Ubuntu"),
            ("Debian", "Debian"),
            ("Fedora", "Fedora"),
            ("Arch", "Arch"),
            ("Manjaro", "Manjaro"),
            ("Linux Mint", "Linux Mint"),
            ("Pop!_OS", "Pop!_OS"),
            ("Kali", "Kali"),
            ("openSUSE Tumbleweed", "openSUSE Tumbleweed"),
            ("openSUSE Leap", "openSUSE Leap"),
            ("Rocky", "Rocky Linux"),
            ("AlmaLinux", "AlmaLinux"),
            ("CentOS Stream", "CentOS Stream"),
            ("CentOS", "CentOS"),
            ("Raspbian", "Raspbian"),
            ("Amazon", "Amazon Linux"),
        ];

        for (prefix, replacement) in replacements {
            if let Some(rest) = normalized.strip_prefix(prefix) {
                let suffix = rest.trim();
                return if suffix.is_empty() {
                    replacement.to_string()
                } else {
                    format!("{replacement} {suffix}")
                };
            }
        }

        normalized.to_string()
    }

    let trimmed = platform.trim();
    if trimmed.is_empty() {
        return trimmed.to_string();
    }

    let preprocessed = trimmed.replace("GNU/Linux", "");

    let mut parts = preprocessed.split('/').map(str::trim);
    let primary = parts.next().unwrap_or(&preprocessed);
    let suffix = parts.next();

    let simplified_primary = simplify_linux_label(primary);
    match suffix {
        Some(suffix) if !suffix.is_empty() => format!("{simplified_primary} / {suffix}"),
        _ => simplified_primary,
    }
}

pub fn detect_wsl() -> Option<&'static str> {
    if std::env::var("WSL_DISTRO_NAME").is_ok() {
        return Some(wsl_flavor());
    }
    if let Ok(release) = std::fs::read_to_string("/proc/sys/kernel/osrelease") {
        let lower = release.to_ascii_lowercase();
        if lower.contains(MICROSOFT) || lower.contains(WSL) {
            return Some(if lower.contains(WSL2) { "WSL2" } else { "WSL" });
        }
    }
    None
}

fn wsl_flavor() -> &'static str {
    if let Ok(release) = std::fs::read_to_string("/proc/sys/kernel/osrelease")
        && release.to_ascii_lowercase().contains(WSL2)
    {
        return "WSL2";
    }
    "WSL"
}

pub fn detect_platform_metadata() -> (Option<String>, Option<String>) {
    let info = os_info::get();

    match info.os_type() {
        os_info::Type::Windows => (
            Some("windows".to_string()),
            Some(windows_version_label(&info)),
        ),
        os_info::Type::Macos => (Some("macos".to_string()), Some(info.version().to_string())),
        _ => {
            let distro = info
                .codename()
                .map(|c| format!("{} {c}", info.os_type()))
                .or_else(detect_linux_distro_label)
                .unwrap_or_else(|| info.os_type().to_string());

            if let Some(flavor) = detect_wsl() {
                (
                    Some("linux".to_string()),
                    Some(format!("{distro} / {flavor}")),
                )
            } else {
                (Some("linux".to_string()), Some(distro))
            }
        }
    }
}

pub fn generate_id() -> Uuid {
    Uuid::now_v7()
}

pub fn now_timestamp() -> i64 {
    Utc::now().timestamp()
}

pub fn now() -> DateTime<Utc> {
    Utc::now()
}

pub fn format_timestamp(ts: i64) -> String {
    let dt = DateTime::<Utc>::from_timestamp(ts, 0).unwrap_or_else(Utc::now);
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

pub fn is_blank(s: Option<&str>) -> bool {
    match s {
        Some(s) => s.trim().is_empty(),
        None => true,
    }
}

pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    let mut end = max_len.saturating_sub(3);
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...", &s[..end])
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_generate_id() -> Result<()> {
        let id1 = generate_id();
        let id2 = generate_id();
        assert_ne!(id1, id2);
        assert!(!id1.is_nil());
        Ok(())
    }

    #[test]
    fn test_now_timestamp() -> Result<()> {
        let ts = now_timestamp();
        assert!(ts > 0);
        Ok(())
    }

    #[test]
    fn test_is_blank() -> Result<()> {
        assert!(is_blank(None));
        assert!(is_blank(Some("")));
        assert!(is_blank(Some("   ")));
        assert!(!is_blank(Some("hello")));
        Ok(())
    }

    #[test]
    fn test_truncate() -> Result<()> {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("", 10), "");
        Ok(())
    }

    #[test]
    fn test_simplify_platform_title_keeps_suffix() -> Result<()> {
        assert_eq!(
            simplify_platform_title("Debian GNU/Linux 12 (bookworm) / WSL2"),
            "Debian 12 / WSL2"
        );
        Ok(())
    }

    #[test]
    fn test_simplify_platform_title_for_ubuntu() -> Result<()> {
        assert_eq!(
            simplify_platform_title("Ubuntu 24.04.2 LTS / WSL2"),
            "Ubuntu 24.04.2 / WSL2"
        );
        Ok(())
    }

    #[test]
    fn test_simplify_platform_title_for_opensuse() -> Result<()> {
        assert_eq!(
            simplify_platform_title("openSUSE Tumbleweed GNU/Linux"),
            "openSUSE Tumbleweed"
        );
        Ok(())
    }

    #[test]
    fn test_format_timestamp() -> Result<()> {
        let ts: i64 = 1700000000;
        let formatted = format_timestamp(ts);
        assert!(formatted.contains("2023"));
        assert!(formatted.ends_with("UTC"));
        Ok(())
    }

    #[test]
    fn test_format_timestamp_zero() -> Result<()> {
        let formatted = format_timestamp(0);
        assert!(formatted.contains("1970"));
        Ok(())
    }

    #[test]
    fn test_truncate_exact_boundary() -> Result<()> {
        assert_eq!(truncate("abcde", 5), "abcde");
        Ok(())
    }

    #[test]
    fn test_truncate_unicode() -> Result<()> {
        let result = truncate("你好世界hello", 10);
        assert!(result.ends_with("..."));
        assert!(result.len() < "你好世界hello".len());
        Ok(())
    }

    #[test]
    fn test_simplify_platform_title_empty() -> Result<()> {
        assert_eq!(simplify_platform_title(""), "");
        assert_eq!(simplify_platform_title("   "), "");
        Ok(())
    }
}
