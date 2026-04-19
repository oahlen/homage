use std::{fmt::Display, fs, os::unix::fs as unix_fs, path::PathBuf};

use log::{error, info};

use crate::format::{fmt_error, fmt_file, fmt_link};

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

        if self.target.is_symlink() && !self.exists() {
            info!("Overwriting broken symlink at {}", fmt_error(&self.target));

            match fs::remove_file(&self.target) {
                Ok(_) => {}
                Err(err) => {
                    error!("Failed to cleanup broken symlink {}: {}", self, err);
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::{self};

    use crate::tests::tests::{test_dir, write_file};

    #[test]
    fn is_installed_false_for_regular_file() {
        let base = test_dir("regular");

        let source = write_file(&base, "source.txt", "src");
        let target = write_file(&base, "target.txt", "target");

        let link = Symlink::new(source, target);

        assert!(!link.is_installed());
    }

    #[test]
    fn install_and_uninstall_symlink() {
        let base = test_dir("install");
        let source = write_file(&base, "source.txt", "src");

        let target_dir = base.join("target");
        let target = target_dir.join("source.txt");

        let link = Symlink::new(source.clone(), target.clone());
        assert!(!link.is_installed());

        link.install();

        assert!(target.is_symlink());
        assert!(link.is_installed());

        link.uninstall();

        assert!(!target.exists());
    }

    #[test]
    fn exists_true_for_existing_file() {
        let base = test_dir("exists");

        let source = write_file(&base, "source.txt", "src");
        let target = write_file(&base, "target.txt", "target");

        let link = Symlink::new(source, target.clone());

        assert!(link.exists());
    }

    #[test]
    fn install_replaces_broken_symlink() {
        let base = test_dir("broken_symlink");

        let old_source = base.join("old_source.txt");
        let new_source = write_file(&base, "new_source.txt", "new content");
        let target = base.join("link.txt");

        // Create a broken symlink by pointing to a non-existent file
        unix_fs::symlink(&old_source, &target).unwrap();
        assert!(target.is_symlink());
        assert!(!target.exists()); // broken: destination doesn't exist

        // Install should replace the broken symlink
        let link = Symlink::new(new_source.clone(), target.clone());
        link.install();

        assert!(target.is_symlink());
        assert!(link.is_installed());
        assert_eq!(fs::read_to_string(&target).unwrap(), "new content");
    }

    #[test]
    fn is_installed_true_after_install() {
        let base = test_dir("installed");

        let source = write_file(&base, "source.txt", "src");
        let target = base.join("link.txt");

        let link = Symlink::new(source.clone(), target.clone());
        link.install();

        assert!(link.is_installed());
    }
}
