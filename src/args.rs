use std::path::PathBuf;

use clap::{Parser, Subcommand};
use log::LevelFilter;

#[derive(Debug, Parser, Clone)]
#[command(author, version, about, long_about)]
pub struct Args {
    #[clap(subcommand)]
    pub action: ActionType,

    /// Whether to perform a dry run of the specified operation. Does not perform any file system
    /// operations.
    #[arg(long, value_name = "dry-run", global = true)]
    pub dry_run: bool,

    /// Prints more detailed information of the performed actions.
    #[clap(short = 'v', long = "verbosity", action = clap::ArgAction::Count, global = true)]
    verbosity: u8,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ActionType {
    /// Installs the specified dotifles directory by symlinking all the files into the target
    /// directory. Only files are symlinked while subdirectories are created as needed in the
    /// target directory.
    Install {
        /// The dotfiles directory to install symlinks from.
        source: PathBuf,

        /// The root directory where to create the symlinks, defaults to the home directory is not
        /// specified.
        target: Option<PathBuf>,

        /// Whether to backup files that would otherwise be overridden by the specified operation.
        /// Backed up files will be the original file with '.bak' appended to the end.
        #[arg(long, value_name = "backup")]
        backup: bool,
    },
    /// Uninstall the specified dotfiles directory by removing the symlinks in the target directory
    /// that points back to it. If the backup option is specified any existing backed up file is
    /// restored.
    Uninstall {
        /// The dotfiles directory to uninstall symlinks from.
        source: PathBuf,

        /// The root directory where to remove the symlinks, defaults to the home directory is not
        /// specified.
        target: Option<PathBuf>,
    },
}

impl Args {
    pub fn parse_args() -> Args {
        let mut cli = Args::parse();
        cli.verbosity = std::cmp::min(3, cli.verbosity);
        cli
    }

    pub fn log_level(&self) -> LevelFilter {
        if self.dry_run {
            return match self.verbosity {
                3 => LevelFilter::Trace,
                _ => LevelFilter::Debug,
            };
        }

        match self.verbosity {
            0 => LevelFilter::Warn,
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            3 => LevelFilter::Trace,
            _ => LevelFilter::Warn,
        }
    }
}
