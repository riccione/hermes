use std::path::PathBuf;

mod args;
mod cmd;
mod file;
mod models;
mod otp;

use args::{Cli, Commands};
use clap::Parser;

fn main() {
    let codex_path: PathBuf = file::get_codex_path();
    let cli = Cli::parse();

    if let Err(e) = run(cli.command, codex_path) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run(command: Commands, codex_path: PathBuf) -> Result<(), String> {
    match command {
        Commands::Add {
            alias,
            code,
            encryption,
        } => {
            if alias.contains(":") {
                eprintln!("Error: Don't use ':' in alias.");
                std::process::exit(1);
            }

            cmd::add(
                &codex_path,
                alias.as_str(),
                code.as_str(),
                &encryption.unencrypt,
                &encryption.password,
            );
        }
        Commands::Remove { alias } => {
            if !cmd::remove(&codex_path, alias.as_str()) {
                eprintln!("Error: Could not find alias '{}'", alias);
            }
        }
        Commands::Update {
            alias,
            code,
            encryption,
        } => {
            cmd::update_code(
                &codex_path,
                alias.as_str(),
                code.as_str(),
                &encryption.unencrypt,
                &encryption.password,
            );
        }
        Commands::Rename {
            old_alias,
            new_alias,
        } => {
            cmd::rename(&codex_path, old_alias.as_str(), new_alias.as_str())?;
        }
        Commands::Ls {
            alias,
            format,
            encryption,
        } => {
            cmd::ls(
                &codex_path,
                &alias,
                &encryption.unencrypt,
                &encryption.password,
                &format,
            );
        }
        Commands::Config {} => {
            if codex_path.exists() {
                println!("{}", codex_path.display());
            } else {
                eprintln!("Codex file does not exists at {}", codex_path.display());
            }
        }
        Commands::Migrate => {
            if let Err(e) = cmd::migrate(&codex_path) {
                eprintln!("Migration failed: {e}");
                std::process::exit(1);
            }
        }
    }
    Ok(())
}
