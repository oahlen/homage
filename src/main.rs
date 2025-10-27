use std::io::Write;

use crate::{
    args::{Action, Args},
    context::Context,
};

mod args;
mod context;
mod symlink;

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse_args();

    env_logger::builder()
        .filter_level(args.log_level())
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .init();

    match args.action {
        Action::Install {
            source,
            target,
            backup,
        } => {
            let context = Context::new(source, target, args.dry_run, backup)?;
            context.install();
        }
        Action::Uninstall { source, target } => {
            let context = Context::new(source, target, args.dry_run, true)?;
            context.uninstall();
        }
    }

    Ok(())
}
