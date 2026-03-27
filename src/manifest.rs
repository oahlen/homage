use anyhow::{Context, anyhow};
use log::debug;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::symlink::Symlink;
use crate::utils::expand_tilde;

#[derive(Debug, Deserialize)]
struct ManifestFile {
    #[serde(default)]
    includes: Vec<String>,
    #[serde(default)]
    files: BTreeMap<String, String>,
}

/// A fully resolved manifest containing absolute source -> target path mappings.
#[derive(Debug)]
pub struct Manifest {
    pub entries: BTreeMap<PathBuf, PathBuf>,
}

impl Manifest {
    /// Load a manifest file and recursively resolve all includes.
    /// Detects circular includes and returns an error if found.
    /// Detects conflicting target paths and returns an error if found.
    pub fn load(path: &Path) -> anyhow::Result<Manifest> {
        let mut visited = HashSet::new();
        let mut entries = BTreeMap::new();

        Self::load_recursive(path, &mut visited, &mut entries)?;
        Self::validate_no_duplicate_targets(&entries)?;

        Ok(Manifest { entries })
    }

    fn load_recursive(
        path: &Path,
        visited: &mut HashSet<PathBuf>,
        entries: &mut BTreeMap<PathBuf, PathBuf>,
    ) -> anyhow::Result<()> {
        let canonical = path
            .canonicalize()
            .with_context(|| format!("Failed to resolve manifest path: {}", path.display()))?;

        if !visited.insert(canonical.clone()) {
            return Err(anyhow!(
                "Circular include detected: {}",
                canonical.display()
            ));
        }

        let content = std::fs::read_to_string(&canonical)
            .with_context(|| format!("Failed to read manifest: {}", canonical.display()))?;

        let manifest: ManifestFile = toml::from_str(&content)
            .with_context(|| format!("Failed to parse manifest: {}", canonical.display()))?;

        let manifest_dir = canonical
            .parent()
            .ok_or_else(|| anyhow!("Manifest has no parent directory: {}", canonical.display()))?;

        // Process includes first
        for include in &manifest.includes {
            let include_path = manifest_dir.join(include);
            debug!("Processing include: {}", include_path.display());
            Self::load_recursive(&include_path, visited, entries)?;
        }

        // Process file entries
        for (source, target) in &manifest.files {
            Self::resolve_entry(manifest_dir, source, target, entries)?;
        }

        Ok(())
    }

    fn resolve_entry(
        manifest_dir: &Path,
        source: &str,
        target: &str,
        entries: &mut BTreeMap<PathBuf, PathBuf>,
    ) -> anyhow::Result<()> {
        let abs_source = manifest_dir.join(source);
        let abs_source = abs_source
            .canonicalize()
            .with_context(|| format!("Failed to resolve source path: {}", abs_source.display()))?;

        let abs_target = expand_tilde(target)?;

        if abs_source.is_dir() {
            for entry in WalkDir::new(&abs_source)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
            {
                let rel = entry.path().strip_prefix(&abs_source).unwrap();
                let file_target = abs_target.join(rel);
                debug!(
                    "Resolved directory entry: {} -> {}",
                    entry.path().display(),
                    file_target.display()
                );
                entries.insert(entry.path().to_path_buf(), file_target);
            }
        } else if abs_source.is_file() {
            // If the target is an existing directory, place the file inside it
            let final_target = Self::resolve_final_target(&abs_source, abs_target)?;

            debug!(
                "Resolved file entry: {} -> {}",
                abs_source.display(),
                final_target.display()
            );
            entries.insert(abs_source, final_target);
        } else {
            return Err(anyhow!(
                "Source path is neither a file nor directory: {}",
                abs_source.display()
            ));
        }

        Ok(())
    }

    fn resolve_final_target(abs_source: &Path, abs_target: PathBuf) -> anyhow::Result<PathBuf> {
        let final_target = if abs_target.is_dir() {
            let file_name = abs_source
                .file_name()
                .ok_or_else(|| anyhow!("Source has no file name: {}", abs_source.display()))?;
            abs_target.join(file_name)
        } else {
            abs_target
        };

        Ok(final_target)
    }

    /// Validate that no two source files map to the same target path.
    fn validate_no_duplicate_targets(entries: &BTreeMap<PathBuf, PathBuf>) -> anyhow::Result<()> {
        let mut seen: BTreeMap<&PathBuf, &PathBuf> = BTreeMap::new();
        let mut conflicts: Vec<String> = Vec::new();

        for (source, target) in entries {
            if let Some(prev_source) = seen.insert(target, source) {
                conflicts.push(format!(
                    "  {} and {} both target {}",
                    prev_source.display(),
                    source.display(),
                    target.display()
                ));
            }
        }

        if conflicts.is_empty() {
            Ok(())
        } else {
            Err(anyhow!(
                "Conflicting target paths detected:\n{}",
                conflicts.join("\n")
            ))
        }
    }

    /// Convert all entries into a list of symlinks.
    pub fn to_symlinks(&self) -> Vec<Symlink> {
        self.entries
            .iter()
            .map(|(src, target)| Symlink::new(src.clone(), target.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::tests::{test_dir, write_file};

    use std::fs;

    #[test]
    fn parse_basic_manifest() {
        let dir = test_dir("basic");
        write_file(&dir, "dotfile.conf", "content");

        let manifest_content = format!(
            "[files]\n\"dotfile.conf\" = \"{}/target/dotfile.conf\"",
            dir.display()
        );
        let manifest_path = write_file(&dir, "manifest.toml", &manifest_content);

        let manifest = Manifest::load(&manifest_path).unwrap();
        assert_eq!(manifest.entries.len(), 1);

        let source = dir.join("dotfile.conf").canonicalize().unwrap();
        let target = dir.join("target/dotfile.conf");
        assert_eq!(manifest.entries.get(&source).unwrap(), &target);
    }

    #[test]
    fn parse_manifest_with_includes() {
        let dir = test_dir("includes");
        write_file(&dir, "file_a.conf", "a");
        write_file(&dir, "file_b.conf", "b");

        let child_content = format!(
            "[files]\n\"file_b.conf\" = \"{}/target/file_b.conf\"",
            dir.display()
        );
        write_file(&dir, "child.toml", &child_content);

        let parent_content = format!(
            "includes = [\"child.toml\"]\n\n[files]\n\"file_a.conf\" = \"{}/target/file_a.conf\"",
            dir.display()
        );
        let manifest_path = write_file(&dir, "manifest.toml", &parent_content);

        let manifest = Manifest::load(&manifest_path).unwrap();
        assert_eq!(manifest.entries.len(), 2);
    }

    #[test]
    fn circular_include_detected() {
        let dir = test_dir("circular");

        write_file(&dir, "a.toml", "includes = [\"b.toml\"]\n[files]\n");
        write_file(&dir, "b.toml", "includes = [\"a.toml\"]\n[files]\n");

        let result = Manifest::load(&dir.join("a.toml"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circular include"));
    }

    #[test]
    fn directory_source_expands_to_files() {
        let dir = test_dir("dir_source");
        fs::create_dir_all(dir.join("configs/nested")).unwrap();
        write_file(&dir, "configs/a.conf", "a");
        write_file(&dir, "configs/nested/b.conf", "b");

        let manifest_content = format!(
            "[files]\n\"configs\" = \"{}/target/configs\"",
            dir.display()
        );
        let manifest_path = write_file(&dir, "manifest.toml", &manifest_content);

        let manifest = Manifest::load(&manifest_path).unwrap();
        assert_eq!(manifest.entries.len(), 2);

        for (_src, target) in &manifest.entries {
            assert!(target.starts_with(dir.join("target/configs")));
        }
    }

    #[test]
    fn missing_source_errors() {
        let dir = test_dir("missing_src");

        let manifest_content = format!("[files]\n\"nonexistent\" = \"{}/target\"", dir.display());
        let manifest_path = write_file(&dir, "manifest.toml", &manifest_content);

        let result = Manifest::load(&manifest_path);
        assert!(result.is_err());
    }

    #[test]
    fn missing_include_errors() {
        let dir = test_dir("missing_include");

        write_file(
            &dir,
            "manifest.toml",
            "includes = [\"nonexistent.toml\"]\n[files]\n",
        );

        let result = Manifest::load(&dir.join("manifest.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn file_source_into_existing_directory_target() {
        let dir = test_dir("file_into_dir");
        write_file(&dir, "src/foo.conf", "foo");

        // Create the target as an existing directory
        let target_dir = dir.join("target_dir");
        fs::create_dir_all(&target_dir).unwrap();

        let manifest_content = format!("[files]\n\"src/foo.conf\" = \"{}\"", target_dir.display());
        let manifest_path = write_file(&dir, "manifest.toml", &manifest_content);

        let manifest = Manifest::load(&manifest_path).unwrap();
        assert_eq!(manifest.entries.len(), 1);

        let source = dir.join("src/foo.conf").canonicalize().unwrap();
        let expected_target = target_dir.join("foo.conf");
        assert_eq!(manifest.entries.get(&source).unwrap(), &expected_target);
    }

    #[test]
    fn file_source_into_nonexistent_target_uses_path_as_is() {
        let dir = test_dir("file_no_dir");
        write_file(&dir, "src/foo.conf", "foo");

        let target_path = dir.join("target/renamed.conf");

        let manifest_content = format!("[files]\n\"src/foo.conf\" = \"{}\"", target_path.display());
        let manifest_path = write_file(&dir, "manifest.toml", &manifest_content);

        let manifest = Manifest::load(&manifest_path).unwrap();
        assert_eq!(manifest.entries.len(), 1);

        let source = dir.join("src/foo.conf").canonicalize().unwrap();
        assert_eq!(manifest.entries.get(&source).unwrap(), &target_path);
    }

    #[test]
    fn duplicate_targets_in_single_manifest_errors() {
        let dir = test_dir("dup_single");
        write_file(&dir, "a.conf", "a");
        write_file(&dir, "b.conf", "b");

        let target = dir.join("target/same.conf");
        let manifest_content = format!(
            "[files]\n\"a.conf\" = \"{t}\"\n\"b.conf\" = \"{t}\"",
            t = target.display()
        );
        let manifest_path = write_file(&dir, "manifest.toml", &manifest_content);

        let result = Manifest::load(&manifest_path);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Conflicting target paths"));
        assert!(err.contains("same.conf"));
    }

    #[test]
    fn duplicate_targets_across_includes_errors() {
        let dir = test_dir("dup_includes");
        write_file(&dir, "a.conf", "a");
        write_file(&dir, "b.conf", "b");

        let target = dir.join("target/collision.conf");

        let child_content = format!("[files]\n\"b.conf\" = \"{}\"", target.display());
        write_file(&dir, "child.toml", &child_content);

        let parent_content = format!(
            "includes = [\"child.toml\"]\n\n[files]\n\"a.conf\" = \"{}\"",
            target.display()
        );
        let manifest_path = write_file(&dir, "manifest.toml", &parent_content);

        let result = Manifest::load(&manifest_path);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Conflicting target paths")
        );
    }

    #[test]
    fn duplicate_targets_via_dir_expansion_errors() {
        let dir = test_dir("dup_dir");

        // Create a file and a directory that both resolve to the same target
        write_file(&dir, "standalone/app.conf", "standalone");
        write_file(&dir, "configs/app.conf", "from dir");

        let target_dir = dir.join("target");
        let manifest_content = format!(
            "[files]\n\"standalone/app.conf\" = \"{t}/app.conf\"\n\"configs\" = \"{t}\"",
            t = target_dir.display()
        );
        let manifest_path = write_file(&dir, "manifest.toml", &manifest_content);

        let result = Manifest::load(&manifest_path);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Conflicting target paths")
        );
    }
}
