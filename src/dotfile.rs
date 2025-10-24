use anyhow::anyhow;
use log::{debug, error, info};
use std::{env, fs, path::PathBuf};

use crate::{context::Context, symlink::Symlink};

pub struct Dotfile {
    pub source: PathBuf,
    pub dest: PathBuf,
}

impl Dotfile {
    pub fn new(
        source_file: &str,
        dest_file: &str,
        context: &Context,
    ) -> Result<Dotfile, anyhow::Error> {
        let source = context.dotfiles_dir.join(source_file);

        if !source.exists() {
            return Err(anyhow!(
                "Error: Source file {} does not exist.",
                source.display()
            ));
        }

        Ok(Dotfile {
            source,
            dest: Dotfile::resolve_dest(dest_file),
        })
    }

    fn resolve_dest(dest: &str) -> PathBuf {
        let expanded = if dest.starts_with('~') {
            let home = home_dir();
            home.join(dest.trim_start_matches('~').trim_start_matches('/'))
        } else {
            PathBuf::from(dest)
        };

        if expanded.is_absolute() {
            expanded
        } else {
            home_dir().join(expanded)
        }
    }

    pub fn install(&self, context: &Context) {
        if !self.source.exists() {
            error!(
                "Error: Source file {} does not exist.",
                self.source.display()
            );
            return;
        }

        if self.source.is_dir() {
            for entry in walkdir::WalkDir::new(&self.source)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
            {
                let rel_path = entry.path().strip_prefix(&self.source).unwrap();
                let dest = self.dest.join(rel_path);
                install_dotfile_entry(
                    Symlink {
                        source: entry.path().to_path_buf(),
                        dest,
                    },
                    context,
                );
            }
        } else {
            install_dotfile_entry(
                Symlink {
                    source: self.source.to_path_buf(),
                    dest: self.dest.to_path_buf(),
                },
                context,
            );
        }
    }

    pub fn uninstall(&self, context: &Context) {
        if self.dest.is_dir() {
            for entry in walkdir::WalkDir::new(&self.dest)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.path().is_symlink())
            {
                let file = entry.path();
                uninstall_dotfile_entry(&file.to_path_buf(), context);
            }
        } else if self.dest.is_symlink() {
            uninstall_dotfile_entry(&self.dest, context);
        } else {
            info!(
                "File {} is not a symlink, skipping ...",
                self.dest.display()
            );
        }
    }
}

fn install_dotfile_entry(symlink: Symlink, context: &Context) {
    debug!("Installing {}", symlink);

    if context.dry_run {
        return;
    }

    if let Some(parent) = symlink.dest.parent() {
        fs::create_dir_all(parent).ok();
    }

    if context.backup {
        match symlink.backup() {
            Ok(_) => (),
            Err(_) => {
                error!("Failed to backup file {}", symlink.dest.display());
            }
        }
    }

    match &symlink.create() {
        Ok(result) => {
            if *result {
                info!("Installed {}", symlink)
            } else {
                info!("Symlink {} already installed", symlink)
            }
        }
        Err(err) => {
            error!("Failed to create symlink {}: {}", symlink, err);
        }
    };
}

fn uninstall_dotfile_entry(dest: &PathBuf, context: &Context) {
    info!("Uninstalling {}", dest.display());

    if context.dry_run {
        return;
    }

    fs::remove_file(dest).ok();

    info!("Removed symlink {}", dest.display());

    let bak = dest.with_extension("bak");

    if bak.exists() {
        info!("Restoring backup {}", bak.display());
        fs::rename(&bak, dest).ok();
    }
}

fn home_dir() -> PathBuf {
    env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| {
        panic!("Could not determine $HOME");
    })
}
