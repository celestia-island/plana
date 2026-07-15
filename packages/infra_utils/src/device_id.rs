use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

const DEVICE_UUID_NAMESPACE: Uuid = Uuid::from_bytes([
    0x6e, 0x74, 0x65, 0x6c, 0x65, 0x63, 0x68, 0x65, 0x69, 0x61, 0x2d, 0x64, 0x65, 0x76, 0x69, 0x63,
]);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFingerprint {
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub hostname: String,
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub total_memory_mb: u64,
    pub disk_serials: Vec<String>,
    pub primary_mac: String,
    pub machine_id: String,
}

impl DeviceFingerprint {
    pub fn collect() -> Self {
        Self {
            os_name: collect_os_name(),
            os_version: collect_os_version(),
            kernel_version: collect_kernel_version(),
            hostname: collect_hostname(),
            cpu_model: collect_cpu_model(),
            cpu_cores: collect_cpu_cores(),
            total_memory_mb: collect_total_memory(),
            disk_serials: collect_disk_serials(),
            primary_mac: collect_primary_mac(),
            machine_id: collect_machine_id(),
        }
    }

    pub fn fingerprint_text(&self) -> String {
        let mut parts = Vec::new();
        parts.push(format!("os={}/{}", self.os_name, self.os_version));
        parts.push(format!("kernel={}", self.kernel_version));
        parts.push(format!("host={}", self.hostname));
        parts.push(format!("cpu={}@{}", self.cpu_model, self.cpu_cores));
        parts.push(format!("mem={}mb", self.total_memory_mb));
        parts.push(format!("disks={}", self.disk_serials.join(",")));
        parts.push(format!("mac={}", self.primary_mac));
        parts.push(format!("mid={}", self.machine_id));
        parts.join("|")
    }

    pub fn device_uuid(&self) -> Uuid {
        Uuid::new_v5(&DEVICE_UUID_NAMESPACE, self.fingerprint_text().as_bytes())
    }
}

fn collect_os_name() -> String {
    if let Ok(s) = std::fs::read_to_string("/etc/os-release") {
        for line in s.lines() {
            if let Some(v) = line.strip_prefix("NAME=") {
                return v.trim_matches('"').to_string();
            }
        }
    }
    std::env::consts::OS.to_string()
}

fn collect_os_version() -> String {
    if let Ok(s) = std::fs::read_to_string("/etc/os-release") {
        for line in s.lines() {
            if let Some(v) = line.strip_prefix("VERSION_ID=") {
                return v.trim_matches('"').to_string();
            }
        }
    }
    String::new()
}

fn collect_kernel_version() -> String {
    std::process::Command::new("uname")
        .arg("-r")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn collect_hostname() -> String {
    if let Ok(h) = std::env::var("HOSTNAME") {
        return h;
    }
    std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn collect_cpu_model() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|s| {
            s.lines().find_map(|l| {
                l.strip_prefix("model name").and_then(|v| {
                    let v = v.trim_start_matches([':', ' ']);
                    if v.is_empty() {
                        None
                    } else {
                        Some(v.to_string())
                    }
                })
            })
        })
        .unwrap_or_default()
}

fn collect_cpu_cores() -> u32 {
    std::fs::read_to_string("/proc/cpuinfo")
        .map(|s| s.matches("processor\t:").count() as u32)
        .unwrap_or(1)
}

fn collect_total_memory() -> u64 {
    std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|s| {
            s.lines().find_map(|l| {
                l.strip_prefix("MemTotal:").and_then(|v| {
                    v.split_whitespace()
                        .next()
                        .and_then(|n| n.parse::<u64>().ok())
                        .map(|kb| kb / 1024)
                })
            })
        })
        .unwrap_or(0)
}

fn collect_disk_serials() -> Vec<String> {
    let mut serials: BTreeMap<String, ()> = BTreeMap::new();
    if let Ok(entries) = std::fs::read_dir("/dev/disk/by-id/") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("usb-") && !name.contains("part") {
                serials.insert(name, ());
            }
        }
    }
    serials.into_keys().collect()
}

fn collect_primary_mac() -> String {
    if let Ok(entries) = std::fs::read_dir("/sys/class/net/") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "lo" {
                continue;
            }
            let path = format!("/sys/class/net/{}/address", name);
            if let Ok(mac) = std::fs::read_to_string(&path) {
                let mac = mac.trim().to_string();
                if !mac.is_empty() && mac != "00:00:00:00:00:00" {
                    return mac;
                }
            }
        }
    }
    String::new()
}

fn collect_machine_id() -> String {
    for path in &["/etc/machine-id", "/var/lib/dbus/machine-id"] {
        if let Ok(id) = std::fs::read_to_string(path) {
            let id = id.trim().to_string();
            if !id.is_empty() {
                return id;
            }
        }
    }
    String::new()
}

pub fn get_device_uuid() -> Uuid {
    DeviceFingerprint::collect().device_uuid()
}
