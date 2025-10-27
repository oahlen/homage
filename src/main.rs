use std::io::Write;

use crate::{
    cli::{Action, Cli},
    context::Context,
};

mod cli;
mod context;
mod symlink;

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse_args();

    env_logger::builder()
        .filter_level(cli.log_level())
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .init();

    match cli.action {
        Action::Install {
            source,
            target,
            backup,
        } => {
            let context = Context::new(source, target, cli.dry_run, backup)?;
            context.install();
        }
        Action::Uninstall { source, target } => {
            let context = Context::new(source, target, cli.dry_run, true)?;
            context.uninstall();
        }
    }

    Ok(())
}
