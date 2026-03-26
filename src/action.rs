use anyhow::Context;
use log::{debug, info};
use std::collections::{BTreeMap, HashSet};
use std::io::{BufRead, stdin};
use std::path::PathBuf;

use crate::cache::Cache;
use crate::format::{fmt_file, fmt_number};
use crate::manifest::Manifest;
use crate::symlink::Symlink;

pub struct Action {
    manifest_path: PathBuf,
    cache_path: PathBuf,
    dry_run: bool,
    skip_confirmation: bool,
}

impl Action {
    pub fn new(
        manifest_path: PathBuf,
        cache_path: PathBuf,
        dry_run: bool,
        skip_confirmation: bool,
    ) -> Action {
        Action {
            manifest_path,
            cache_path,
            dry_run,
            skip_confirmation,
        }
    }

    pub fn install(&self) -> anyhow::Result<()> {
        info!(
            "Installing dotfiles from manifest: {}",
            self.manifest_path.display()
        );

        let manifest = Manifest::load(&self.manifest_path).context("Failed to load manifest")?;

        let cache = Cache::load(&self.cache_path).context("Failed to load cache")?;

        // Find stale entries: in cache but no longer in manifest (or target changed)
        let stale: Vec<_> = cache
            .stale_entries(&manifest.entries)
            .into_iter()
            .filter(|s| s.is_installed())
            .collect();

        // Find new entries: in manifest but not yet installed
        let to_install: Vec<_> = manifest
            .to_symlinks()
            .into_iter()
            .filter(|s| !s.is_installed())
            .collect();

        // Pre-flight check: verify every target is available before making any changes.
        // Targets occupied by stale symlinks are excluded since those will be removed first.
        let stale_targets: HashSet<&PathBuf> = stale.iter().map(|s| &s.target).collect();
        let conflicts: Vec<_> = to_install
            .iter()
            .filter(|entry| entry.exists() && !stale_targets.contains(&entry.target))
            .collect();

        if !conflicts.is_empty() {
            let listing: Vec<String> = conflicts
                .iter()
                .map(|e| format!("  {}", e.target.display()))
                .collect();
            return Err(anyhow::anyhow!(
                "Cannot install, the following target files already exist:\n{}",
                listing.join("\n")
            ));
        }

        if stale.is_empty() && to_install.is_empty() {
            println!("Everything is up to date");
            return Ok(());
        }

        if !stale.is_empty() {
            println!(
                "Found {} stale dotfile(s) to remove",
                fmt_number(stale.len()),
            );
        }

        if !to_install.is_empty() {
            println!(
                "Found {} dotfile(s) to install",
                fmt_number(to_install.len()),
            );
        }

        if !self.skip_confirmation {
            println!("Do you want to proceed? (y/n)");
            if !confirm() {
                return Ok(());
            }
        }

        // Remove stale symlinks
        for entry in &stale {
            debug!("Removing stale symlink: {}", fmt_file(&entry.target));
            if !self.dry_run {
                entry.uninstall();
            }
        }

        // Install new symlinks
        for entry in &to_install {
            debug!("Installing: {}", entry);
            if !self.dry_run {
                entry.install();
            }
        }

        // Update cache with current manifest entries
        if !self.dry_run {
            let mut new_cache = Cache::default();
            new_cache.update(&manifest.entries);
            new_cache
                .save(&self.cache_path)
                .context("Failed to save cache")?;
            debug!("Cache updated at {}", self.cache_path.display());
        }

        Ok(())
    }

    pub fn uninstall(&self) -> anyhow::Result<()> {
        info!(
            "Uninstalling dotfiles from manifest: {}",
            self.manifest_path.display()
        );

        let manifest = Manifest::load(&self.manifest_path).context("Failed to load manifest")?;

        let cache = Cache::load(&self.cache_path).context("Failed to load cache")?;

        // Collect all entries from both manifest and cache (union)
        let mut all_entries: BTreeMap<PathBuf, PathBuf> = BTreeMap::new();
        for (src, target) in &manifest.entries {
            all_entries.insert(src.clone(), target.clone());
        }
        for entry in cache.all_entries() {
            all_entries
                .entry(entry.source.clone())
                .or_insert(entry.target.clone());
        }

        // Filter to entries that are actually installed as symlinks
        let to_remove: Vec<_> = all_entries
            .iter()
            .map(|(src, target)| Symlink::new(src.clone(), target.clone()))
            .filter(|s| s.is_installed())
            .collect();

        if to_remove.is_empty() {
            println!("No dotfiles to uninstall");
            if !self.dry_run {
                Cache::delete(&self.cache_path)?;
            }
            return Ok(());
        }

        println!(
            "Found {} dotfile(s) to uninstall",
            fmt_number(to_remove.len()),
        );

        if !self.skip_confirmation {
            println!("Do you want to proceed? (y/n)");
            if !confirm() {
                return Ok(());
            }
        }

        for entry in &to_remove {
            debug!("Uninstalling: {}", fmt_file(&entry.target));
            if !self.dry_run {
                entry.uninstall();
            }
        }

        // Delete cache
        if !self.dry_run {
            Cache::delete(&self.cache_path)?;
            debug!("Cache deleted");
        }

        Ok(())
    }
}

fn confirm() -> bool {
    let mut buffer = String::new();
    let mut handle = stdin().lock();

    match handle.read_line(&mut buffer) {
        Ok(_) => matches!(
            buffer.to_string().trim().to_lowercase().as_str(),
            "yes" | "y"
        ),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::Path;

    fn test_dir(name: &str) -> PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("homage_action_{}_{}", name, ts));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn install_creates_symlinks_and_cache() {
        let dir = test_dir("install");
        let target_dir = dir.join("home");
        fs::create_dir_all(&target_dir).unwrap();

        // Create source dotfiles
        write_file(&dir, "dotfiles/config.toml", "config content");
        write_file(&dir, "dotfiles/nested/app.conf", "app content");

        // Create manifest pointing to the dotfiles directory
        let manifest_content = format!(
            "[files]\n\"dotfiles\" = \"{}/dotfiles_target\"",
            target_dir.display()
        );
        let manifest_path = write_file(&dir, "manifest.toml", &manifest_content);

        let cache_path = dir.join("cache/cache.toml");

        let action = Action::new(manifest_path, cache_path.clone(), false, true);
        action.install().unwrap();

        // Verify symlinks were created
        let target_config = target_dir.join("dotfiles_target/config.toml");
        let target_app = target_dir.join("dotfiles_target/nested/app.conf");
        assert!(target_config.is_symlink());
        assert!(target_app.is_symlink());

        // Verify cache was created
        assert!(cache_path.exists());

        let cache = Cache::load(&cache_path).unwrap();
        assert_eq!(cache.files.len(), 2);
    }

    #[test]
    fn install_removes_stale_entries() {
        let dir = test_dir("stale");
        let target_dir = dir.join("home");
        fs::create_dir_all(&target_dir).unwrap();

        // Create source dotfiles
        write_file(&dir, "dotfiles/keep.conf", "keep");
        write_file(&dir, "dotfiles/remove.conf", "remove");

        // Create manifest with both files
        let manifest_content = format!(
            "[files]\n\"dotfiles\" = \"{}/target\"",
            target_dir.display()
        );
        let manifest_path = write_file(&dir, "manifest.toml", &manifest_content);
        let cache_path = dir.join("cache/cache.toml");

        // Install both files
        let action = Action::new(manifest_path.clone(), cache_path.clone(), false, true);
        action.install().unwrap();

        assert!(target_dir.join("target/keep.conf").is_symlink());
        assert!(target_dir.join("target/remove.conf").is_symlink());

        // Remove one file from source and reinstall
        fs::remove_file(dir.join("dotfiles/remove.conf")).unwrap();

        let manifest_content = format!(
            "[files]\n\"dotfiles\" = \"{}/target\"",
            target_dir.display()
        );
        write_file(&dir, "manifest.toml", &manifest_content);

        let action = Action::new(manifest_path, cache_path.clone(), false, true);
        action.install().unwrap();

        // keep.conf should still be installed
        assert!(target_dir.join("target/keep.conf").is_symlink());
        // remove.conf should be gone (stale entry removed)
        assert!(!target_dir.join("target/remove.conf").exists());

        let cache = Cache::load(&cache_path).unwrap();
        assert_eq!(cache.files.len(), 1);
    }

    #[test]
    fn uninstall_removes_all_and_deletes_cache() {
        let dir = test_dir("uninstall");
        let target_dir = dir.join("home");
        fs::create_dir_all(&target_dir).unwrap();

        write_file(&dir, "dotfiles/a.conf", "a");
        write_file(&dir, "dotfiles/b.conf", "b");

        let manifest_content = format!(
            "[files]\n\"dotfiles\" = \"{}/target\"",
            target_dir.display()
        );
        let manifest_path = write_file(&dir, "manifest.toml", &manifest_content);
        let cache_path = dir.join("cache/cache.toml");

        // Install first
        let action = Action::new(manifest_path.clone(), cache_path.clone(), false, true);
        action.install().unwrap();

        assert!(target_dir.join("target/a.conf").is_symlink());
        assert!(target_dir.join("target/b.conf").is_symlink());
        assert!(cache_path.exists());

        // Uninstall
        let action = Action::new(manifest_path, cache_path.clone(), false, true);
        action.uninstall().unwrap();

        assert!(!target_dir.join("target/a.conf").exists());
        assert!(!target_dir.join("target/b.conf").exists());
        assert!(!cache_path.exists());
    }

    #[test]
    fn idempotent_install() {
        let dir = test_dir("idempotent");
        let target_dir = dir.join("home");
        fs::create_dir_all(&target_dir).unwrap();

        write_file(&dir, "dotfiles/file.conf", "content");

        let manifest_content = format!(
            "[files]\n\"dotfiles\" = \"{}/target\"",
            target_dir.display()
        );
        let manifest_path = write_file(&dir, "manifest.toml", &manifest_content);
        let cache_path = dir.join("cache/cache.toml");

        // Install twice - second should be a no-op
        let action = Action::new(manifest_path.clone(), cache_path.clone(), false, true);
        action.install().unwrap();

        let action = Action::new(manifest_path, cache_path.clone(), false, true);
        action.install().unwrap();

        assert!(target_dir.join("target/file.conf").is_symlink());

        let cache = Cache::load(&cache_path).unwrap();
        assert_eq!(cache.files.len(), 1);
    }

    #[test]
    fn install_aborts_on_existing_target_files() {
        let dir = test_dir("conflict");
        let target_dir = dir.join("home");
        fs::create_dir_all(&target_dir).unwrap();

        write_file(&dir, "dotfiles/a.conf", "a");
        write_file(&dir, "dotfiles/b.conf", "b");

        // Pre-create target files so they conflict
        write_file(&target_dir, "target/a.conf", "existing a");
        write_file(&target_dir, "target/b.conf", "existing b");

        let manifest_content = format!(
            "[files]\n\"dotfiles\" = \"{}/target\"",
            target_dir.display()
        );
        let manifest_path = write_file(&dir, "manifest.toml", &manifest_content);
        let cache_path = dir.join("cache/cache.toml");

        let action = Action::new(manifest_path, cache_path.clone(), false, true);
        let result = action.install();

        // Should fail listing all conflicts
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("a.conf"));
        assert!(err.contains("b.conf"));

        // Nothing should have been modified
        assert!(!cache_path.exists());
        assert!(!target_dir.join("target/a.conf").is_symlink());
        assert!(!target_dir.join("target/b.conf").is_symlink());
        // Original files should be untouched
        assert_eq!(
            fs::read_to_string(target_dir.join("target/a.conf")).unwrap(),
            "existing a"
        );
        assert_eq!(
            fs::read_to_string(target_dir.join("target/b.conf")).unwrap(),
            "existing b"
        );
    }

    #[test]
    fn install_allows_stale_target_reuse() {
        let dir = test_dir("stale_reuse");
        let target_dir = dir.join("home");
        fs::create_dir_all(&target_dir).unwrap();

        // First: install file_a.conf -> target/shared.conf
        write_file(&dir, "file_a.conf", "a");
        let manifest_v1 = format!(
            "[files]\n\"file_a.conf\" = \"{}/target/shared.conf\"",
            target_dir.display()
        );
        let manifest_path = write_file(&dir, "manifest.toml", &manifest_v1);
        let cache_path = dir.join("cache/cache.toml");

        let action = Action::new(manifest_path.clone(), cache_path.clone(), false, true);
        action.install().unwrap();
        assert!(target_dir.join("target/shared.conf").is_symlink());

        // Second: change manifest so file_b.conf -> target/shared.conf (same target, different source)
        write_file(&dir, "file_b.conf", "b");
        let manifest_v2 = format!(
            "[files]\n\"file_b.conf\" = \"{}/target/shared.conf\"",
            target_dir.display()
        );
        write_file(&dir, "manifest.toml", &manifest_v2);

        // This should succeed: old symlink is stale and will be removed before installing new one
        let action = Action::new(manifest_path, cache_path.clone(), false, true);
        action.install().unwrap();

        let link = target_dir.join("target/shared.conf");
        assert!(link.is_symlink());
        assert_eq!(fs::read_to_string(&link).unwrap(), "b");
    }
}
