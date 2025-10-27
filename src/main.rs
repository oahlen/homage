use log::warn;

use crate::{
    action::Action,
    args::{ActionType, Args},
};

mod action;
mod args;
mod format;
mod symlink;

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse_args();
    args.init_logger();

    if args.dry_run {
        warn!("Running in dry-run mode");
    }

    match args.action {
        ActionType::Install {
            source,
            target,
            backup,
        } => Action::new(source, target, args.dry_run, backup)?.install(),
        ActionType::Uninstall { source, target } => {
            Action::new(source, target, args.dry_run, true)?.uninstall()
        }
    }

    Ok(())
}
