use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::symlink::Symlink;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Cache {
    #[serde(default)]
    pub files: BTreeMap<String, String>,
}

impl Cache {
    /// Load cache from the given path. Returns an empty cache if the file does not exist.
    pub fn load(path: &Path) -> anyhow::Result<Cache> {
        if !path.exists() {
            return Ok(Cache::default());
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read cache: {}", path.display()))?;

        toml::from_str(&content)
            .with_context(|| format!("Failed to parse cache: {}", path.display()))
    }

    /// Save cache to the given path, creating parent directories as needed.
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create cache directory: {}", parent.display())
            })?;
        }

        let content = toml::to_string_pretty(self).context("Failed to serialize cache")?;

        std::fs::write(path, content)
            .with_context(|| format!("Failed to write cache: {}", path.display()))
    }

    /// Delete the cache file if it exists.
    pub fn delete(path: &Path) -> anyhow::Result<()> {
        if path.exists() {
            std::fs::remove_file(path)
                .with_context(|| format!("Failed to delete cache: {}", path.display()))?;
        }
        Ok(())
    }

    /// Returns entries that are in the cache but not in the new manifest,
    /// or whose target path has changed compared to the new manifest.
    pub fn stale_entries(&self, new_entries: &BTreeMap<PathBuf, PathBuf>) -> Vec<Symlink> {
        self.files
            .iter()
            .filter(|(src, target)| {
                let src_path = PathBuf::from(src);
                let target_path = PathBuf::from(target);
                match new_entries.get(&src_path) {
                    None => true,
                    Some(new_target) => *new_target != target_path,
                }
            })
            .map(|(src, target)| Symlink::new(PathBuf::from(src), PathBuf::from(target)))
            .collect()
    }

    /// Returns all cached entries as symlinks.
    pub fn all_entries(&self) -> Vec<Symlink> {
        self.files
            .iter()
            .map(|(src, target)| Symlink::new(PathBuf::from(src), PathBuf::from(target)))
            .collect()
    }

    /// Replace all cache entries with the given manifest entries.
    pub fn update(&mut self, entries: &BTreeMap<PathBuf, PathBuf>) {
        self.files.clear();
        for (src, target) in entries {
            self.files.insert(
                src.to_string_lossy().to_string(),
                target.to_string_lossy().to_string(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::tests::test_dir;

    #[test]
    fn load_missing_cache_returns_default() {
        let dir = test_dir("missing");
        let cache = Cache::load(&dir.join("nonexistent.toml")).unwrap();
        assert!(cache.files.is_empty());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = test_dir("roundtrip");
        let path = dir.join("cache.toml");

        let mut cache = Cache::default();
        cache
            .files
            .insert("/src/a".to_string(), "/target/a".to_string());
        cache
            .files
            .insert("/src/b".to_string(), "/target/b".to_string());
        cache.save(&path).unwrap();

        let loaded = Cache::load(&path).unwrap();
        assert_eq!(loaded.files.len(), 2);
        assert_eq!(loaded.files.get("/src/a").unwrap(), "/target/a");
        assert_eq!(loaded.files.get("/src/b").unwrap(), "/target/b");
    }

    #[test]
    fn stale_entries_detects_removed() {
        let mut cache = Cache::default();
        cache
            .files
            .insert("/src/old".to_string(), "/target/old".to_string());
        cache
            .files
            .insert("/src/keep".to_string(), "/target/keep".to_string());

        let mut new_entries = BTreeMap::new();
        new_entries.insert(PathBuf::from("/src/keep"), PathBuf::from("/target/keep"));

        let stale = cache.stale_entries(&new_entries);
        assert_eq!(stale.len(), 1);
        assert_eq!(stale[0].source, PathBuf::from("/src/old"));
    }

    #[test]
    fn stale_entries_detects_target_change() {
        let mut cache = Cache::default();
        cache
            .files
            .insert("/src/file".to_string(), "/target/old_location".to_string());

        let mut new_entries = BTreeMap::new();
        new_entries.insert(
            PathBuf::from("/src/file"),
            PathBuf::from("/target/new_location"),
        );

        let stale = cache.stale_entries(&new_entries);
        assert_eq!(stale.len(), 1);
        assert_eq!(stale[0].target, PathBuf::from("/target/old_location"));
    }

    #[test]
    fn no_stale_when_entries_match() {
        let mut cache = Cache::default();
        cache
            .files
            .insert("/src/a".to_string(), "/target/a".to_string());

        let mut new_entries = BTreeMap::new();
        new_entries.insert(PathBuf::from("/src/a"), PathBuf::from("/target/a"));

        let stale = cache.stale_entries(&new_entries);
        assert!(stale.is_empty());
    }

    #[test]
    fn update_replaces_entries() {
        let mut cache = Cache::default();
        cache
            .files
            .insert("/old/src".to_string(), "/old/target".to_string());

        let mut new_entries = BTreeMap::new();
        new_entries.insert(PathBuf::from("/new/src"), PathBuf::from("/new/target"));

        cache.update(&new_entries);
        assert_eq!(cache.files.len(), 1);
        assert!(cache.files.contains_key("/new/src"));
    }

    #[test]
    fn delete_nonexistent_is_ok() {
        let dir = test_dir("delete");
        assert!(Cache::delete(&dir.join("no_such_file.toml")).is_ok());
    }

    #[test]
    fn delete_existing_removes_file() {
        let dir = test_dir("delete_existing");
        let path = dir.join("cache.toml");

        let cache = Cache::default();
        cache.save(&path).unwrap();
        assert!(path.exists());

        Cache::delete(&path).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn all_entries_returns_symlinks() {
        let mut cache = Cache::default();
        cache
            .files
            .insert("/src/a".to_string(), "/target/a".to_string());
        cache
            .files
            .insert("/src/b".to_string(), "/target/b".to_string());

        let entries = cache.all_entries();
        assert_eq!(entries.len(), 2);
    }
}
