use clap::Parser;

use crate::{
    cli::{Action, Cli},
    context::Context,
};

mod cli;
mod context;
mod symlink;

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();
    let context = Context::new(&cli)?;

    match cli.action {
        Action::Install => context.install(),
        Action::Uninstall => context.uninstall(),
    }

    Ok(())
}
