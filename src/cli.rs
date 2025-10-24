use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use log::LevelFilter;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(value_enum)]
    pub action: Action,
    pub directory: PathBuf,
    #[arg(long, value_name = "dry-run")]
    pub dry_run: bool,
    #[arg(long, value_name = "backup")]
    pub backup: bool,
    #[arg(short = 'v', long, value_name = "verbosity", value_enum, default_value_t = Verbosity::Default)]
    pub verbosity: Verbosity,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Action {
    Install,
    Uninstall,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Verbosity {
    Default,
    Verbose,
    Debug,
}

impl Verbosity {
    pub fn to_log_level(self) -> LevelFilter {
        match self {
            Self::Default => LevelFilter::Warn,
            Self::Verbose => LevelFilter::Info,
            Self::Debug => LevelFilter::Trace,
        }
    }
}
