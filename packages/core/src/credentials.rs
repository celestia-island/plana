//! Credential type definitions
//!
//! Defines credential-related traits for use by shared and core.

pub use crate::errors::CredentialError;

/// Database credential storage interface
#[async_trait::async_trait]
pub trait CredentialStorage: Send + Sync {
    async fn save(&self, key: &str, encrypted_value: &str) -> Result<(), CredentialError>;
    async fn load(&self, key: &str) -> Result<Option<String>, CredentialError>;
    async fn delete(&self, key: &str) -> Result<bool, CredentialError>;
    async fn list_all(&self) -> Result<Vec<(String, String)>, CredentialError>;
}
