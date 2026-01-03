use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)] // Read from Cargo.toml
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
}

#[derive(clap::Args)]
pub struct EncryptArgs {
    /// WARNING: Store the secret in plain text. Use for debugging only.
    #[clap(short = 'u', long, verbatim_doc_comment)]
    pub unencrypt: bool,
    /// WARNING: Using this flag leaves password in shell history.
    #[clap(short = 'p', long, verbatim_doc_comment)]
    pub password: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
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
    /// Rename alias
    Rename {
        old_alias: String,
        new_alias: String,
    },
    /// Get codes for all/alias records
    Ls {
        #[clap(short = 'a', long)]
        alias: Option<String>,
        #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
        #[clap(flatten)]
        encryption: EncryptArgs,
    },
    /// Show location of codex file
    Config {},
    /// Migrate legacy codex format to JSON
    Migrate,
}
