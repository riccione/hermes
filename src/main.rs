use clap::{Parser, Subcommand};
use std::path::{PathBuf};

mod cmd;
mod file;
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
    Config {
    },
}

fn main() {
    let codex_path: PathBuf = file::get_codex_path();

    let args = Args::parse();
    
    match &args.command {
        Commands::Add { alias, code, encryption } => {
            if !alias.contains(":") {
                cmd::add(&codex_path, alias.as_str(), 
                    code.as_str(),
                    &encryption.unencrypt,
                    &encryption.password);
            } else {
                println!("Don't use ':' in alias or code");
                std::process::exit(1);
            }
        },
        Commands::Remove { alias } => {
            cmd::remove(&codex_path, alias.as_str());
        },
        Commands::Update { alias, code, encryption } => {
            cmd::update_code(&codex_path, alias.as_str(), 
                code.as_str(),
                &encryption.unencrypt,
                &encryption.password);
        },
        Commands::Ls { alias, encryption } => {
            cmd::ls(&codex_path, alias,
                &encryption.unencrypt,
                &encryption.password);
        },
        Commands::Config { } => {
            if file::file_exists(&codex_path) {
                let p = codex_path.into_os_string().into_string();
                match p {
                    Ok(x) => { println!("{x}"); },
                    Err(e) => { println!("Error: {:?}", e); }
                }
            } else {
                println!("Codex file does not exists in the default location");
            }
        }
    };
}
