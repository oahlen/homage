use anyhow::anyhow;
use std::{fs, path::PathBuf};

use crate::cli::Cli;

pub struct Context {
    pub dotfiles_dir: PathBuf,
    pub dry_run: bool,
    pub backup: bool,
}

impl Context {
    pub fn new(cli: &Cli) -> Result<Context, anyhow::Error> {
        Ok(Context {
            dotfiles_dir: resolve_dotfiles_dir(&cli.manifest)?,
            dry_run: cli.dry_run,
            backup: cli.backup,
        })
    }
}

fn resolve_dotfiles_dir(manifest: &str) -> Result<PathBuf, anyhow::Error> {
    let manifest_file = PathBuf::from(manifest).to_path_buf();

    if manifest_file.is_relative() {
        let canonical = fs::canonicalize(&manifest_file)?;
        let parent = canonical
            .parent()
            .ok_or_else(|| anyhow!("Failed to get parent directory of manifest file"))?;
        Ok(parent.to_path_buf())
    } else {
        Ok(manifest_file
            .parent()
            .ok_or_else(|| anyhow!("Failed to get parent directory of manifest file"))?
            .to_path_buf())
    }
}
