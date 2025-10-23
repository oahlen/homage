use anyhow::anyhow;
use clap::{Parser, ValueEnum};
use core::str;
use std::{env, fmt::Display, fs, os::unix::fs as unix_fs, path::PathBuf, process::exit};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(value_enum)]
    action: Action,
    manifest: String,
    #[arg(long, value_name = "dry-run")]
    dry_run: bool,
    #[arg(long, value_name = "backup")]
    backup: bool,
    #[arg(short = 'v', long, value_name = "verbose")]
    verbose: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Action {
    Install,
    Uninstall,
}

#[derive(serde::Deserialize)]
struct Manifest {
    all: std::collections::HashMap<String, String>,
}

struct Context {
    dotfiles_dir: PathBuf,
    dry_run: bool,
    backup: bool,
    verbose: bool,
}

struct Symlink {
    source: PathBuf,
    dest: PathBuf,
}

impl Symlink {
    pub fn create(&self) -> Result<bool, anyhow::Error> {
        if self.dest.is_symlink() {
            let current_target = fs::read_link(&self.dest)?;
            if current_target == *self.source {
                return Ok(false);
            }

            fs::remove_file(&self.dest)?
        }

        unix_fs::symlink(&self.source, &self.dest)?;
        Ok(true)
    }
}

impl Display for Symlink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.source.display(), self.dest.display())
    }
}

fn home_dir() -> PathBuf {
    env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| {
        eprintln!("Could not determine $HOME");
        exit(1);
    })
}

fn backup(file: &PathBuf) {
    if file.exists() && !file.is_symlink() {
        println!(
            "Backing up existing {} to {}.bak",
            file.display(),
            file.display()
        );
        fs::rename(file, file.with_extension("bak")).ok();
    }
}

fn install_dotfile_entry(symlink: Symlink, context: &Context) {
    println!("Installing {}", symlink);

    if context.dry_run {
        return;
    }

    if let Some(parent) = symlink.dest.parent() {
        fs::create_dir_all(parent).ok();
    }

    if context.backup {
        backup(&symlink.dest);
    }

    match &symlink.create() {
        Ok(result) => {
            if *result {
                println!("Installed {}", symlink)
            } else if context.verbose {
                println!("Symlink {} already installed", symlink)
            }
        }
        Err(err) => {
            eprintln!("Failed to create symlink {}: {}", symlink, err);
        }
    };
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

fn install_dotfile(source_file: &str, dest_file: &str, context: &Context) {
    let source_path = context.dotfiles_dir.join(source_file);
    let dest_path = resolve_dest(dest_file);

    if !source_path.exists() {
        eprintln!(
            "Error: Source file {} does not exist.",
            source_path.display()
        );
        return;
    }

    if source_path.is_dir() {
        for entry in walkdir::WalkDir::new(&source_path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let rel_path = entry.path().strip_prefix(&source_path).unwrap();
            let dest = dest_path.join(rel_path);
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
                source: source_path,
                dest: dest_path,
            },
            context,
        );
    }
}

fn uninstall_dotfile_entry(dest: &PathBuf, context: &Context) {
    println!("Uninstalling {}", dest.display());

    if context.dry_run {
        return;
    }

    fs::remove_file(dest).ok();

    if context.verbose {
        println!("Removed symlink {}", dest.display());
    }

    let bak = dest.with_extension("bak");

    if bak.exists() {
        println!("Restoring backup {}", bak.display());
        fs::rename(&bak, dest).ok();
    }
}

fn uninstall_dotfile(dest_file: &str, context: &Context) {
    let dest_path = resolve_dest(dest_file);

    if dest_path.is_dir() {
        for entry in walkdir::WalkDir::new(&dest_path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().is_symlink())
        {
            let file = entry.path();
            uninstall_dotfile_entry(&file.to_path_buf(), context);
        }
    } else if dest_path.is_symlink() {
        uninstall_dotfile_entry(&dest_path, context);
    } else {
        println!(
            "File {} is not a symlink, skipping ...",
            dest_path.display()
        );
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

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    let manifest_str = fs::read_to_string(&cli.manifest).unwrap_or_else(|_| {
        eprintln!("Manifest {} not found.", &cli.manifest);
        exit(1);
    });

    let manifest: Manifest = toml::from_str(&manifest_str).unwrap_or_else(|e| {
        eprintln!("Failed to parse manifest: {}", e);
        exit(1);
    });

    let files: Vec<(String, String)> = manifest.all.into_iter().collect();

    let context = Context {
        dotfiles_dir: resolve_dotfiles_dir(&cli.manifest)?,
        dry_run: cli.dry_run,
        backup: cli.backup,
        verbose: cli.verbose,
    };

    println!(
        "Installing dotfiles from {}",
        context.dotfiles_dir.display()
    );

    if context.dry_run {
        println!("Running in dry-run mode");
    }

    match cli.action {
        Action::Install => {
            for (source, dest) in &files {
                install_dotfile(source, dest, &context);
            }
        }
        Action::Uninstall => {
            for (_, dest) in &files {
                uninstall_dotfile(dest, &context);
            }
        }
    }

    Ok(())
}
