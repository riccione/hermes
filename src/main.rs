use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod cmd;
mod file;
mod models;
mod otp;

#[derive(Parser)]
#[command(author, version, about, long_about = None)] // Read from Cargo.toml
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Args)]
struct EncryptArgs {
    /// WARNING: Store the secret in plain text. Use for debugging only.
    #[clap(short = 'u', long, verbatim_doc_comment)]
    unencrypt: bool,
    /// WARNING: Using this flag leaves password in shell history.
    #[clap(short = 'p', long, verbatim_doc_comment)]
    password: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Adds code to the hermes
    Add {
        #[clap(short = 'a', long)]
        alias: String,
        #[clap(short = 'c', long)]
        code: String,
        #[clap(flatten)]
        encryption: EncryptArgs,
    },
    /// Remove code from the hermes
    Remove {
        #[clap(short = 'a', long)]
        alias: String,
    },
    /// Update code by alias
    Update {
        #[clap(short = 'a', long)]
        alias: String,
        #[clap(short = 'c', long)]
        code: String,
        #[clap(flatten)]
        encryption: EncryptArgs,
    },
    /// Get codes for all/alias records
    Ls {
        #[clap(short = 'a', long)]
        alias: Option<String>,
        #[clap(flatten)]
        encryption: EncryptArgs,
    },
    /// Show location of codex file
    Config {},
    /// Migrate legacy codex format to JSON
    Migrate,
}

fn main() {
    let codex_path: PathBuf = file::get_codex_path();
    let args = Args::parse();

    if let Err(e) = run(args, codex_path) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run(args: Args, codex_path: PathBuf) -> Result<(), String> {
    match &args.command {
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
        Commands::Ls { alias, encryption } => {
            cmd::ls(
                &codex_path,
                alias,
                &encryption.unencrypt,
                &encryption.password,
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
