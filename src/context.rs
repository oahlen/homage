use anyhow::anyhow;
use std::{fs, path::PathBuf};

use crate::cli::Cli;

pub struct Context {
    pub directory: PathBuf,
    pub dry_run: bool,
    pub backup: bool,
}

impl Context {
    pub fn new(cli: &Cli) -> Result<Context, anyhow::Error> {
        Ok(Context {
            directory: resolve_directory(&cli.directory)?,
            dry_run: cli.dry_run,
            backup: cli.backup,
        })
    }
}

fn resolve_directory(path: &PathBuf) -> Result<PathBuf, anyhow::Error> {
    if !path.exists() {
        return Err(anyhow!("Directory {} does not exist", path.display()));
    }

    let resovled = if path.is_relative() {
        match fs::canonicalize(path) {
            Ok(result) => Ok(result.to_path_buf()),
            Err(_) => Err(anyhow!("Unable to resolve directory {}", path.display())),
        }
    } else {
        Ok(path.to_path_buf())
    }?;

    if !resovled.is_dir() {
        return Err(anyhow!("Input {} is not a directory", path.display()));
    }

    Ok(resovled)
}
