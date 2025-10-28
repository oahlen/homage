use std::io::Write;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use colored::Colorize;
use log::{Level, LevelFilter};

#[derive(Debug, Parser, Clone)]
#[command(author, version, about, long_about)]
pub struct Args {
    #[clap(subcommand)]
    pub action: ActionType,

    /// Whether to perform a dry run of the specified action. Does not perform any file system
    /// operations.
    #[arg(long, value_name = "dry-run", global = true)]
    pub dry_run: bool,

    /// Whether to skip user confirmation of the action to perform.
    #[arg(long, value_name = "no-confirm", global = true)]
    pub no_confirm: bool,

    /// Prints more detailed information of the performed actions.
    #[clap(short = 'v', long = "verbosity", action = clap::ArgAction::Count, global = true)]
    verbosity: u8,

    /// Whether to only print error messages, disbables the 'verbosity' arg.
    #[arg(long, value_name = "no-confirm", global = true)]
    pub quiet: bool,
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

        /// Whether to backup files that would otherwise be overridden by the specified action.
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

    pub fn init_logger(&self) {
        env_logger::builder()
            .filter_level(self.log_level())
            .format(|buf, record| {
                let message = format!("{}", record.args());
                match record.level() {
                    Level::Error => writeln!(buf, "{}", message.red()),
                    Level::Warn => writeln!(buf, "{}", message.yellow()),
                    _ => writeln!(buf, "{}", record.args()),
                }
            })
            .init();
    }

    fn log_level(&self) -> LevelFilter {
        if self.quiet {
            return LevelFilter::Error;
        }

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
