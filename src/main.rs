use clap::{Parser, Subcommand};

use std::time::{SystemTime, UNIX_EPOCH};
use totp_lite::{totp_custom, Sha1, DEFAULT_STEP};
use koibumi_base32 as base32;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::io::{self, BufRead, BufReader, Write};
use magic_crypt::{new_magic_crypt, MagicCryptTrait};

const FILE_CODEX: &str = "codex";
// Odyssea V 45
const TALARIA: &str = "immortales, aureos";
const DELIMETER: &str = ":";

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
        #[clap(short = 'p', long)]
        password: bool,
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
        #[clap(short = 'p', long)]
        password: bool,
    },
    /// Get codes for all/alias records
    Ls {
        #[clap(short = 'a', long)]
        alias: Option<String>,
        #[clap(short = 'p', long)]
        password: bool,
    },
}

fn main() {
    let args = Args::parse();

    match &args.command {
        Commands::Add { alias, code, password } => {
            if code.is_some() && alias.is_some() {
                if !alias.as_ref().unwrap().contains(":") || 
                    !code.as_ref().unwrap().contains(":") {
                    let pass = if *password { 
                        crypt(true, 
                            code.as_ref().unwrap(), 
                            &input_password())
                    } else {
                        code.as_ref().unwrap().to_string()
                    };
                    let is_encrypted = *password as u8;
                    add(alias.as_ref().unwrap().as_str(), 
                        pass.as_str(),
                        &is_encrypted);
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
                remove(alias.as_ref().unwrap().as_str());
            } else {
                println!("Error: no arguments for remove command");
                std::process::exit(1);
            }
        },
        Commands::Update { alias, code, password } => {
            if alias.is_some() && code.is_some() {
                let pass = if *password {
                    crypt(true, 
                        code.as_ref().unwrap(), 
                        &input_password())
                } else {
                    code.as_ref().unwrap().to_string()
                };
                let is_encrypted = *password as u8;
                update_code(alias.as_ref().unwrap().as_str(), 
                    pass.as_str(),
                    &is_encrypted);
            }
        },
        Commands::Ls { alias, password } => {
            ls(alias, password);
        }
    };
}

fn add(alias: &str, code: &str, is_encrypted: &u8) {
    // create a storage file if it does not exist
    let data = format!("{}:{}:{}\n", alias, code, is_encrypted);
    
    if file_exists() {
        // check if alias already exists and return error message
        if alias_exists(&alias) == true {
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
    let otp = generate_otp(code);
    println!("{otp}");
}

fn update_code(alias: &str, code: &str, is_encrypted: &u8) {
    remove(&alias);
    add(&alias, &code, &is_encrypted);
}

fn remove(alias: &str) {
    let lines = read_file_to_vec();
    let mut data = "".to_owned();
    for l in lines {
        let x: Vec<&str> = l.split(DELIMETER).collect();
        if x[0] != alias {
            data = data + &l + "\n";
        }
    }
    write_to_file(&data);
}

fn get(is_password: &bool, is_encrypted: &str, pass: &String, x: &str) -> String {
    let code = if *is_password && is_encrypted == "1" {
        crypt(false,
            &x.to_string(),
            &pass)
    } else {
        x.to_string()
    };
    let otp = if !*is_password && is_encrypted == "1" {
        "Cannot decrypt - provide a password".to_string()
    } else {
        generate_otp(code.as_str())
    };
    otp
}

fn ls(alias: &Option<String>, is_password: &bool) {
    let pass = if *is_password {
        input_password()
    } else {
        "".to_string()
    };
    let lines = read_file_to_vec();
    if alias.is_none() {
        println!("Alias\tOTP");
    }
    for l in lines {
        let x: Vec<&str> = l.split(DELIMETER).collect();
        let alias_curr = x[0];
        let is_encrypted = x[2];
        if alias.is_some() {
            if alias.as_ref().unwrap() == alias_curr {
                let otp = get(&is_password, &is_encrypted, &pass, &x[1]);
                println!("{otp}");
                break;
            }
        } else {
            let otp = get(&is_password, &is_encrypted, &pass, &x[1]);
            println!("{alias_curr}\t{otp}");
        }
    }
    std::process::exit(0);
}

fn read_file_to_vec() -> Vec<String> {
    if file_exists() {
        let file = File::open(FILE_CODEX)
            .expect("There is no codex file");
        let file = BufReader::new(file);
        file.lines()
            .map(|x| x.expect("Could not parse line"))
            .collect()
    } else {
        println!("{FILE_CODEX} file does not exist");
        std::process::exit(1);
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

fn generate_otp(x: &str) -> String {
    // handles case where password cannot decrypt the code
    if x == TALARIA {
        return "Error: cannot decrypt".to_string()
    }

    let password = &base32::decode(x.to_string().trim().to_lowercase())
        .expect("Error: Invalid base32 character");
    let seconds: u64 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    totp_custom::<Sha1>(
        DEFAULT_STEP, 
        6, 
        password, 
        seconds,)
}

fn input_password() -> String {
    let mut input = String::new();
    println!("Enter password: ");
    let stdin = io::stdin();
    stdin.read_line(&mut input).expect("Could not read password");
    input
}

/*
 * encrypt/decrypt fn uses magic_crypt crate 
 */
fn crypt(encrypt: bool, code: &String, password: &str) -> String {
    let mcrypt = new_magic_crypt!(password.trim(), 256);
    if encrypt {
            mcrypt
            .encrypt_str_to_base64(code)
    } else {
        let decrypted = match mcrypt.decrypt_base64_to_string(code) {
            Ok(decrypted) => decrypted,
            Err(_) => TALARIA.to_string(),
        };
        decrypted
    }
}
