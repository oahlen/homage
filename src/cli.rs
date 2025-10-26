use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about)]
pub struct Cli {
    #[clap(subcommand)]
    pub action: Action,

    /// The dotfiles directory to create symlinks from.
    pub source: PathBuf,

    /// The root directory where to create the symlinks, defaults to the home directory is not
    /// specified.
    pub target: Option<PathBuf>,

    /// Whether to perform a dry run of the specified operation. Does not perform any file system
    /// operations.
    #[arg(long, value_name = "dry-run")]
    pub dry_run: bool,

    /// Whether to backup files that would otherwise be overridden by the specified operation.
    /// Backed up files will be the original file with '.bak' appended to the end.
    #[arg(long, value_name = "backup")]
    pub backup: bool,

    /// Prints more detailed information of the performed actions.
    #[arg(short = 'v', long, value_name = "verbose")]
    pub verbose: bool,
}

#[derive(Debug, Clone, Subcommand, Default)]
pub enum Action {
    /// Installs the specified dotifles directory by symlinking all the files into the target
    /// directory. Only files are symlinked while subdirectories are created as needed in the
    /// target directory.
    #[default]
    Install,
    /// Uninstall the specified dotfiles directory by removing the symlinks in the target directory
    /// that points back to it. If the backup option is specified any existing backed up file is
    /// restored.
    Uninstall,
}
