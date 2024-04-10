use clap::{Parser, Subcommand};
//use data_encoding::BASE32;
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

#[derive(Subcommand)]
enum Commands {
    /// Adds code to the hermes
    Add {
        #[clap(short = 'a', long)]
        alias: Option<String>,
        #[clap(short = 'c', long)]
        code: Option<String>,
        #[clap(short = 'u', long)]
        unencrypt: bool,
        #[clap(short = 'p', long)]
        password: Option<String>,
    },
    /// Remove code from the hermes
    Remove {
        #[clap(short = 'a', long)]
        alias: Option<String>,
    },
    /// Update code by alias
    Update {
        #[clap(short = 'a', long)]
        alias: Option<String>,
        #[clap(short = 'c', long)]
        code: Option<String>,
        #[clap(short = 'u', long)]
        unencrypt: bool,
        #[clap(short = 'p', long)]
        password: Option<String>,
    },
    /// Get codes for all/alias records
    Ls {
        #[clap(short = 'a', long)]
        alias: Option<String>,
        #[clap(short = 'u', long)]
        unencrypt: bool,
        #[clap(short = 'p', long)]
        password: Option<String>,
    },
    /// Show location of codex file
    Config {
    },
}

fn main() {
    let codex_path: PathBuf = file::get_codex_path();

    let args = Args::parse();
    
    match &args.command {
        Commands::Add { alias, code, unencrypt, password } => {
            if code.is_some() && alias.is_some() {
                let code = code.as_ref().unwrap();
                let alias = alias.as_ref().unwrap();
                if !alias.contains(":") {
                    cmd::add(&codex_path, alias.as_str(), 
                        code.as_str(),
                        &unencrypt, password);
                } else {
                    println!("Don't use ':' in alias or code");
                    std::process::exit(1);
                }
            } else {
                eprintln!("Please provide valid alias and code");
                eprintln!("See 'hermes help add/update' for more information");
                std::process::exit(1);
            }
        },
        Commands::Remove { alias } => {
            if alias.is_some() {
                cmd::remove(&codex_path, alias.as_ref().unwrap().as_str());
            } else {
                println!("Error: no arguments for remove command");
                println!("See 'hermes help remove' for more information");
                std::process::exit(1);
            }
        },
        Commands::Update { alias, code, unencrypt, password } => {
            if alias.is_some() && code.is_some() {
                cmd::update_code(&codex_path, alias.as_ref().unwrap().as_str(), 
                    code.as_ref().unwrap().as_str(),
                    &unencrypt, password);
            }
        },
        Commands::Ls { alias, unencrypt, password } => {
            cmd::ls(&codex_path, alias, unencrypt, password);
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
