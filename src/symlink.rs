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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_dir(prefix: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("homage_symlink_{}_{}", prefix, ts));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn create_file(path: &PathBuf, name: &str, content: &[u8]) -> PathBuf {
        let source = path.join(name);
        File::create(&source).unwrap().write_all(content).unwrap();
        source
    }

    #[test]
    fn is_installed_false_for_regular_file() {
        let base = unique_dir("regular");

        let source = create_file(&base, "source.txt", b"src");
        let target = create_file(&base, "target.txt", b"tgt");

        let link = Symlink::new(source, target);

        assert!(!link.is_installed());
    }

    #[test]
    fn install_and_uninstall_symlink() {
        let base = unique_dir("install");
        let source = create_file(&base, "source.txt", b"src");

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
        let base = unique_dir("exists");

        let source = create_file(&base, "source.txt", b"src");
        let target = create_file(&base, "target.txt", b"tgt");

        let link = Symlink::new(source, target.clone());

        assert!(link.exists());
    }

    #[test]
    fn is_installed_true_after_install() {
        let base = unique_dir("installed");

        let source = create_file(&base, "source.txt", b"src");
        let target = base.join("link.txt");

        let link = Symlink::new(source.clone(), target.clone());
        link.install();

        assert!(link.is_installed());
    }
}
