use std::{fmt::Display, fs, os::unix::fs as unix_fs, path::PathBuf};

use crate::format::{fmt_file, fmt_link};

pub struct Symlink {
    pub source: PathBuf,
    pub target: PathBuf,
}

impl Symlink {
    pub fn create(&self) -> Result<bool, anyhow::Error> {
        if self.target.is_symlink() {
            let current_target = fs::read_link(&self.target)?;
            if current_target == *self.source {
                return Ok(false);
            }

            fs::remove_file(&self.target)?
        } else if self.target.exists() {
            fs::remove_file(&self.target)?
        }

        unix_fs::symlink(&self.source, &self.target)?;
        Ok(true)
    }

    pub fn backup(&self) -> Result<(), anyhow::Error> {
        if self.target.exists() && !self.target.is_symlink() {
            fs::rename(&self.target, self.target.with_extension("bak"))?;
        }
        Ok(())
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
