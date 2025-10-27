use std::io::Write;

use crate::{
    action::Action,
    args::{ActionType, Args},
};

mod action;
mod args;
mod symlink;

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse_args();

    env_logger::builder()
        .filter_level(args.log_level())
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .init();

    match args.action {
        ActionType::Install {
            source,
            target,
            backup,
        } => {
            let context = Action::new(source, target, args.dry_run, backup)?;
            context.install();
        }
        ActionType::Uninstall { source, target } => {
            let context = Action::new(source, target, args.dry_run, true)?;
            context.uninstall();
        }
    }

    Ok(())
}
