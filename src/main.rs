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

    let skip_confirmation = args.dry_run || args.no_confirm;

    match args.action {
        ActionType::Install {
            source,
            target,
            backup,
        } => Action::new(source, target, args.dry_run, backup, skip_confirmation)?.install(),
        ActionType::Uninstall { source, target } => {
            Action::new(source, target, args.dry_run, true, skip_confirmation)?.uninstall()
        }
        ActionType::Clean { source, target } => {
            Action::new(source, target, args.dry_run, true, skip_confirmation)?.clean()
        }
    }

    Ok(())
}
