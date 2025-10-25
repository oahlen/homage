use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(value_enum)]
    pub action: Action,

    pub source: PathBuf,

    pub target: Option<PathBuf>,

    #[arg(long, value_name = "dry-run")]
    pub dry_run: bool,

    #[arg(long, value_name = "backup")]
    pub backup: bool,

    #[arg(short = 'v', long, value_name = "verbose")]
    pub verbose: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Action {
    Install,
    Uninstall,
}
