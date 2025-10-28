use anyhow::anyhow;
use colored::Colorize;
use log::{debug, error, info, trace};
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

pub struct Action {
    source: PathBuf,
    target: PathBuf,
    dry_run: bool,
    backup: bool,
    skip_confirmation: bool,
}

impl Action {
    pub fn new(
        source: PathBuf,
        target: Option<PathBuf>,
        dry_run: bool,
        backup: bool,
        skip_confirmation: bool,
    ) -> Result<Action, anyhow::Error> {
        Ok(Action {
            source: resolve_directory(&source)?,
            target: resolve_directory(&target.clone().unwrap_or(home_dir()?))?,
            dry_run,
            backup,
            skip_confirmation,
        })
    }

    pub fn install(&self) {
        info!("Installing dotfiles from {}", fmt_dir(&self.source));

        if let Some(entries) = self.entries_to_process() {
            for entry in entries {
                let rel_path = entry.path().strip_prefix(&self.source).unwrap();
                let target = self.target.join(rel_path);
                self.install_symlink(Symlink {
                    source: entry.path().to_path_buf(),
                    target,
                });
            }
        }
    }

    pub fn uninstall(&self) {
        info!("Uninstalling dotfiles from {}", fmt_dir(&self.source));

        if let Some(entries) = self.entries_to_process() {
            for entry in entries {
                let src = entry.path();
                let rel = src.strip_prefix(&self.source).unwrap_or(src);
                let file = self.target.join(rel);

                if file.is_symlink() {
                    self.uninstall_symlink(&file.to_path_buf());
                }
            }
        }
    }

    fn install_symlink(&self, symlink: Symlink) {
        debug!("Installing {}", symlink);

        if self.dry_run {
            return;
        }

        if let Some(parent) = symlink.target.parent() {
            fs::create_dir_all(parent).ok();
        }

        if self.backup {
            match symlink.backup() {
                Ok(_) => {
                    info!(
                        "Backing up existing {} to {}{}",
                        fmt_file(&symlink.target),
                        fmt_file(&symlink.target),
                        ".bak".blue()
                    );
                }
                Err(_) => {
                    error!("Failed to backup file {}", symlink.target.display());
                }
            }
        }

        match &symlink.create() {
            Ok(result) => {
                if *result {
                    trace!("Created symlink {}", symlink);
                } else {
                    debug!("Symlink {} already installed", symlink)
                }
            }
            Err(err) => {
                error!("Failed to create symlink {}: {}", symlink, err);
            }
        };
    }

    fn uninstall_symlink(&self, target: &PathBuf) {
        debug!("Uninstalling dotfiles from {}", fmt_dir(target));

        if self.dry_run {
            return;
        }

        fs::remove_file(target).ok();

        let bak = target.with_extension("bak");

        if bak.exists() {
            debug!("Restoring backup {}", bak.display());
            fs::rename(&bak, target).ok();
        }
    }

    fn entries_to_process(&self) -> Option<Vec<DirEntry>> {
        let entries: Vec<DirEntry> = walkdir::WalkDir::new(&self.source)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .collect();

        if self.skip_confirmation {
            return Some(entries);
        }

        println!(
            "{} dotfiles to process, do you want to proceed? (y/n)",
            fmt_number(entries.len()),
        );

        if !confirm() {
            return None;
        }

        Some(entries)
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
