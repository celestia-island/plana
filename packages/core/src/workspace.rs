use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const WORKSPACE_UUID_NAMESPACE: Uuid = Uuid::from_u128(0x3a7bc1d2_e4f56789_0abcdef0_12345678);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceConnectionKind {
    LocalFilesystem,
    DockerVolume,
    PolemosRemote,
    GitRemote,
    NoaRemote,
    /// Ephemeral scratch workspace — empty dir created on-demand.
    /// Stored under `~/.local/share/entelecheia/ephemeral/{uuid}/`.
    Ephemeral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WritebackMode {
    /// All changes are written back to the host workspace (default).
    ReadWrite,
    /// Files are copied into the container; changes stay in the container's
    /// ephemeral layer and never touch the host. Used for benchmarks and
    /// testing. The user can manually promote changes via a writeback command.
    Ephemeral,
}

impl Default for WritebackMode {
    fn default() -> Self {
        Self::ReadWrite
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceIdentity {
    pub path: String,
    pub connection_kind: WorkspaceConnectionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Uuid>,
}

impl WorkspaceIdentity {
    pub fn local(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            connection_kind: WorkspaceConnectionKind::LocalFilesystem,
            host_id: None,
            remote_address: None,
            user_id: None,
        }
    }

    pub fn volume(volume_name: impl Into<String>) -> Self {
        Self {
            path: volume_name.into(),
            connection_kind: WorkspaceConnectionKind::DockerVolume,
            host_id: None,
            remote_address: None,
            user_id: None,
        }
    }

    pub fn polemos(
        path: impl Into<String>,
        host_id: impl Into<String>,
        remote_address: Option<String>,
    ) -> Self {
        Self {
            path: path.into(),
            connection_kind: WorkspaceConnectionKind::PolemosRemote,
            host_id: Some(host_id.into()),
            remote_address,
            user_id: None,
        }
    }

    pub fn git_remote(repo_url: impl Into<String>) -> Self {
        Self {
            path: repo_url.into(),
            connection_kind: WorkspaceConnectionKind::GitRemote,
            host_id: None,
            remote_address: None,
            user_id: None,
        }
    }

    pub fn ephemeral(uuid: Uuid) -> Self {
        Self {
            path: uuid.to_string(),
            connection_kind: WorkspaceConnectionKind::Ephemeral,
            host_id: None,
            remote_address: None,
            user_id: None,
        }
    }

    pub fn noa_remote(remote_name: impl Into<String>, path: impl Into<String>) -> Self {
        let name = remote_name.into();
        Self {
            path: format!("{}/{}", name, path.into()),
            connection_kind: WorkspaceConnectionKind::NoaRemote,
            host_id: Some(name),
            remote_address: None,
            user_id: None,
        }
    }

    pub fn default_workspace() -> Self {
        Self {
            path: "/workspace".to_string(),
            connection_kind: WorkspaceConnectionKind::LocalFilesystem,
            host_id: None,
            remote_address: None,
            user_id: None,
        }
    }

    pub fn with_user_id(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn to_uri(&self) -> String {
        match self.connection_kind {
            WorkspaceConnectionKind::LocalFilesystem => {
                let path = ensure_leading_slash(&self.path);
                format!("local://{path}")
            }
            WorkspaceConnectionKind::DockerVolume => {
                let path = ensure_leading_slash(&self.path);
                format!("volume://{path}")
            }
            WorkspaceConnectionKind::PolemosRemote => {
                let authority = self.host_id.as_deref().unwrap_or("unknown");
                let path = self.path.trim_start_matches('/');
                let base = format!("ssh://{authority}/{path}");
                match &self.remote_address {
                    Some(addr) => format!("{base}?addr={addr}"),
                    None => base,
                }
            }
            WorkspaceConnectionKind::GitRemote => {
                format!("git://{}", self.path)
            }
            WorkspaceConnectionKind::NoaRemote => {
                format!("noa://{}", self.path)
            }
            WorkspaceConnectionKind::Ephemeral => {
                format!("ephemeral://{}", self.path)
            }
        }
    }

    pub fn from_uri(uri: &str) -> Result<Self> {
        let (scheme, rest) = uri
            .split_once("://")
            .with_context(|| format!("missing scheme separator '://' in URI: {}", uri))?;

        match scheme {
            "local" => {
                let path = ensure_leading_slash(rest);
                if path == "/" {
                    bail!("empty path in URI: {}", uri);
                }
                Ok(Self::local(path))
            }
            "volume" => {
                let name = rest.trim_start_matches('/');
                if name.is_empty() {
                    bail!("empty path in URI: {}", uri);
                }
                Ok(Self::volume(name))
            }
            "ssh" => {
                let (authority, path_and_query) = rest
                    .split_once('/')
                    .with_context(|| format!("SSH URI requires an authority (host): {}", uri))?;
                if authority.is_empty() {
                    bail!("SSH URI requires an authority (host): {}", uri);
                }
                let path = format!("/{}", path_and_query);
                let (path, remote_address) = if let Some((p, addr)) = path.split_once("?addr=") {
                    (p.to_string(), Some(addr.to_string()))
                } else {
                    (path, None)
                };
                Ok(Self::polemos(path, authority, remote_address))
            }
            "git" => {
                if rest.is_empty() {
                    bail!("empty path in URI: {}", uri);
                }
                Ok(Self::git_remote(rest))
            }
            "noa" => {
                if rest.is_empty() {
                    bail!("empty path in URI: {}", uri);
                }
                let (remote_name, path) = rest
                    .split_once('/')
                    .with_context(|| format!("empty path in URI: {}", uri))?;
                if remote_name.is_empty() || path.is_empty() {
                    bail!("empty path in URI: {}", uri);
                }
                Ok(Self::noa_remote(remote_name, path))
            }
            "ephemeral" => {
                if rest.is_empty() || rest == "auto" {
                    return Ok(Self::ephemeral(Uuid::now_v7()));
                }
                match Uuid::parse_str(rest) {
                    Ok(uuid) => Ok(Self::ephemeral(uuid)),
                    Err(_) => Ok(Self::ephemeral(Uuid::now_v7())),
                }
            }
            _ => bail!(
                "unknown workspace scheme '{}'; expected local|volume|ssh|git|noa|ephemeral",
                scheme
            ),
        }
    }

    pub fn compute_uuid(&self) -> Uuid {
        let canonical_path = if cfg!(windows) {
            self.path.replace('\\', "/")
        } else {
            self.path.clone()
        };

        let kind_str = match self.connection_kind {
            WorkspaceConnectionKind::LocalFilesystem => "local",
            WorkspaceConnectionKind::DockerVolume => "volume",
            WorkspaceConnectionKind::PolemosRemote => "polemos",
            WorkspaceConnectionKind::GitRemote => "git",
            WorkspaceConnectionKind::NoaRemote => "noa",
            WorkspaceConnectionKind::Ephemeral => "ephemeral",
        };

        let mut payload = format!("{}:{}:", kind_str, canonical_path);
        if let Some(ref host) = self.host_id {
            payload.push_str(&format!("host={}", host));
        }
        if let Some(ref addr) = self.remote_address {
            payload.push_str(&format!("addr={}", addr));
        }
        if let Some(ref uid) = self.user_id {
            payload.push_str(&format!("user={}", uid));
        }

        Uuid::new_v5(&WORKSPACE_UUID_NAMESPACE, payload.as_bytes())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceDescriptor {
    pub id: Uuid,
    pub identity: WorkspaceIdentity,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub resolved_path: Option<String>,
    #[serde(default = "default_merge_dirs")]
    pub merge_dirs: Vec<String>,
    #[serde(default)]
    pub allow_cross_ws: bool,
    /// Only meaningful when `connection_kind == LocalFilesystem`.
    /// `Ephemeral` means files are copied into the container's writable layer
    /// and never written back to the host. Default: `ReadWrite`.
    #[serde(default)]
    pub writeback_mode: WritebackMode,
}

fn default_merge_dirs() -> Vec<String> {
    vec!["/".to_string()]
}

impl WorkspaceDescriptor {
    pub fn from_identity(identity: WorkspaceIdentity) -> Self {
        let id = identity.compute_uuid();
        let display_name = Self::infer_display_name(&identity);
        Self {
            id,
            identity,
            display_name,
            resolved_path: None,
            merge_dirs: default_merge_dirs(),
            allow_cross_ws: false,
            writeback_mode: WritebackMode::default(),
        }
    }

    fn infer_display_name(identity: &WorkspaceIdentity) -> Option<String> {
        let path = &identity.path;
        match identity.connection_kind {
            WorkspaceConnectionKind::LocalFilesystem | WorkspaceConnectionKind::PolemosRemote => {
                std::path::Path::new(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
            }
            WorkspaceConnectionKind::DockerVolume => Some(format!("vol:{}", path)),
            WorkspaceConnectionKind::GitRemote => path
                .rsplit('/')
                .next()
                .map(|s| s.strip_suffix(".git").unwrap_or(s).to_string()),
            WorkspaceConnectionKind::NoaRemote => path.rsplit('/').next().map(|s| s.to_string()),
            WorkspaceConnectionKind::Ephemeral => {
                Some(format!("ephemeral-{}", &path[..8.min(path.len())]))
            }
        }
    }

    pub fn container_prefix(&self) -> String {
        format!("e-{}", &self.id.to_string()[..8])
    }

    pub fn short_id(&self) -> String {
        let hex = self.id.as_hyphenated().to_string();
        hex[hex.len() - 6..].to_string()
    }
}

#[derive(Debug, Clone, Default)]
pub struct WorkspaceRegistry {
    workspaces: Vec<WorkspaceDescriptor>,
}

impl WorkspaceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, descriptor: WorkspaceDescriptor) {
        if !self.workspaces.iter().any(|w| w.id == descriptor.id) {
            self.workspaces.push(descriptor);
        }
    }

    pub fn unregister(&mut self, id: Uuid) {
        self.workspaces.retain(|w| w.id != id);
    }

    pub fn get(&self, id: Uuid) -> Option<&WorkspaceDescriptor> {
        self.workspaces.iter().find(|w| w.id == id)
    }

    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut WorkspaceDescriptor> {
        self.workspaces.iter_mut().find(|w| w.id == id)
    }

    pub fn list(&self) -> &[WorkspaceDescriptor] {
        &self.workspaces
    }

    pub fn find_by_path(
        &self,
        path: &str,
        kind: WorkspaceConnectionKind,
    ) -> Option<&WorkspaceDescriptor> {
        self.workspaces
            .iter()
            .find(|w| w.identity.path == path && w.identity.connection_kind == kind)
    }
}

fn ensure_leading_slash(s: &str) -> String {
    if s.starts_with('/') {
        s.to_string()
    } else {
        format!("/{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_local_identity_uuid() -> Result<()> {
        let a = WorkspaceIdentity::local("/home/user/project");
        let b = WorkspaceIdentity::local("/home/user/project");
        let c = WorkspaceIdentity::local("/home/user/other");

        assert_eq!(a.compute_uuid(), b.compute_uuid());
        assert_ne!(a.compute_uuid(), c.compute_uuid());
        Ok(())
    }

    #[test]
    fn test_volume_vs_local_uuid() -> Result<()> {
        let a = WorkspaceIdentity::volume("my-workspace-data");
        let b = WorkspaceIdentity::local("my-workspace-data");

        assert_ne!(a.compute_uuid(), b.compute_uuid());
        Ok(())
    }

    #[test]
    fn test_polemos_uuid_includes_host() -> Result<()> {
        let a = WorkspaceIdentity::polemos("/opt/project", "host-a", None);
        let b = WorkspaceIdentity::polemos("/opt/project", "host-b", None);

        assert_ne!(a.compute_uuid(), b.compute_uuid());
        Ok(())
    }

    #[test]
    fn test_uuid_is_v7() -> Result<()> {
        let id = WorkspaceIdentity::local("/home/user/test").compute_uuid();
        assert_eq!(id.get_version(), Some(uuid::Version::Sha1));
        Ok(())
    }

    #[test]
    fn test_descriptor_container_prefix() -> Result<()> {
        let desc = WorkspaceDescriptor::from_identity(WorkspaceIdentity::local("/home/user/proj"));
        let prefix = desc.container_prefix();
        assert!(prefix.starts_with("e-"));
        assert_eq!(prefix.len(), 10);
        Ok(())
    }

    #[test]
    fn test_display_name_inference() -> Result<()> {
        let desc =
            WorkspaceDescriptor::from_identity(WorkspaceIdentity::local("/home/user/myproject"));
        assert_eq!(desc.display_name.as_deref(), Some("myproject"));

        let vol_desc = WorkspaceDescriptor::from_identity(WorkspaceIdentity::volume("app-data"));
        assert_eq!(vol_desc.display_name.as_deref(), Some("vol:app-data"));

        let git_desc = WorkspaceDescriptor::from_identity(WorkspaceIdentity::git_remote(
            "https://github.com/org/repo.git",
        ));
        assert_eq!(git_desc.display_name.as_deref(), Some("repo"));
        Ok(())
    }

    #[test]
    fn test_registry_dedup() -> Result<()> {
        let mut reg = WorkspaceRegistry::new();

        reg.register(WorkspaceDescriptor::from_identity(
            WorkspaceIdentity::local("/home/user/proj"),
        ));
        reg.register(WorkspaceDescriptor::from_identity(
            WorkspaceIdentity::local("/home/user/proj"),
        ));
        assert_eq!(reg.list().len(), 1);

        reg.register(WorkspaceDescriptor::from_identity(
            WorkspaceIdentity::local("/home/user/other"),
        ));
        assert_eq!(reg.list().len(), 2);

        let id = WorkspaceIdentity::local("/home/user/proj").compute_uuid();
        assert!(reg.get(id).is_some());
        reg.unregister(id);
        assert_eq!(reg.list().len(), 1);
        Ok(())
    }

    #[test]
    fn test_default_workspace_identity_deterministic() -> Result<()> {
        let a = WorkspaceIdentity::default_workspace();
        let b = WorkspaceIdentity::default_workspace();
        assert_eq!(a.compute_uuid(), b.compute_uuid());

        let c = WorkspaceIdentity::local("/workspace");
        assert_eq!(a.compute_uuid(), c.compute_uuid());
        Ok(())
    }

    #[test]
    fn test_default_workspace_produces_valid_uuid() -> Result<()> {
        let id = WorkspaceIdentity::default_workspace().compute_uuid();
        assert_eq!(id.get_version(), Some(uuid::Version::Sha1));
        let desc = WorkspaceDescriptor::from_identity(WorkspaceIdentity::default_workspace());
        assert_eq!(desc.id, id);
        Ok(())
    }

    #[test]
    fn test_default_workspace_registerable() -> Result<()> {
        let mut reg = WorkspaceRegistry::new();
        let desc = WorkspaceDescriptor::from_identity(WorkspaceIdentity::default_workspace());
        let id = desc.id;
        reg.register(desc);
        assert_eq!(reg.list().len(), 1);
        assert!(reg.get(id).is_some());
        Ok(())
    }

    #[test]
    fn test_local_to_uri() -> Result<()> {
        let id = WorkspaceIdentity::local("/mnt/sdb1/entelecheia");
        assert_eq!(id.to_uri(), "local:///mnt/sdb1/entelecheia");
        Ok(())
    }

    #[test]
    fn test_volume_to_uri() -> Result<()> {
        let id = WorkspaceIdentity::volume("ws-data-abc");
        assert_eq!(id.to_uri(), "volume:///ws-data-abc");
        Ok(())
    }

    #[test]
    fn test_polemos_to_uri_no_addr() -> Result<()> {
        let id = WorkspaceIdentity::polemos("/opt/proj", "gpu-node", None);
        assert_eq!(id.to_uri(), "ssh://gpu-node/opt/proj");
        Ok(())
    }

    #[test]
    fn test_polemos_to_uri_with_addr() -> Result<()> {
        let id = WorkspaceIdentity::polemos(
            "/opt/proj",
            "deploy@gpu-node",
            Some("192.168.1.100:22".into()),
        );
        assert_eq!(
            id.to_uri(),
            "ssh://deploy@gpu-node/opt/proj?addr=192.168.1.100:22"
        );
        Ok(())
    }

    #[test]
    fn test_git_to_uri() -> Result<()> {
        let id = WorkspaceIdentity::git_remote("https://github.com/org/repo.git");
        assert_eq!(id.to_uri(), "git://https://github.com/org/repo.git");
        Ok(())
    }

    #[test]
    fn test_roundtrip_local() -> Result<()> {
        let original = WorkspaceIdentity::local("/mnt/sdb1/entelecheia");
        let uri = original.to_uri();
        let parsed = WorkspaceIdentity::from_uri(&uri)?;
        assert_eq!(parsed.path, original.path);
        assert_eq!(parsed.connection_kind, original.connection_kind);
        Ok(())
    }

    #[test]
    fn test_roundtrip_volume() -> Result<()> {
        let original = WorkspaceIdentity::volume("ws-data-abc");
        let uri = original.to_uri();
        let parsed = WorkspaceIdentity::from_uri(&uri)?;
        assert_eq!(parsed.path, original.path);
        assert_eq!(parsed.connection_kind, original.connection_kind);
        Ok(())
    }

    #[test]
    fn test_roundtrip_polemos_no_addr() -> Result<()> {
        let original = WorkspaceIdentity::polemos("/opt/proj", "gpu-node", None);
        let uri = original.to_uri();
        let parsed = WorkspaceIdentity::from_uri(&uri)?;
        assert_eq!(parsed.path, original.path);
        assert_eq!(parsed.host_id, original.host_id);
        assert_eq!(parsed.remote_address, None);
        Ok(())
    }

    #[test]
    fn test_roundtrip_polemos_with_addr() -> Result<()> {
        let original = WorkspaceIdentity::polemos(
            "/opt/proj",
            "deploy@gpu-node",
            Some("192.168.1.100:22".into()),
        );
        let uri = original.to_uri();
        let parsed = WorkspaceIdentity::from_uri(&uri)?;
        assert_eq!(parsed.path, original.path);
        assert_eq!(parsed.host_id, original.host_id);
        assert_eq!(parsed.remote_address, Some("192.168.1.100:22".into()));
        Ok(())
    }

    #[test]
    fn test_roundtrip_git() -> Result<()> {
        let original = WorkspaceIdentity::git_remote("https://github.com/org/repo.git");
        let uri = original.to_uri();
        let parsed = WorkspaceIdentity::from_uri(&uri)?;
        assert_eq!(parsed.path, original.path);
        assert_eq!(parsed.connection_kind, WorkspaceConnectionKind::GitRemote);
        Ok(())
    }

    #[test]
    fn test_from_uri_rejects_bad_scheme() -> Result<()> {
        let err = WorkspaceIdentity::from_uri("ftp:///something").unwrap_err();
        assert!(err.to_string().contains("unknown workspace scheme"));
        Ok(())
    }

    #[test]
    fn test_from_uri_rejects_no_scheme() -> Result<()> {
        let err = WorkspaceIdentity::from_uri("/just/a/path").unwrap_err();
        assert!(err.to_string().contains("missing scheme separator"));
        Ok(())
    }

    #[test]
    fn test_default_workspace_to_uri() -> Result<()> {
        let id = WorkspaceIdentity::default_workspace();
        assert_eq!(id.to_uri(), "local:///workspace");
        Ok(())
    }

    #[test]
    fn test_workspace_descriptor_default_merge_dirs() -> Result<()> {
        let desc = WorkspaceDescriptor::from_identity(WorkspaceIdentity::local("/tmp/test"));
        assert_eq!(desc.merge_dirs, vec!["/".to_string()]);
        Ok(())
    }

    #[test]
    fn test_workspace_descriptor_merge_dirs_roundtrip() -> Result<()> {
        let mut desc = WorkspaceDescriptor::from_identity(WorkspaceIdentity::local("/tmp/proj"));
        desc.merge_dirs = vec!["/home".to_string(), "/data".to_string()];
        let json = serde_json::to_string(&desc)?;
        let restored: WorkspaceDescriptor = serde_json::from_str(&json)?;
        assert_eq!(
            restored.merge_dirs,
            vec!["/home".to_string(), "/data".to_string()]
        );
        Ok(())
    }

    #[test]
    fn test_workspace_descriptor_merge_dirs_absent_deserialize() -> Result<()> {
        let desc = WorkspaceDescriptor::from_identity(WorkspaceIdentity::local("/tmp/old"));
        let json = serde_json::to_string(&desc)?;
        let json_without = json.replace(&format!(",\"merge_dirs\":{:?}", desc.merge_dirs), "");
        let restored: WorkspaceDescriptor = serde_json::from_str(&json_without)?;
        assert_eq!(
            restored.merge_dirs,
            vec!["/".to_string()],
            "absent merge_dirs defaults to [\"/\"]"
        );
        Ok(())
    }

    #[test]
    fn test_noa_remote_construction() -> Result<()> {
        let id = WorkspaceIdentity::noa_remote("my-remote", "workspace-1");
        assert_eq!(id.path, "my-remote/workspace-1");
        assert_eq!(id.connection_kind, WorkspaceConnectionKind::NoaRemote);
        assert_eq!(id.host_id.as_deref(), Some("my-remote"));
        assert!(id.remote_address.is_none());
        Ok(())
    }

    #[test]
    fn test_noa_to_uri() -> Result<()> {
        let id = WorkspaceIdentity::noa_remote("my-remote", "workspace-1");
        assert_eq!(id.to_uri(), "noa://my-remote/workspace-1");
        Ok(())
    }

    #[test]
    fn test_roundtrip_noa() -> Result<()> {
        let original = WorkspaceIdentity::noa_remote("my-remote", "workspace-1");
        let uri = original.to_uri();
        let parsed = WorkspaceIdentity::from_uri(&uri)?;
        assert_eq!(parsed.path, original.path);
        assert_eq!(parsed.connection_kind, WorkspaceConnectionKind::NoaRemote);
        assert_eq!(parsed.host_id, original.host_id);
        Ok(())
    }

    #[test]
    fn test_noa_display_name() -> Result<()> {
        let desc = WorkspaceDescriptor::from_identity(WorkspaceIdentity::noa_remote(
            "my-remote",
            "workspace-1",
        ));
        assert_eq!(desc.display_name.as_deref(), Some("workspace-1"));
        Ok(())
    }

    #[test]
    fn test_noa_uuid_differs_from_git() -> Result<()> {
        let noa = WorkspaceIdentity::noa_remote("my-remote", "repo");
        let git = WorkspaceIdentity::git_remote("my-remote/repo");
        assert_ne!(noa.compute_uuid(), git.compute_uuid());
        Ok(())
    }

    #[test]
    fn test_from_uri_rejects_noa_without_path() -> Result<()> {
        let err = WorkspaceIdentity::from_uri("noa://onlyname").unwrap_err();
        assert!(
            err.to_string().contains("empty path"),
            "expected 'empty path' error, got: {}",
            err
        );
        Ok(())
    }

    #[test]
    fn test_open_command_uri_variants() -> Result<()> {
        let local = WorkspaceIdentity::from_uri("local:///home/user/proj")?;
        assert_eq!(
            local.connection_kind,
            WorkspaceConnectionKind::LocalFilesystem
        );

        let git = WorkspaceIdentity::from_uri("git://https://github.com/org/repo.git")?;
        assert_eq!(git.connection_kind, WorkspaceConnectionKind::GitRemote);

        let noa = WorkspaceIdentity::from_uri("noa://remote/workspace")?;
        assert_eq!(noa.connection_kind, WorkspaceConnectionKind::NoaRemote);

        let ssh = WorkspaceIdentity::from_uri("ssh://host/opt/proj?addr=1.2.3.4")?;
        assert_eq!(ssh.connection_kind, WorkspaceConnectionKind::PolemosRemote);
        Ok(())
    }

    #[test]
    fn test_user_id_changes_uuid() -> Result<()> {
        let user_a = Uuid::parse_str("11111111-1111-1111-1111-111111111111")?;
        let user_b = Uuid::parse_str("22222222-2222-2222-2222-222222222222")?;

        let ws_no_user = WorkspaceIdentity::local("/home/user/project");
        let ws_user_a = WorkspaceIdentity::local("/home/user/project").with_user_id(user_a);
        let ws_user_b = WorkspaceIdentity::local("/home/user/project").with_user_id(user_b);

        let uuid_none = ws_no_user.compute_uuid();
        let uuid_a = ws_user_a.compute_uuid();
        let uuid_b = ws_user_b.compute_uuid();

        assert_ne!(uuid_none, uuid_a, "no-user and user-a should differ");
        assert_ne!(uuid_none, uuid_b, "no-user and user-b should differ");
        assert_ne!(
            uuid_a, uuid_b,
            "different users should produce different UUIDs"
        );
        Ok(())
    }

    #[test]
    fn test_user_id_none_backward_compat() -> Result<()> {
        let old_style = WorkspaceIdentity::local("/home/user/project");
        let new_style_explicit = WorkspaceIdentity {
            path: "/home/user/project".to_string(),
            connection_kind: WorkspaceConnectionKind::LocalFilesystem,
            host_id: None,
            remote_address: None,
            user_id: None,
        };
        assert_eq!(old_style.compute_uuid(), new_style_explicit.compute_uuid());
        Ok(())
    }

    #[test]
    fn test_with_user_id_builder() -> Result<()> {
        let uid = Uuid::parse_str("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")?;
        let ws = WorkspaceIdentity::local("/proj").with_user_id(uid);
        assert_eq!(ws.user_id, Some(uid));
        assert_eq!(ws.path, "/proj");
        Ok(())
    }

    #[test]
    fn test_user_id_with_polemos() -> Result<()> {
        let uid = Uuid::parse_str("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")?;
        let ws = WorkspaceIdentity::polemos("/opt/proj", "host-a", None).with_user_id(uid);
        assert_eq!(ws.user_id, Some(uid));
        assert_eq!(ws.host_id.as_deref(), Some("host-a"));
        Ok(())
    }

    #[test]
    fn test_short_id() -> Result<()> {
        let desc = WorkspaceDescriptor::from_identity(WorkspaceIdentity::local("/home/user/proj"));
        let short = desc.short_id();
        assert_eq!(short.len(), 6, "short_id should be 6 hex characters");
        let full_hex = desc.id.as_hyphenated().to_string();
        assert!(
            full_hex.ends_with(&short),
            "short_id should be last 6 chars of UUID hex"
        );
        Ok(())
    }

    #[test]
    fn test_short_id_unique_across_users() -> Result<()> {
        let user_a = Uuid::parse_str("11111111-1111-1111-1111-111111111111")?;
        let user_b = Uuid::parse_str("22222222-2222-2222-2222-222222222222")?;
        let desc_a = WorkspaceDescriptor::from_identity(
            WorkspaceIdentity::local("/home/user/proj").with_user_id(user_a),
        );
        let desc_b = WorkspaceDescriptor::from_identity(
            WorkspaceIdentity::local("/home/user/proj").with_user_id(user_b),
        );
        assert_ne!(desc_a.short_id(), desc_b.short_id());
        Ok(())
    }

    #[test]
    fn test_user_id_serde_roundtrip() -> Result<()> {
        let uid = Uuid::parse_str("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")?;
        let ws = WorkspaceIdentity::local("/home/user/proj").with_user_id(uid);
        let json = serde_json::to_string(&ws)?;
        let restored: WorkspaceIdentity = serde_json::from_str(&json)?;
        assert_eq!(restored.user_id, Some(uid));
        assert_eq!(restored.path, "/home/user/proj");
        Ok(())
    }

    #[test]
    fn test_user_id_absent_deserialize() -> Result<()> {
        let json = r#"{"path":"/proj","connection_kind":"local_filesystem"}"#;
        let restored: WorkspaceIdentity = serde_json::from_str(json)?;
        assert!(restored.user_id.is_none());
        Ok(())
    }

    #[test]
    fn test_same_user_different_paths() -> Result<()> {
        let uid = Uuid::parse_str("11111111-1111-1111-1111-111111111111")?;
        let ws_a = WorkspaceIdentity::local("/proj-a").with_user_id(uid);
        let ws_b = WorkspaceIdentity::local("/proj-b").with_user_id(uid);
        assert_ne!(ws_a.compute_uuid(), ws_b.compute_uuid());
        Ok(())
    }

    #[test]
    fn test_same_path_different_connection_kind_with_user() -> Result<()> {
        let uid = Uuid::parse_str("11111111-1111-1111-1111-111111111111")?;
        let local = WorkspaceIdentity::local("/data/proj").with_user_id(uid);
        let volume = WorkspaceIdentity::volume("data/proj").with_user_id(uid);
        assert_ne!(local.compute_uuid(), volume.compute_uuid());
        Ok(())
    }

    #[test]
    fn test_noa_remote_with_user_id() -> Result<()> {
        let uid = Uuid::parse_str("11111111-1111-1111-1111-111111111111")?;
        let ws = WorkspaceIdentity::noa_remote("remote", "proj").with_user_id(uid);
        assert_eq!(ws.user_id, Some(uid));
        assert_eq!(ws.connection_kind, WorkspaceConnectionKind::NoaRemote);
        assert_eq!(ws.path, "remote/proj");
        let uuid_with = ws.compute_uuid();

        let ws_no_user = WorkspaceIdentity::noa_remote("remote", "proj");
        let uuid_without = ws_no_user.compute_uuid();
        assert_ne!(uuid_with, uuid_without);
        Ok(())
    }

    #[test]
    fn test_git_remote_with_user_id() -> Result<()> {
        let uid = Uuid::parse_str("11111111-1111-1111-1111-111111111111")?;
        let ws = WorkspaceIdentity::git_remote("https://github.com/org/repo.git").with_user_id(uid);
        assert_eq!(ws.user_id, Some(uid));
        assert_eq!(ws.connection_kind, WorkspaceConnectionKind::GitRemote);
        Ok(())
    }

    #[test]
    fn test_short_id_deterministic() -> Result<()> {
        let uid = Uuid::parse_str("11111111-1111-1111-1111-111111111111")?;
        let desc = WorkspaceDescriptor::from_identity(
            WorkspaceIdentity::local("/home/user/proj").with_user_id(uid),
        );
        assert_eq!(
            desc.short_id(),
            desc.short_id(),
            "short_id must be deterministic"
        );
        Ok(())
    }

    #[test]
    fn test_short_id_different_for_every_field_combo() -> Result<()> {
        let uid_a = Uuid::parse_str("11111111-1111-1111-1111-111111111111")?;
        let uid_b = Uuid::parse_str("22222222-2222-2222-2222-222222222222")?;
        let ids: Vec<Uuid> = vec![
            WorkspaceIdentity::local("/proj").compute_uuid(),
            WorkspaceIdentity::local("/proj")
                .with_user_id(uid_a)
                .compute_uuid(),
            WorkspaceIdentity::local("/proj")
                .with_user_id(uid_b)
                .compute_uuid(),
            WorkspaceIdentity::volume("proj").compute_uuid(),
            WorkspaceIdentity::volume("proj")
                .with_user_id(uid_a)
                .compute_uuid(),
            WorkspaceIdentity::git_remote("https://x.com/repo").compute_uuid(),
            WorkspaceIdentity::git_remote("https://x.com/repo")
                .with_user_id(uid_a)
                .compute_uuid(),
            WorkspaceIdentity::noa_remote("r", "p").compute_uuid(),
            WorkspaceIdentity::noa_remote("r", "p")
                .with_user_id(uid_a)
                .compute_uuid(),
        ];
        let short_ids: Vec<String> = ids
            .iter()
            .map(|id| {
                let hex = id.as_hyphenated().to_string();
                hex[hex.len() - 6..].to_string()
            })
            .collect();
        let unique: std::collections::HashSet<_> = short_ids.iter().collect();
        assert_eq!(
            unique.len(),
            ids.len(),
            "all 9 combinations should produce unique short_ids"
        );
        Ok(())
    }

    #[test]
    fn test_workspace_descriptor_display_name_no_user() -> Result<()> {
        let desc = WorkspaceDescriptor::from_identity(WorkspaceIdentity::local("/home/user/myapp"));
        assert_eq!(desc.display_name.as_deref(), Some("myapp"));
        Ok(())
    }

    #[test]
    fn test_workspace_descriptor_display_name_with_user() -> Result<()> {
        let uid = Uuid::parse_str("11111111-1111-1111-1111-111111111111")?;
        let desc = WorkspaceDescriptor::from_identity(
            WorkspaceIdentity::local("/home/user/myapp").with_user_id(uid),
        );
        assert_eq!(
            desc.display_name.as_deref(),
            Some("myapp"),
            "display_name should not depend on user_id"
        );
        Ok(())
    }

    #[test]
    fn test_container_prefix_different_with_user() -> Result<()> {
        let uid = Uuid::parse_str("11111111-1111-1111-1111-111111111111")?;
        let desc_no = WorkspaceDescriptor::from_identity(WorkspaceIdentity::local("/proj"));
        let desc_yes =
            WorkspaceDescriptor::from_identity(WorkspaceIdentity::local("/proj").with_user_id(uid));
        assert_ne!(desc_no.container_prefix(), desc_yes.container_prefix());
        Ok(())
    }

    #[test]
    fn test_user_id_with_polemos_and_remote_address() -> Result<()> {
        let uid = Uuid::parse_str("11111111-1111-1111-1111-111111111111")?;
        let ws = WorkspaceIdentity::polemos(
            "/opt/proj",
            "deploy@gpu-node",
            Some("192.168.1.100:22".into()),
        )
        .with_user_id(uid);
        assert_eq!(ws.user_id, Some(uid));
        assert_eq!(ws.host_id.as_deref(), Some("deploy@gpu-node"));
        assert_eq!(ws.remote_address.as_deref(), Some("192.168.1.100:22"));
        Ok(())
    }
}
