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

    match cli.action {
        Action::Install {
            source,
            target,
            dry_run,
            backup,
        } => {
            let context = Context::new(source, target, dry_run, backup, cli.verbose)?;
            context.install();
        }
        Action::Uninstall {
            source,
            target,
            dry_run,
        } => {
            let context = Context::new(source, target, dry_run, true, cli.verbose)?;
            context.uninstall();
        }
    }

    Ok(())
}
