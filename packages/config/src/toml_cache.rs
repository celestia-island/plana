use anyhow::{Context, Result, anyhow};
use parking_lot::RwLock;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    hash::Hash,
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

use tracing::info;

/// The freshness stamp recorded for one cached config file.
///
/// mtime ALONE is not a change detector: Linux timestamps are
/// coarse-clock (jiffy-aligned), so a second write inside the same
/// tick as the first keeps the mtime bit-for-bit, and an mtime-only
/// cache would keep serving the stale bytes (the scepter #625 flake).
/// A stamp therefore also carries the file size and, when the bytes
/// were readable, their SHA-256 digest — same mtime AND same size AND
/// same digest is the only combination trusted as "unchanged".
#[derive(Debug, Clone, PartialEq, Eq)]
struct FileStamp {
    mtime: SystemTime,
    size: u64,
    /// `None` when the bytes could not be read at stamp time; such a
    /// stamp never proves "unchanged" (reload falls through instead).
    digest: Option<[u8; 32]>,
}

impl FileStamp {
    /// Stat + hash one config file. Config files are small, so the
    /// read+hash cost is acceptable on every reload probe.
    fn of(path: &Path) -> Option<Self> {
        let meta = std::fs::metadata(path).ok()?;
        let mtime = meta.modified().ok()?;
        let size = meta.len();
        let digest = std::fs::read(path).ok().map(|b| Sha256::digest(&b).into());
        Some(Self {
            mtime,
            size,
            digest,
        })
    }

    /// Whether `self` and `other` describe provably identical bytes.
    /// Equal mtime is necessary but never sufficient (coarse clocks);
    /// size and digest must both match, and both must be known.
    fn proves_unchanged(&self, other: &Self) -> bool {
        self.mtime == other.mtime
            && self.size == other.size
            && matches!((self.digest, other.digest), (Some(a), Some(b)) if a == b)
    }
}

pub struct TomlConfigCache<K, T>
where
    K: Hash + Eq + Clone,
    T: Clone,
{
    configs: Arc<RwLock<HashMap<K, T>>>,
    stamps: Arc<RwLock<HashMap<K, FileStamp>>>,
}

impl<K, T> TomlConfigCache<K, T>
where
    K: Hash + Eq + Clone + std::fmt::Debug,
    T: Clone + DeserializeOwned,
{
    pub fn new() -> Self {
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
            stamps: Arc::new(RwLock::new(HashMap::new())),
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
        let stamp = FileStamp::of(path);
        self.configs.write().insert(key.clone(), config);
        if let Some(stamp) = stamp {
            self.stamps.write().insert(key, stamp);
        }
    }

    pub fn reload<F, E>(&self, key: &K, path: &Path, validate: F) -> Result<bool>
    where
        F: FnOnce(&T) -> Result<(), E>,
        E: std::fmt::Display,
    {
        let current = FileStamp::of(path);

        // Skip only when the stamps PROVE the bytes are unchanged.
        // mtime equality alone used to short-circuit here; under a
        // coarse clock a same-tick rewrite inherited the old mtime and
        // the fresh bytes were never read (scepter #625).
        {
            let stamps = self.stamps.read();
            if let (Some(cached), Some(now)) = (stamps.get(key), &current)
                && cached.proves_unchanged(now)
            {
                return Ok(false);
            }
        }

        if let Some(config) = self.load_from_file(path)? {
            validate(&config).map_err(|e| anyhow!("Validation failed: {e}"))?;
            self.configs.write().insert(key.clone(), config);
            if let Some(stamp) = current {
                self.stamps.write().insert(key.clone(), stamp);
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
        self.stamps.write().remove(key);
    }

    pub fn invalidate_all(&self) {
        self.configs.write().clear();
        self.stamps.write().clear();
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

    /// Forces `path` to carry exactly `mtime` (std `File::set_modified`)
    /// — the deterministic way to reproduce coarse-clock same-tick
    /// writes without sleeping.
    fn force_mtime(path: &Path, mtime: std::time::SystemTime) -> Result<()> {
        let file = std::fs::OpenOptions::new().write(true).open(path)?;
        file.set_modified(mtime)?;
        Ok(())
    }

    #[test]
    fn reload_reloads_when_mtime_and_size_match_but_content_differs() -> Result<()> {
        // The original weakness, pinned: a second write inside the
        // same clock tick keeps the mtime, and a size-preserving edit
        // defeats a size check too — only the content digest catches
        // it. The cache must reload and serve the new bytes.
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("test.toml");
        std::fs::write(&path, b"name = \"aa\"\nvalue = 1\n")?;

        let cache: TomlConfigCache<String, TestConfig> = TomlConfigCache::new();
        let key = "test".to_string();
        assert!(cache.reload_simple(&key, &path)?);
        assert_eq!(
            cache.get_cached(&key).context("cached after 1st")?.name,
            "aa"
        );

        let cached_mtime = std::fs::metadata(&path)?.modified()?;
        std::fs::write(&path, b"name = \"bb\"\nvalue = 1\n")?; // same length
        force_mtime(&path, cached_mtime)?;

        assert!(
            cache.reload_simple(&key, &path)?,
            "same mtime + same size but different content must reload"
        );
        assert_eq!(
            cache.get_cached(&key).context("cached after 2nd")?.name,
            "bb",
            "the cache must serve the rewritten bytes, not the stale ones"
        );
        Ok(())
    }

    #[test]
    fn reload_skips_when_mtime_and_content_unchanged() -> Result<()> {
        // Rewriting the IDENTICAL bytes and forcing the SAME mtime:
        // the digest proves unchanged, so the reload is skipped.
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("test.toml");
        std::fs::write(&path, b"name = \"aa\"\nvalue = 1\n")?;

        let cache: TomlConfigCache<String, TestConfig> = TomlConfigCache::new();
        let key = "test".to_string();
        assert!(cache.reload_simple(&key, &path)?);

        let cached_mtime = std::fs::metadata(&path)?.modified()?;
        std::fs::write(&path, b"name = \"aa\"\nvalue = 1\n")?;
        force_mtime(&path, cached_mtime)?;

        assert!(
            !cache.reload_simple(&key, &path)?,
            "identical bytes under the same mtime must stay skipped"
        );
        Ok(())
    }

    #[test]
    fn reload_reloads_when_mtime_changes() -> Result<()> {
        // mtime moved → reload, regardless of what the digest would say.
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("test.toml");
        std::fs::write(&path, b"name = \"aa\"\nvalue = 1\n")?;

        let cache: TomlConfigCache<String, TestConfig> = TomlConfigCache::new();
        let key = "test".to_string();
        assert!(cache.reload_simple(&key, &path)?);

        let cached_mtime = std::fs::metadata(&path)?.modified()?;
        std::fs::write(&path, b"name = \"cc\"\nvalue = 9\n")?;
        force_mtime(&path, cached_mtime + std::time::Duration::from_secs(3600))?;

        assert!(
            cache.reload_simple(&key, &path)?,
            "a moved mtime must always reload"
        );
        assert_eq!(
            cache.get_cached(&key).context("cached after move")?.value,
            9
        );
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
