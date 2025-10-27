use anyhow::anyhow;
use log::{debug, error, info, trace, warn};
use std::{env, fs, path::PathBuf};

use crate::{format::format_dir, symlink::Symlink};

pub struct Action {
    source: PathBuf,
    target: PathBuf,
    dry_run: bool,
    backup: bool,
}

impl Action {
    pub fn new(
        source: PathBuf,
        target: Option<PathBuf>,
        dry_run: bool,
        backup: bool,
    ) -> Result<Action, anyhow::Error> {
        Ok(Action {
            source: resolve_directory(&source)?,
            target: resolve_directory(&target.clone().unwrap_or(home_dir()?))?,
            dry_run,
            backup,
        })
    }

    pub fn install(&self) {
        info!("Installing dotfiles from {}", format_dir(&self.source));

        for entry in walkdir::WalkDir::new(&self.source)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let rel_path = entry.path().strip_prefix(&self.source).unwrap();
            let dest = self.target.join(rel_path);
            self.install_symlink(Symlink {
                source: entry.path().to_path_buf(),
                dest,
            });
        }
    }

    pub fn uninstall(&self) {
        for entry in walkdir::WalkDir::new(&self.source)
            .into_iter()
            .filter_map(Result::ok)
        {
            let src = entry.path();
            let rel = src.strip_prefix(&self.source).unwrap_or(src);
            let file = self.target.join(rel);

            if file.is_symlink() {
                self.uninstall_symlink(&file.to_path_buf());
            }
        }
    }

    fn install_symlink(&self, symlink: Symlink) {
        info!("Installing {}", symlink);

        if self.dry_run {
            return;
        }

        if let Some(parent) = symlink.dest.parent() {
            fs::create_dir_all(parent).ok();
        }

        if self.backup {
            match symlink.backup() {
                Ok(_) => {
                    warn!(
                        "Backing up existing {} to {}.bak",
                        symlink.dest.display(),
                        symlink.dest.display()
                    );
                }
                Err(_) => {
                    error!("Failed to backup file {}", symlink.dest.display());
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

    fn uninstall_symlink(&self, dest: &PathBuf) {
        info!("Uninstalling dotfiles from {}", format_dir(dest));

        if self.dry_run {
            return;
        }

        fs::remove_file(dest).ok();

        let bak = dest.with_extension("bak");

        if bak.exists() {
            debug!("Restoring backup {}", bak.display());
            fs::rename(&bak, dest).ok();
        }
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
