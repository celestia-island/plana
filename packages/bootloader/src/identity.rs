//! Instance identity — the `celestia-XXX` id that names containers, shifts
//! ports, and labels the evernight node.
//!
//! The id is a `u16` in `0..=999`. On first boot it is generated at random;
//! thereafter it is persisted (by the caller, via [`InstanceIdentity::save`])
//! so the same id is reused across restarts. The caller picks the persistence
//! path — for evernight that is `~/.config/evernight/config.toml`, for the
//! entelecheia CLI it is the existing prefix mechanism.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Maximum instance id (inclusive). Three digits → 0..=999.
pub const MAX_INSTANCE_ID: u16 = 999;

/// An instance identity bundles the numeric id with the derived names evernight
/// and the container stack both consume.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceIdentity {
    pub id: u16,
}

impl InstanceIdentity {
    /// Wrap an existing id. Returns an error when the id is out of range.
    pub fn new(id: u16) -> Result<Self> {
        if id > MAX_INSTANCE_ID {
            bail!("instance id {id} out of range (0..={MAX_INSTANCE_ID})");
        }
        Ok(Self { id })
    }

    /// Generate a fresh random id in `0..=999`.
    pub fn generate() -> Self {
        let id: u16 = rand::thread_rng().gen_range(0..=MAX_INSTANCE_ID);
        Self { id }
    }

    /// The human-readable node id, e.g. `celestia-042`.
    pub fn node_id(&self) -> String {
        node_id_for(self.id)
    }

    /// The container-name prefix, e.g. `e-042-` (note the trailing dash).
    /// Concatenating this with `postgres` / `scepter` / `registry` yields the
    /// full container names `e-042-postgres` etc.
    pub fn container_prefix(&self) -> String {
        container_prefix_for(self.id)
    }

    /// Port offset applied to the configured base ports. We use the raw id so
    /// two instances never collide (instance 0 → base, instance 42 → base+42).
    pub fn port_offset(&self) -> u16 {
        self.id
    }

    /// Persist the identity to `path` as a minimal TOML document. The file is
    /// merged by hand (we only own the `[instance]` table) so the caller can
    /// keep other keys — node_id, socket, bootloader config — in the same file.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        // Read existing file (if any) and replace/insert the [instance] table
        // without touching unrelated keys. This keeps the file stable across
        // rewrites and avoids clobbering node_id / socket / bootloader config.
        let existing = std::fs::read_to_string(path).unwrap_or_default();
        let updated = replace_or_insert_instance_section(&existing, self.id);
        std::fs::write(path, updated)
            .with_context(|| format!("failed to write {}", path.display()))?;
        Ok(())
    }

    /// Load the identity from `path`. Returns `None` when the file or the
    /// `[instance]` table is absent — the caller then generates and saves one.
    pub fn load(path: &Path) -> Result<Option<Self>> {
        let Ok(text) = std::fs::read_to_string(path) else {
            return Ok(None);
        };
        let parsed: IdentityFile =
            toml::from_str(&text).context("failed to parse identity file")?;
        match parsed.instance {
            Some(row) if row.id <= MAX_INSTANCE_ID => Ok(Some(Self { id: row.id })),
            Some(row) => bail!(
                "instance id {} out of range (0..={})",
                row.id,
                MAX_INSTANCE_ID
            ),
            None => Ok(None),
        }
    }

    /// Load if present, otherwise generate, persist, and return. The standard
    /// first-boot path used by evernight's `supervise` command.
    pub fn load_or_create(path: &Path) -> Result<Self> {
        if let Some(existing) = Self::load(path)? {
            return Ok(existing);
        }
        let fresh = Self::generate();
        fresh.save(path)?;
        Ok(fresh)
    }
}

#[derive(Debug, Deserialize)]
struct IdentityFile {
    #[serde(default)]
    instance: Option<InstanceRow>,
}

#[derive(Debug, Deserialize)]
struct InstanceRow {
    id: u16,
}

/// Format the node id for a raw numeric id (`celestia-042`).
pub fn node_id_for(id: u16) -> String {
    format!("celestia-{id:03}")
}

/// Format the container-name prefix for a raw numeric id (`e-042-`).
pub fn container_prefix_for(id: u16) -> String {
    format!("e-{id:03}-")
}

/// Generate a random instance id in `0..=999`.
pub fn generate_instance_id() -> u16 {
    InstanceIdentity::generate().id
}

/// Replace the `[instance]` table in `text`, or append one if absent. Other
/// sections are preserved verbatim so we never lose caller-owned keys.
fn replace_or_insert_instance_section(text: &str, id: u16) -> String {
    let new_block = format!("[instance]\nid = {id}\n");

    // Match an existing [instance] header and rewrite through to the next
    // top-level section header (line starting with `[` at col 0).
    let mut out = String::new();
    let mut replaced = false;
    let mut in_instance = false;
    for line in text.lines() {
        if line.trim_start().starts_with('[') {
            // Entering a new section — flush the rewritten block if we were
            // inside [instance], and decide whether this header is [instance].
            if in_instance {
                in_instance = false;
            }
            if line.trim() == "[instance]" {
                replaced = true;
                in_instance = true;
                out.push_str(&new_block);
                continue;
            }
        }
        if in_instance {
            // Drop the old instance row(s); already rewritten above.
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    if !replaced {
        // No existing table — ensure there's a blank-line separator before
        // appending, but never double up leading newlines on an empty file.
        if !out.is_empty() && !out.ends_with("\n\n") {
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push('\n');
        }
        out.push_str(&new_block);
    }
    out
}

/// Configuration for writing the endpoint-discovery file
/// (`~/.config/celestia/instance.toml`). Consumed by shittim-chest WebUI/Tauri,
/// scriptum CLI, and any other client that needs to auto-discover a running
/// celestia-XXX instance.
#[derive(Debug, Clone)]
pub struct InstanceEndpointConfig {
    pub id: u16,
    pub scepter_port: u32,
    pub repo_root: String,
    pub mounted_projects: Vec<String>,
}

/// Write `~/.config/celestia/instance.toml` with scepter endpoint and project
/// mount information. Returns the path that was written.
///
/// Called by:
/// - `evernight supervise` on startup (inside WSL2/VM instance)
/// - `celestia-init.sh` on first init
/// - `celestia-install.ps1` after instance setup
///
/// Consumers:
/// - shittim-chest `useInstanceDiscovery` (Windows reads `\\wsl$\{instance}\...`)
/// - shittim-chest Tauri desktop (reads local file + cross-validates with evernight config)
/// - scriptum CLI (reads local `~/.config/celestia/instance.toml`)
pub fn write_instance_toml(config: &InstanceEndpointConfig) -> Result<PathBuf, anyhow::Error> {
    let base_dir = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    write_instance_toml_at(config, &base_dir)
}

/// Write instance.toml under a specific config base directory (for testing).
pub fn write_instance_toml_at(
    config: &InstanceEndpointConfig,
    config_base_dir: &Path,
) -> Result<PathBuf, anyhow::Error> {
    let toml_path = config_base_dir.join("celestia").join("instance.toml");

    if let Some(parent) = toml_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let name = node_id_for(config.id);
    let mounted = config
        .mounted_projects
        .iter()
        .map(|p| format!("\"{p}\""))
        .collect::<Vec<_>>()
        .join(", ");

    let content = format!(
        r#"[instance]
id = {id}
name = "{name}"

[scepter]
host = "localhost"
port = {port}
health_url = "http://localhost:{port}/health"

[projects]
root = "{repo_root}"
mounted = [{mounted}]
"#,
        id = config.id,
        name = name,
        port = config.scepter_port,
        repo_root = config.repo_root,
        mounted = mounted,
    );

    std::fs::write(&toml_path, &content)
        .with_context(|| format!("failed to write {}", toml_path.display()))?;

    Ok(toml_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn generates_in_range() {
        for _ in 0..1000 {
            let id = generate_instance_id();
            assert!(id <= MAX_INSTANCE_ID);
        }
    }

    #[test]
    fn formats_three_digits() {
        assert_eq!(node_id_for(0), "celestia-000");
        assert_eq!(node_id_for(42), "celestia-042");
        assert_eq!(node_id_for(999), "celestia-999");
        assert_eq!(container_prefix_for(7), "e-007-");
    }

    #[test]
    fn save_then_load_round_trips() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path: PathBuf = tmp.path().to_path_buf();
        let id = InstanceIdentity::new(123).unwrap();
        id.save(&path).unwrap();
        let loaded = InstanceIdentity::load(&path).unwrap().unwrap();
        assert_eq!(loaded.id, 123);
    }

    #[test]
    fn load_or_create_persists_across_calls() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path: PathBuf = tmp.path().to_path_buf();
        let first = InstanceIdentity::load_or_create(&path).unwrap();
        let second = InstanceIdentity::load_or_create(&path).unwrap();
        assert_eq!(first.id, second.id, "second call must reuse persisted id");
    }

    #[test]
    fn save_preserves_other_sections() {
        let existing = "node_id = \"old\"\nsocket = \"/tmp/x\"\n\n[instance]\nid = 5\n\n[bootloader]\nenabled = true\n";
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), existing).unwrap();
        InstanceIdentity::new(99).unwrap().save(tmp.path()).unwrap();
        let after = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(after.contains("node_id = \"old\""));
        assert!(after.contains("[bootloader]"));
        assert!(after.contains("enabled = true"));
        assert!(after.contains("[instance]\nid = 99"));
        // Old id 5 must be gone.
        assert!(!after.contains("id = 5\n"));
    }

    #[test]
    fn load_missing_file_is_none() {
        let path = PathBuf::from("/nonexistent/celestia/missing.toml");
        assert!(InstanceIdentity::load(&path).unwrap().is_none());
    }

    #[test]
    fn out_of_range_rejected() {
        assert!(InstanceIdentity::new(1000).is_err());
    }

    // -- instance_toml tests --

    #[test]
    fn write_instance_toml_generates_valid_file() {
        let tmp = tempfile::tempdir().unwrap();

        let config = InstanceEndpointConfig {
            id: 42,
            scepter_port: 8424,
            repo_root: "/celestia".into(),
            mounted_projects: vec!["entelecheia".into(), "shittim-chest".into(), "arona".into()],
        };

        let path = write_instance_toml_at(&config, tmp.path()).unwrap();
        assert!(path.exists());

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("id = 42"));
        assert!(content.contains("celestia-042"));
        assert!(content.contains("host = \"localhost\""));
        assert!(content.contains("port = 8424"));
        assert!(content.contains("http://localhost:8424/health"));
        assert!(content.contains("root = \"/celestia\""));
        assert!(content.contains("\"entelecheia\", \"shittim-chest\", \"arona\""));
    }

    #[test]
    fn write_instance_toml_is_valid_toml() {
        let tmp = tempfile::tempdir().unwrap();

        let config = InstanceEndpointConfig {
            id: 7,
            scepter_port: 8424 + 7 * 100,
            repo_root: "/home/user/celestia".into(),
            mounted_projects: vec![],
        };
        let path = write_instance_toml_at(&config, tmp.path()).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: toml::Value = toml::from_str(&content).expect("must be valid TOML");
        assert_eq!(parsed["instance"]["id"].as_integer(), Some(7));
        assert_eq!(
            parsed["scepter"]["health_url"].as_str(),
            Some("http://localhost:9124/health")
        );
        assert_eq!(parsed["projects"]["mounted"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn write_instance_toml_port_offset_formula() {
        let tmp = tempfile::tempdir().unwrap();

        for id in [0u32, 1, 42, 128, 999] {
            let scepter_port = 8424u32 + id * 100;
            let config = InstanceEndpointConfig {
                id: id as u16,
                scepter_port,
                repo_root: "/celestia".into(),
                mounted_projects: vec![],
            };
            let path = write_instance_toml_at(&config, tmp.path()).unwrap();
            let content = std::fs::read_to_string(&path).unwrap();
            assert!(
                content.contains(&format!("port = {scepter_port}")),
                "id={id} should produce port={scepter_port}"
            );
        }
    }
}
