use anyhow::{Context, Result, anyhow};
use parking_lot::RwLock;
use serde::de::DeserializeOwned;
use std::{
    collections::HashMap,
    hash::Hash,
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

use tracing::info;

pub struct TomlConfigCache<K, T>
where
    K: Hash + Eq + Clone,
    T: Clone,
{
    configs: Arc<RwLock<HashMap<K, T>>>,
    mtimes: Arc<RwLock<HashMap<K, SystemTime>>>,
}

impl<K, T> TomlConfigCache<K, T>
where
    K: Hash + Eq + Clone + std::fmt::Debug,
    T: Clone + DeserializeOwned,
{
    pub fn new() -> Self {
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
            mtimes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn load_from_file(&self, path: &Path) -> Result<Option<T>> {
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let config: T = toml::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(Some(config))
    }

    pub fn get_cached(&self, key: &K) -> Option<T> {
        self.configs.read().get(key).cloned()
    }

    pub fn get_all(&self) -> Vec<T> {
        self.configs.read().values().cloned().collect()
    }

    pub fn insert(&self, key: K, config: T, path: &Path) {
        let mtime = std::fs::metadata(path).and_then(|m| m.modified()).ok();
        self.configs.write().insert(key.clone(), config);
        if let Some(mt) = mtime {
            self.mtimes.write().insert(key, mt);
        }
    }

    pub fn reload<F, E>(&self, key: &K, path: &Path, validate: F) -> Result<bool>
    where
        F: FnOnce(&T) -> Result<(), E>,
        E: std::fmt::Display,
    {
        let current_modified = std::fs::metadata(path).ok().and_then(|m| m.modified().ok());

        {
            let mtimes = self.mtimes.read();
            if let Some(cached_mtime) = mtimes.get(key)
                && let Some(current) = current_modified
                && &current == cached_mtime
            {
                return Ok(false);
            }
        }

        if let Some(config) = self.load_from_file(path)? {
            validate(&config).map_err(|e| anyhow!("Validation failed: {e}"))?;
            self.configs.write().insert(key.clone(), config);
            if let Some(modified) = current_modified {
                self.mtimes.write().insert(key.clone(), modified);
            }
            info!("Reloaded config for {:?}", key);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn reload_simple(&self, key: &K, path: &Path) -> Result<bool> {
        self.reload::<_, std::convert::Infallible>(key, path, |_| Ok(()))
    }

    pub fn invalidate(&self, key: &K) {
        self.configs.write().remove(key);
        self.mtimes.write().remove(key);
    }

    pub fn invalidate_all(&self) {
        self.configs.write().clear();
        self.mtimes.write().clear();
    }

    pub fn cached_keys(&self) -> Vec<K> {
        self.configs.read().keys().cloned().collect()
    }

    pub fn get_cached_configs_map(&self) -> HashMap<K, T> {
        self.configs.read().clone()
    }

    pub fn config_root() -> PathBuf {
        super::app_config::UserConfig::discover_config_root()
    }
}

impl<K, T> Default for TomlConfigCache<K, T>
where
    K: Hash + Eq + Clone + std::fmt::Debug,
    T: Clone + DeserializeOwned,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};
    use serde::Deserialize;
    use std::io::Write;

    #[derive(Debug, Clone, Deserialize, PartialEq)]
    struct TestConfig {
        name: String,
        value: u32,
    }

    #[test]
    fn load_from_missing_file_returns_none() -> Result<()> {
        let cache: TomlConfigCache<String, TestConfig> = TomlConfigCache::new();
        let result = cache.load_from_file(Path::new("/nonexistent/test.toml"))?;
        assert!(result.is_none());
        Ok(())
    }

    #[test]
    fn load_parse_and_cache_roundtrip() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("test.toml");
        let mut f = std::fs::File::create(&path)?;
        f.write_all(b"name = \"hello\"\nvalue = 42\n")?;

        let cache: TomlConfigCache<String, TestConfig> = TomlConfigCache::new();
        let loaded = cache
            .load_from_file(&path)?
            .context("expected Some config")?;
        assert_eq!(loaded.name, "hello");
        assert_eq!(loaded.value, 42);

        cache.insert("test".to_string(), loaded.clone(), &path);
        let cached = cache
            .get_cached(&"test".to_string())
            .context("expected cached config")?;
        assert_eq!(cached, loaded);
        Ok(())
    }

    #[test]
    fn reload_skips_when_mtime_unchanged() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("test.toml");
        std::fs::write(&path, b"name = \"a\"\nvalue = 1\n")?;

        let cache: TomlConfigCache<String, TestConfig> = TomlConfigCache::new();
        let key = "test".to_string();

        assert!(cache.reload_simple(&key, &path)?);
        assert!(!cache.reload_simple(&key, &path)?);
        Ok(())
    }

    #[test]
    fn invalidate_removes_cached_entry() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("test.toml");
        std::fs::write(&path, b"name = \"b\"\nvalue = 2\n")?;

        let cache: TomlConfigCache<String, TestConfig> = TomlConfigCache::new();
        let key = "test".to_string();
        cache.reload_simple(&key, &path)?;
        assert!(cache.get_cached(&key).is_some());

        cache.invalidate(&key);
        assert!(cache.get_cached(&key).is_none());
        Ok(())
    }

    #[test]
    fn get_all_returns_all_entries() -> Result<()> {
        let _dir = tempfile::tempdir()?;
        let cache: TomlConfigCache<String, TestConfig> = TomlConfigCache::new();
        cache.insert(
            "a".to_string(),
            TestConfig {
                name: "a".into(),
                value: 1,
            },
            Path::new("a"),
        );
        cache.insert(
            "b".to_string(),
            TestConfig {
                name: "b".into(),
                value: 2,
            },
            Path::new("b"),
        );

        let all = cache.get_all();
        assert_eq!(all.len(), 2);
        Ok(())
    }

    #[test]
    fn load_from_file_with_invalid_toml_returns_error() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("bad.toml");
        std::fs::write(&path, b"not valid toml {{{")?;

        let cache: TomlConfigCache<String, TestConfig> = TomlConfigCache::new();
        assert!(cache.load_from_file(&path).is_err());
        Ok(())
    }
}
