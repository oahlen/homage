use anyhow::anyhow;
use std::{env, fs, path::PathBuf};

use crate::symlink::Symlink;

pub struct Context {
    source: PathBuf,
    target: PathBuf,
    dry_run: bool,
    backup: bool,
    verbose: bool,
}

impl Context {
    pub fn new(
        source: PathBuf,
        target: Option<PathBuf>,
        dry_run: bool,
        backup: bool,
        verbose: bool,
    ) -> Result<Context, anyhow::Error> {
        Ok(Context {
            source: resolve_directory(&source)?,
            target: resolve_directory(&target.clone().unwrap_or(home_dir()))?,
            dry_run,
            backup,
            verbose,
        })
    }

    pub fn install(&self) {
        println!("Installing dotfiles from {}", self.source.display());

        if self.dry_run {
            println!("Running in dry-run mode");
        }

        for entry in walkdir::WalkDir::new(&self.source)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let rel_path = entry.path().strip_prefix(&self.source).unwrap();
            let dest = self.target.join(rel_path);
            self.install_dotfile_entry(Symlink {
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
                self.uninstall_dotfile_entry(&file.to_path_buf());
            }
        }
    }

    fn install_dotfile_entry(&self, symlink: Symlink) {
        println!("Installing {}", symlink);

        if self.dry_run {
            return;
        }

        if let Some(parent) = symlink.dest.parent() {
            fs::create_dir_all(parent).ok();
        }

        if self.backup {
            match symlink.backup(self.verbose) {
                Ok(_) => (),
                Err(_) => {
                    eprintln!("Failed to backup file {}", symlink.dest.display());
                }
            }
        }

        match &symlink.create() {
            Ok(result) => {
                if !*result {
                    println!("Symlink {} already installed", symlink)
                }
            }
            Err(err) => {
                eprintln!("Failed to create symlink {}: {}", symlink, err);
            }
        };
    }

    fn uninstall_dotfile_entry(&self, dest: &PathBuf) {
        println!("Uninstalling {}", dest.display());

        if self.dry_run {
            return;
        }

        fs::remove_file(dest).ok();

        let bak = dest.with_extension("bak");

        if bak.exists() {
            if self.verbose {
                println!("Restoring backup {}", bak.display());
            }
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

fn home_dir() -> PathBuf {
    env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| {
        panic!("Could not determine $HOME");
    })
}
