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

use arona_core::{CredentialError, CredentialStorage};

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
        Ok(arona_config::UserConfig::config_dir().join("credentials"))
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

        let mut file_handle = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .await
            .with_context(|| format!("Failed to create credential file: {:?}", path))?;

        file_handle
            .write_all(contents.as_bytes())
            .await
            .with_context(|| "Failed to write credential file")?;

        #[cfg(unix)]
        {
            file_handle
                .set_permissions(std::fs::Permissions::from_mode(0o600))
                .await
                .with_context(
                    || "Failed to set restrictive permissions (0o600) on credential file",
                )?;
        }

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
