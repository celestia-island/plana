use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
};

use plana_core::{CredentialError, CredentialStorage};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CredentialRecord {
    pub credential_type: String,
    pub provider: String,
    pub encrypted_value: String,
    pub metadata: HashMap<String, String>,
    pub auto_imported: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CredentialFile {
    pub version: String,
    pub credentials: HashMap<String, CredentialRecord>,
}

impl Default for CredentialFile {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            credentials: HashMap::new(),
        }
    }
}

/// Open `path` for writing as an owner-only (`0o600`) file.
///
/// The store holds provider credentials, so the file must never exist with a
/// broader mode than 0o600 — including the instant between the write and a
/// follow-up chmod. Hence the mode goes into the `open(2)` call itself
/// (credential hygiene, AGENTS.md red line 1).
async fn open_private_file(path: &Path) -> std::io::Result<fs::File> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    options.mode(0o600);
    options.open(path).await
}

pub struct FileCredentialStorage {
    credentials_dir: PathBuf,
    llm_credentials_file: PathBuf,
    ssh_credentials_file: PathBuf,
}

impl FileCredentialStorage {
    pub fn new() -> Result<Self> {
        let credentials_dir = Self::get_credentials_dir()?;

        std::fs::create_dir_all(&credentials_dir).with_context(|| {
            format!(
                "Failed to create credentials directory: {:?}",
                credentials_dir
            )
        })?;

        #[cfg(unix)]
        {
            if let Err(e) =
                std::fs::set_permissions(&credentials_dir, std::fs::Permissions::from_mode(0o700))
            {
                tracing::warn!(
                    error = %e,
                    path = %credentials_dir.display(),
                    "Failed to set restrictive permissions (0o700) on credentials directory — continuing anyway (may be bind-mount or permission restriction)"
                );
            }
        }

        let llm_credentials_file = credentials_dir.join("llm_providers.json");
        let ssh_credentials_file = credentials_dir.join("ssh_keys.json");

        Ok(Self {
            credentials_dir,
            llm_credentials_file,
            ssh_credentials_file,
        })
    }

    fn get_credentials_dir() -> Result<PathBuf> {
        Ok(plana_config::UserConfig::config_dir().join("credentials"))
    }

    pub fn credentials_dir(&self) -> &Path {
        &self.credentials_dir
    }

    fn get_file_path(&self, credential_type: &str) -> &Path {
        match credential_type {
            "ssh_key" | "ssh_password" => &self.ssh_credentials_file,
            _ => &self.llm_credentials_file,
        }
    }

    async fn read_credential_file(&self, path: &Path) -> Result<CredentialFile> {
        if !path.exists() {
            return Ok(CredentialFile::default());
        }

        let mut file = fs::File::open(path)
            .await
            .with_context(|| format!("Failed to open credential file: {:?}", path))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .with_context(|| "Failed to read credential file")?;

        let credential_file: CredentialFile =
            serde_json::from_str(&contents).with_context(|| "Failed to parse credential file")?;

        Ok(credential_file)
    }

    async fn write_credential_file(&self, path: &Path, file: &CredentialFile) -> Result<()> {
        let contents = serde_json::to_string_pretty(file)
            .with_context(|| "Failed to serialize credentials")?;

        let mut file_handle = open_private_file(path)
            .await
            .with_context(|| format!("Failed to create credential file: {:?}", path))?;

        // open(2) ignores `mode` for an existing file, so the descriptor is
        // fchmod'ed too: handle-scoped (no path can be swapped under it) and it
        // repairs a file an older release left at 0o644.
        #[cfg(unix)]
        {
            file_handle
                .set_permissions(std::fs::Permissions::from_mode(0o600))
                .await
                .with_context(
                    || "Failed to set restrictive permissions (0o600) on credential file",
                )?;
        }

        file_handle
            .write_all(contents.as_bytes())
            .await
            .with_context(|| "Failed to write credential file")?;

        Ok(())
    }

    pub async fn list_all_credentials(&self) -> Result<Vec<(String, CredentialRecord)>> {
        let mut all_credentials = Vec::new();

        if self.llm_credentials_file.exists() {
            let llm_file = self
                .read_credential_file(&self.llm_credentials_file)
                .await?;
            for (key, record) in llm_file.credentials {
                all_credentials.push((key, record));
            }
        }

        if self.ssh_credentials_file.exists() {
            let ssh_file = self
                .read_credential_file(&self.ssh_credentials_file)
                .await?;
            for (key, record) in ssh_file.credentials {
                all_credentials.push((key, record));
            }
        }

        Ok(all_credentials)
    }

    pub async fn export_all(&self) -> Result<CredentialFile> {
        let mut exported = CredentialFile::default();

        let all = self.list_all_credentials().await?;
        for (key, record) in all {
            exported.credentials.insert(key, record);
        }

        Ok(exported)
    }

    pub async fn import_all(&self, data: &CredentialFile) -> Result<u64> {
        let mut count = 0u64;

        for (key, record) in &data.credentials {
            self.save_with_auto_imported(key, &record.encrypted_value, record.auto_imported)
                .await?;
            count += 1;
        }

        Ok(count)
    }

    pub async fn save_with_auto_imported(
        &self,
        key: &str,
        encrypted_value: &str,
        auto_imported: bool,
    ) -> Result<()> {
        let (cred_type, provider) = parse_credential_key_for_storage(key)?;

        let file_path = self.get_file_path(cred_type);
        let mut file = self.read_credential_file(file_path).await?;

        let now = Utc::now();
        let record = if let Some(existing) = file.credentials.get(key) {
            CredentialRecord {
                credential_type: cred_type.to_string(),
                provider: provider.to_string(),
                encrypted_value: encrypted_value.to_string(),
                metadata: existing.metadata.clone(),
                auto_imported,
                created_at: existing.created_at,
                updated_at: now,
            }
        } else {
            CredentialRecord {
                credential_type: cred_type.to_string(),
                provider: provider.to_string(),
                encrypted_value: encrypted_value.to_string(),
                metadata: HashMap::new(),
                auto_imported,
                created_at: now,
                updated_at: now,
            }
        };

        file.credentials.insert(key.to_string(), record);

        self.write_credential_file(file_path, &file).await?;

        Ok(())
    }

    pub async fn is_auto_imported(&self, key: &str) -> Result<bool> {
        let (cred_type, _) = parse_credential_key_for_storage(key)?;

        let file_path = self.get_file_path(cred_type);
        let file = self.read_credential_file(file_path).await?;

        Ok(file
            .credentials
            .get(key)
            .map(|r| r.auto_imported)
            .unwrap_or(false))
    }
}

#[async_trait::async_trait]
impl CredentialStorage for FileCredentialStorage {
    async fn save(&self, key: &str, encrypted_value: &str) -> Result<(), CredentialError> {
        let (cred_type, provider) = parse_credential_key(key)?;

        let file_path = self.get_file_path(cred_type);
        let mut file = self
            .read_credential_file(file_path)
            .await
            .map_err(|e| CredentialError::StorageError(e.to_string()))?;

        let now = Utc::now();
        let record = if let Some(existing) = file.credentials.get(key) {
            CredentialRecord {
                credential_type: cred_type.to_string(),
                provider: provider.to_string(),
                encrypted_value: encrypted_value.to_string(),
                metadata: existing.metadata.clone(),
                auto_imported: existing.auto_imported,
                created_at: existing.created_at,
                updated_at: now,
            }
        } else {
            CredentialRecord {
                credential_type: cred_type.to_string(),
                provider: provider.to_string(),
                encrypted_value: encrypted_value.to_string(),
                metadata: HashMap::new(),
                auto_imported: false,
                created_at: now,
                updated_at: now,
            }
        };

        file.credentials.insert(key.to_string(), record);

        self.write_credential_file(file_path, &file)
            .await
            .map_err(|e| CredentialError::StorageError(e.to_string()))?;

        Ok(())
    }

    async fn load(&self, key: &str) -> Result<Option<String>, CredentialError> {
        let (cred_type, _) = parse_credential_key(key)?;

        let file_path = self.get_file_path(cred_type);
        let file = self
            .read_credential_file(file_path)
            .await
            .map_err(|e| CredentialError::StorageError(e.to_string()))?;

        Ok(file.credentials.get(key).map(|r| r.encrypted_value.clone()))
    }

    async fn delete(&self, key: &str) -> Result<bool, CredentialError> {
        let (cred_type, _) = parse_credential_key(key)?;

        let file_path = self.get_file_path(cred_type);
        let mut file = self
            .read_credential_file(file_path)
            .await
            .map_err(|e| CredentialError::StorageError(e.to_string()))?;

        let existed = file.credentials.remove(key).is_some();

        if existed {
            self.write_credential_file(file_path, &file)
                .await
                .map_err(|e| CredentialError::StorageError(e.to_string()))?;
        }

        Ok(existed)
    }

    async fn list_all(&self) -> Result<Vec<(String, String)>, CredentialError> {
        let all = self
            .list_all_credentials()
            .await
            .map_err(|e| CredentialError::StorageError(e.to_string()))?;

        Ok(all
            .into_iter()
            .map(|(key, record)| (key, record.encrypted_value))
            .collect())
    }
}

fn parse_credential_key(key: &str) -> Result<(&str, &str), CredentialError> {
    let parts: Vec<&str> = key.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(CredentialError::InvalidType(format!(
            "Invalid key format: {}",
            key
        )));
    }
    Ok((parts[0], parts[1]))
}

fn parse_credential_key_for_storage(key: &str) -> Result<(&str, &str)> {
    let parts: Vec<&str> = key.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(anyhow!("Invalid key format: {}", key));
    }
    Ok((parts[0], parts[1]))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    /// Build a storage instance rooted in a temp dir instead of the real config
    /// dir, so the test never touches the user's credentials.
    fn temp_storage() -> (tempfile::TempDir, FileCredentialStorage) {
        let dir = tempfile::tempdir().expect("tempdir");
        let storage = FileCredentialStorage {
            credentials_dir: dir.path().to_path_buf(),
            llm_credentials_file: dir.path().join("llm_providers.json"),
            ssh_credentials_file: dir.path().join("ssh_keys.json"),
        };
        (dir, storage)
    }

    fn mode_of(path: &Path) -> u32 {
        std::fs::metadata(path)
            .expect("credential file metadata")
            .permissions()
            .mode()
            & 0o777
    }

    /// The credential store must be owner-only from the instant the file
    /// exists, both for a fresh file and for one an older release left at 0o644.
    #[tokio::test]
    async fn credential_file_creates_and_tightens_owner_only() {
        let (_dir, storage) = temp_storage();

        // The create call itself must carry the mode: a follow-up chmod lands
        // too late by construction, so observe the file before any tightening.
        drop(
            open_private_file(&storage.llm_credentials_file)
                .await
                .expect("create store"),
        );
        assert_eq!(
            mode_of(&storage.llm_credentials_file),
            0o600,
            "a newly created credential file must be owner-only from the first byte"
        );
        // Empty probe file: drop it so the store starts from its empty default
        // instead of trying to parse zero bytes as JSON.
        std::fs::remove_file(&storage.llm_credentials_file).expect("clear probe file");
        storage
            .save_with_auto_imported("openai:first", "encrypted-blob", false)
            .await
            .expect("save fresh credential");
        assert_eq!(
            mode_of(&storage.llm_credentials_file),
            0o600,
            "rewriting the credential file must keep it owner-only"
        );

        // open(2) ignores mode for an existing file, so the descriptor-level
        // chmod must tighten one an older release left world-readable.
        std::fs::set_permissions(
            &storage.llm_credentials_file,
            std::fs::Permissions::from_mode(0o644),
        )
        .expect("widen credential file");
        storage
            .save_with_auto_imported("openai:second", "encrypted-blob", false)
            .await
            .expect("rewrite credential file");
        assert_eq!(
            mode_of(&storage.llm_credentials_file),
            0o600,
            "an existing 0o644 credential file must be tightened"
        );
    }
}
