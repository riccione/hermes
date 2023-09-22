use clap::{Parser, Subcommand};
use std::time::{SystemTime, UNIX_EPOCH};
use totp_lite::{totp_custom, Sha512, DEFAULT_STEP};


/*
Usage: 
cargo run -- add -c secret -a alias
cargo run -- add --code secret --alias alias
*/

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
        #[clap(short = 'c', long)]
        code: Option<String>,
        #[clap(short = 'a', long)]
        alias: Option<String>,
    },
    Remove {
        #[clap(short = 'a', long)]
        alias: Option<String>,
    },
    Update {
        #[clap(short = 'c', long)]
        code: Option<String>,
        #[clap(short = 'a', long)]
        alias: Option<String>,
    },
    Get {
        #[clap(short = 'a', long)]
        alias: Option<String>,
    },
}

fn main() {
    let args = Args::parse();
    
    //let x, y =
    match &args.command {
        Commands::Add { code, alias } => add(code, alias),
        Commands::Remove { alias } => remove(alias),
        Commands::Update { code, alias } => update(code, alias),
        Commands::Get { alias } => get(alias),
        _ => println!("Please check help and about, the args are wrong"),
    };
    /*
    match x {
        Some(x) => {
            generate_otp(&x);
            //println!("x is {x}");
        }
        None => {
            println!("Nothing");
        }
    }
    println!("{x:?}");
    */
}

fn add(code: &Option<String>, alias: &Option<String>) {
    println!("{code:?}{alias:?}");
}

fn update(code: &Option<String>, alias: &Option<String>) {
    println!("{code:?}{alias:?}");
}

fn remove(alias: &Option<String>) {
    println!("{alias:?}");
}

fn get(alias: &Option<String>) {
    println!("{alias:?}");
}

fn generate_otp(x: &String) {
    let password = x.as_bytes();
    let seconds: u64 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let result: String = totp_custom::<Sha512>(DEFAULT_STEP, 6, password, seconds);
    println!("{}", result);
}
