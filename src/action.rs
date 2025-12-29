use anyhow::anyhow;
use log::{Level, debug, error, info, log_enabled};
use std::{
    env,
    fs::{self},
    io::{BufRead, stdin},
    path::PathBuf,
};
use walkdir::DirEntry;

use crate::{
    format::{fmt_dir, fmt_file, fmt_number},
    symlink::Symlink,
};

#[derive(Debug)]
pub struct Action {
    source: PathBuf,
    target: PathBuf,
    dry_run: bool,
    skip_confirmation: bool,
}

impl Action {
    pub fn new(
        source: PathBuf,
        target: Option<PathBuf>,
        dry_run: bool,
        skip_confirmation: bool,
    ) -> Result<Action, anyhow::Error> {
        Ok(Action {
            source: resolve_directory(&source)?,
            target: resolve_directory(&target.clone().unwrap_or(home_dir()?))?,
            dry_run,
            skip_confirmation,
        })
    }

    pub fn install(&self) {
        info!("Installing dotfiles from {}", fmt_dir(&self.source));

        let entries: Vec<_> = self
            .entries_to_process()
            .into_iter()
            .filter(|e| !e.is_installed())
            .collect();

        if entries.is_empty() {
            println!("No dotfiles to install");
            return;
        }

        let mut errors = 0;
        for entry in &entries {
            if entry.exists() {
                error!(
                    "Target file {} already exists, please remove it or back it up first",
                    entry.target.display()
                );
                errors += 1;
            }
        }

        if errors > 0 {
            return;
        }

        if !self.skip_confirmation {
            println!(
                "Found {} dotfile(s) to install, do you want to proceed? (y/n)",
                fmt_number(entries.len()),
            );

            if !confirm() {
                return;
            }
        }

        for entry in entries {
            debug!("Installing {}", entry);

            if !self.dry_run {
                entry.install();
            }
        }
    }

    pub fn uninstall(&self) {
        info!("Uninstalling dotfiles from {}", fmt_dir(&self.source));

        let entries: Vec<_> = self
            .entries_to_process()
            .into_iter()
            .filter(|e| e.is_installed())
            .collect();

        if entries.is_empty() {
            println!("No dotfiles to uninstall");
            return;
        }

        if !self.skip_confirmation {
            println!(
                "Found {} dotfile(s) to uninstall, do you want to proceed? (y/n)",
                fmt_number(entries.len()),
            );

            if !confirm() {
                return;
            }
        }

        for entry in entries {
            if log_enabled!(Level::Debug) {
                debug!("Uninstalling {}", fmt_file(&entry.target));
            }

            if !self.dry_run {
                entry.uninstall();
            }
        }
    }

    pub fn clean(&self) {
        info!("Cleaning dotfiles from {}", fmt_dir(&self.source));

        let entries = self.entries_to_clean();

        if !self.skip_confirmation {
            println!(
                "Found {} dotfile(s) to clean, do you want to proceed? (y/n)",
                fmt_number(entries.len()),
            );

            if !confirm() {
                return;
            }
        }

        for entry in entries {
            if log_enabled!(Level::Debug) {
                debug!("Cleaning {}", entry.path().display());
            }

            if !self.dry_run {
                match fs::remove_file(entry.path()) {
                    Ok(_) => {}
                    Err(err) => {
                        error!(
                            "Failed to create parent directory {}: {}",
                            entry.path().display(),
                            err
                        );
                    }
                }
            }
        }
    }

    fn entries_to_process(&self) -> Vec<Symlink> {
        let entries: Vec<DirEntry> = walkdir::WalkDir::new(&self.source)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .collect();

        let mut links: Vec<Symlink> = Vec::new();

        for entry in entries {
            let src = entry.path();
            let rel_path = src.strip_prefix(&self.source).unwrap_or(src);
            let target = self.target.join(rel_path);

            links.push(Symlink::new(entry.path().to_path_buf(), target));
        }

        links
    }

    fn entries_to_clean(&self) -> Vec<DirEntry> {
        let entries: Vec<DirEntry> = walkdir::WalkDir::new(&self.target)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path_is_symlink())
            .filter_map(|e| match fs::read_link(e.path()) {
                Ok(res) => Some((e, res)),
                Err(_) => None,
            })
            .filter(|(_, res)| res.starts_with(&self.source))
            .filter_map(|(e, res)| match res.canonicalize() {
                Ok(_) => None,
                Err(_) => Some(e),
            })
            .collect();

        entries
    }
}

fn resolve_directory(path: &PathBuf) -> Result<PathBuf, anyhow::Error> {
    if !path.exists() {
        return Err(anyhow!("Directory {} does not exist", path.display()));
    }

    let resolved = if path.is_relative() {
        match fs::canonicalize(path) {
            Ok(result) => Ok(result.to_path_buf()),
            Err(_) => Err(anyhow!("Unable to resolve directory {}", path.display())),
        }
    } else {
        Ok(path.to_path_buf())
    }?;

    if !resolved.is_dir() {
        return Err(anyhow!("Input {} is not a directory", path.display()));
    }

    Ok(resolved)
}

fn home_dir() -> Result<PathBuf, anyhow::Error> {
    match env::var("HOME").map(PathBuf::from) {
        Ok(result) => Ok(result),
        Err(_) => Err(anyhow!("Could not determine $HOME")),
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
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_dir(prefix: &str, create: bool) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("homage_test_{}_{}", prefix, ts));

        if create {
            fs::create_dir_all(&dir).unwrap();
        }

        dir
    }

    fn create_file(path: &PathBuf, name: &str, content: &[u8]) -> PathBuf {
        let source = path.join(name);
        File::create(&source).unwrap().write_all(content).unwrap();
        source
    }

    #[test]
    fn action_new_missing_source_errors() {
        let missing = unique_dir("missing", false);
        assert!(!missing.exists());

        let err = Action::new(missing.clone(), Some(missing.clone()), false, true).unwrap_err();
        assert!(err.to_string().contains("does not exist"));
    }

    #[test]
    fn entries_to_process_builds_relative_targets() {
        let source = unique_dir("source", true);
        let nested = source.join("nested");
        fs::create_dir_all(&nested).unwrap();

        let _ = create_file(&source, "a.txt", b"alpha");
        let _ = create_file(&nested, "a.txt", b"alpha");
        let target = unique_dir("target", true);

        let action = Action::new(source.clone(), Some(target.clone()), false, true).unwrap();
        let links = action.entries_to_process();
        assert_eq!(links.len(), 2);

        for link in links {
            let rel = link.source.strip_prefix(&source).unwrap();
            assert_eq!(link.target, target.join(rel));
        }
    }

    #[test]
    fn entries_to_clean_finds_broken_symlinks() {
        let source = unique_dir("source_clean", true);
        let nested = source.join("nested");
        fs::create_dir_all(&nested).unwrap();

        let _ = create_file(&nested, "ok.txt", b"ok");
        let file_broken = create_file(&nested, "broken.txt", b"gone");

        let target = unique_dir("target_clean", true);

        let action = Action::new(source.clone(), Some(target.clone()), false, true).unwrap();
        let links = action.entries_to_process();

        for link in &links {
            link.install();
        }

        fs::remove_file(&file_broken).unwrap();

        let broken_entries = action.entries_to_clean();
        assert_eq!(broken_entries.len(), 1);
        let broken_path = broken_entries[0].path();
        assert!(broken_path.ends_with("broken.txt"));
    }
}
