use log::warn;

use crate::{
    action::Action,
    args::{ActionType, Args},
    utils::cache_path,
};

mod action;
mod args;
mod cache;
mod format;
mod manifest;
mod symlink;
mod tests;
mod utils;

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse_args();
    args.init_logger();

    if args.dry_run {
        warn!("Running in dry-run mode");
    }

    let skip_confirmation = args.dry_run || args.no_confirm;
    let cache_file = cache_path()?;

    match args.action {
        ActionType::Install { manifest } => {
            Action::new(manifest, cache_file, args.dry_run, skip_confirmation).install()
        }
        ActionType::Uninstall { manifest } => {
            Action::new(manifest, cache_file, args.dry_run, skip_confirmation).uninstall()
        }
    }
}
