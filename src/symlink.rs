use std::{fmt::Display, fs, os::unix::fs as unix_fs, path::PathBuf};

use crate::format::{format_file, format_link};

pub struct Symlink {
    pub source: PathBuf,
    pub dest: PathBuf,
}

impl Symlink {
    pub fn create(&self) -> Result<bool, anyhow::Error> {
        if self.dest.is_symlink() {
            let current_target = fs::read_link(&self.dest)?;
            if current_target == *self.source {
                return Ok(false);
            }

            fs::remove_file(&self.dest)?
        } else if self.dest.exists() {
            fs::remove_file(&self.dest)?
        }

        unix_fs::symlink(&self.source, &self.dest)?;
        Ok(true)
    }

    pub fn backup(&self) -> Result<(), anyhow::Error> {
        if self.dest.exists() && !self.dest.is_symlink() {
            fs::rename(&self.dest, self.dest.with_extension("bak"))?;
        }
        Ok(())
    }
}

impl Display for Symlink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} -> {}",
            format_file(&self.source),
            format_link(&self.dest)
        )
    }
}
