use clap::Parser;
use core::str;
use std::{fs, process::exit};

use crate::{
    cli::{Action, Cli},
    context::Context,
    dotfile::Dotfile,
};

mod cli;
mod context;
mod dotfile;
mod symlink;

#[derive(serde::Deserialize)]
struct Manifest {
    all: std::collections::HashMap<String, String>,
}

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    let manifest_str = fs::read_to_string(&cli.manifest).unwrap_or_else(|_| {
        eprintln!("Manifest {} not found.", &cli.manifest);
        exit(1);
    });

    let manifest: Manifest = toml::from_str(&manifest_str).unwrap_or_else(|e| {
        eprintln!("Failed to parse manifest: {}", e);
        exit(1);
    });

    let files: Vec<(String, String)> = manifest.all.into_iter().collect();

    let context = Context::new(&cli)?;

    println!(
        "Installing dotfiles from {}",
        context.dotfiles_dir.display()
    );

    if context.dry_run {
        println!("Running in dry-run mode");
    }

    match cli.action {
        Action::Install => {
            for (source, dest) in &files {
                match Dotfile::new(source, dest, &context) {
                    Ok(entry) => entry.install(&context),
                    Err(err) => eprintln!("{}", err),
                }
            }
        }
        Action::Uninstall => {
            for (source, dest) in &files {
                match Dotfile::new(source, dest, &context) {
                    Ok(entry) => entry.uninstall(&context),
                    Err(err) => eprintln!("{}", err),
                }
            }
        }
    }

    Ok(())
}
