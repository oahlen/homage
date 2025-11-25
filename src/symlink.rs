use std::{fmt::Display, fs, os::unix::fs as unix_fs, path::PathBuf};

use log::error;

use crate::format::{fmt_file, fmt_link};

pub struct Symlink {
    pub source: PathBuf,
    pub target: PathBuf,
}

impl Symlink {
    pub fn new(source: PathBuf, target: PathBuf) -> Symlink {
        Symlink { source, target }
    }

    pub fn is_installed(&self) -> bool {
        if !self.target.is_symlink() {
            return false;
        }

        match fs::read_link(&self.target) {
            Ok(current) => current == *self.source,
            Err(_) => false,
        }
    }

    pub fn exists(&self) -> bool {
        if let Ok(result) = self.target.try_exists()
            && result
        {
            return true;
        }
        false
    }

    pub fn install(&self) {
        if let Some(parent) = self.target.parent() {
            match fs::create_dir_all(parent) {
                Ok(_) => {}
                Err(err) => {
                    error!(
                        "Failed to create parent directory {}: {}",
                        parent.display(),
                        err
                    );
                    return;
                }
            }
        }

        match unix_fs::symlink(&self.source, &self.target) {
            Ok(_) => {}
            Err(err) => {
                error!("Failed to create symlink {}: {}", self, err);
            }
        }
    }

    pub fn uninstall(&self) {
        match fs::remove_file(&self.target) {
            Ok(_) => {}
            Err(err) => {
                error!("Failed to remove symlink {}: {}", self, err);
            }
        }
    }
}

impl Display for Symlink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} -> {}",
            fmt_file(&self.source),
            fmt_link(&self.target)
        )
    }
}
