use clap::{Parser, Subcommand};
use std::time::{SystemTime, UNIX_EPOCH};
use totp_lite::{totp_custom, Sha512, DEFAULT_STEP};
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::io::{self, BufRead, Write};

const FILE_CODEX: &str = "codex";

#[derive(Debug)]
pub enum HermesError {
    NullArgs,
}

pub type HermesResult = Result<bool, HermesError>;

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
    },
    /// Get code by alias
    Get {
        #[clap(short = 'a', long)]
        alias: Option<String>,
    },
}

fn main() {
    let args = Args::parse();
    
    match &args.command {
        Commands::Add { alias, code } => {
            if code.is_some() && alias.is_some() {
                if !alias.as_ref().unwrap().contains(":") || 
                    !code.as_ref().unwrap().contains(":") {
                    add(&alias, code);
                } else {
                    println!("Don't use : in alias or code'");
                    std::process::exit(1);
                }
            } else {
                println!("Please provide valid alias and code");
                std::process::exit(1);
            }
        },
        Commands::Remove { alias } => {
            if alias.is_some() {
                remove(&alias.as_ref().unwrap().as_str());
            } else {
                println!("Error: no arguments for remove command");
                std::process::exit(1);
            }
        },
        Commands::Update { code, alias } => {
            if alias.is_some() && code.is_some() {
                update_code(code, alias);
            }
        },
        Commands::Get { alias } => {
            if alias.is_some() {
                get(&alias.as_ref().unwrap().as_str())
            } else {
                println!("Error: no arguments for get command");
                std::process::exit(1);
            }
        }
    };
}

fn add(alias: &Option<String>, code: &Option<String>) {
    let x = alias.clone().unwrap();
    let y = code.clone().unwrap();

    // create a storage file if it does not exist
    let data = format!("{}:{}\n", x, y);
    
    if file_exists() {
        // check if alias already exists and return error message
        if alias_exists(&x) == true {
            println!("Alias already exists, please select another one");
            std::process::exit(1);
        }
        let mut data_file = OpenOptions::new()
            .append(true)
            .open(FILE_CODEX)
            .expect("cannot open file");
        data_file
            .write(data.as_bytes())
            .expect("write failed");
    } else {
        write_to_file(&data);
    }
    match code {
        Some(code) => { generate_otp(&code); },
        None => { println!("Nothing to code") }
    }
}

fn update_code(code: &Option<String>, alias: &Option<String>) {
    remove(&alias.as_ref().unwrap().as_str());
    add(&code, &alias);
}

fn remove(alias: &str) {
    if file_exists() {
        // read codes file and search for alias
        let mut data = "".to_owned();
        if let Ok(lines) = read_lines(FILE_CODEX) {
            for line in lines {
                if let Ok(l) = line {
                    let x: Vec<_> = l.split(":").collect();
                    if x[0] != alias {
                        data = data + &l + "\n";
                    }
                }
            }
            write_to_file(&data);
        }
    } else {
        println!("Codex file does not exist. First add a code");
    }
}

fn get(alias: &str) {
    if file_exists() == false {
        println!("codex file does not exist");
    } else {    
        // read codes file and search for alias
        if let Ok(lines) = read_lines(FILE_CODEX) {
            for line in lines {
                if let Ok(l) = line {
                    let x: Vec<_> = l.split(":").collect();
                    if x[0] == alias {
                        generate_otp(x[1]);
                    }
                }
            }
        }
    }
}

fn file_exists() -> bool {
    Path::new(FILE_CODEX).exists()
}

fn alias_exists(alias: &str) -> bool {
    // read codes file and search for alias
    if let Ok(lines) = read_lines(FILE_CODEX) {
        for line in lines {
            if let Ok(l) = line {
                let x: Vec<_> = l.split(":").collect();
                if x[0] == alias {
                    return true;
                }
            }
        }
    }
    false
}

fn write_to_file(data: &str) {
    std::fs::write(FILE_CODEX, data).expect("write failed");
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn generate_otp(x: &str) {
    let password = x.as_bytes();
    let seconds: u64 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let result: String = totp_custom::<Sha512>(DEFAULT_STEP, 6, password, seconds);
    println!("{}", result);
}
