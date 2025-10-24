use clap::Parser;
use log::{error, info, warn};
use std::{env, io::Write, path::PathBuf};

use crate::{
    cli::{Action, Cli},
    context::Context,
    dotfile::Dotfile,
};

mod cli;
mod context;
mod dotfile;
mod symlink;

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    env_logger::builder()
        .filter_level(cli.verbosity.to_log_level())
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .init();

    let context = Context::new(&cli)?;

    info!("Installing dotfiles from {}", context.directory.display());

    if context.dry_run {
        warn!("Running in dry-run mode");
    }

    match cli.action {
        Action::Install => match Dotfile::new(&context.directory, home_dir()) {
            Ok(entry) => entry.install(&context),
            Err(err) => error!("{}", err),
        },
        Action::Uninstall => match Dotfile::new(&context.directory, home_dir()) {
            Ok(entry) => entry.uninstall(&context),
            Err(err) => error!("{}", err),
        },
    }

    Ok(())
}

fn home_dir() -> PathBuf {
    env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| {
        panic!("Could not determine $HOME");
    })
}
