use clap::{Parser, Subcommand};
use std::time::{SystemTime, UNIX_EPOCH};
use totp_lite::{totp_custom, Sha512, DEFAULT_STEP};
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
    /// Get code by alias
    Get {
        #[clap(short = 'a', long)]
        alias: Option<String>,
        #[clap(short = 'p', long)]
        password: bool,
    },
    /// Get codes for all records
    Ls {
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
                    add(alias.as_ref().unwrap().as_str(), 
                        pass.as_str());
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
                update_code(alias.as_ref().unwrap().as_str(), 
                    pass.as_str());
            }
        },
        Commands::Get { alias, password } => {
            if alias.is_some() {
                get(alias.as_ref().unwrap().as_str(),
                    password);
            } else {
                println!("Error: no arguments for get command");
                std::process::exit(1);
            }
        },
        Commands::Ls { password } => {
            ls(password);
        }
    };
}

fn add(alias: &str, code: &str) {
    // create a storage file if it does not exist
    let data = format!("{}:{}\n", alias, code);
    
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

fn update_code(alias: &str, code: &str) {
    remove(&alias);
    add(&alias, &code);
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

fn get(alias: &str, password: &bool) {
    if file_exists() == false {
        println!("codex file does not exist");
    } else {    
        // read codes file and search for alias
        if let Ok(lines) = read_lines(FILE_CODEX) {
            for line in lines {
                if let Ok(l) = line {
                    let x: Vec<_> = l.split(":").collect();
                    if x[0] == alias {
                        let code = if *password {
                            crypt(false,
                                &x[1].to_string(), 
                                &input_password())
                        } else {
                            x[1].to_string()
                        };
                        let otp  = if code.to_string() == TALARIA {
                            "error: cannot decrypt".to_string()
                        } else {
                            generate_otp(code.as_str())
                        };
                        println!("{otp}");
                    }
                }
            }
        }
    }
}

fn _ls(password: &bool) {
    // read file
    if file_exists() {
        let pass = if *password {
            input_password()
        } else {
            "".to_string()
        };
        if let Ok(lines) = read_lines(FILE_CODEX) {
            println!("Alias\tOTP");
            for line in lines {
                if let Ok(l) = line {
                    let x: Vec<_> = l.split(":").collect();
                    let alias = x[0];
                    let code = if *password {
                        crypt(false,
                            &x[1].to_string(), 
                            &pass)
                    } else {
                        x[1].to_string()
                    };
                    let otp = if code.to_string() == TALARIA {
                        "error: cannot decrypt".to_string()
                    } else {
                        generate_otp(code.as_str())
                    };                 
                    println!("{alias}\t{otp}");
                }
            }
        }
    } else {
        println!("codex file does not exist");
        std::process::exit(1);
    }
}

fn ls(is_encrypted: &bool) {
    let pass = if *is_encrypted {
        input_password()
    } else {
        "".to_string()
    };
    let lines = read_file_to_vec();
    println!("Alias\tOTP");
    for l in lines {
        let x: Vec<&str> = l.split(DELIMETER).collect();
        let alias = x[0];
        let code = if *is_encrypted {
            crypt(false,
                &x[1].to_string(),
                &pass)
        } else {
            x[1].to_string()
        };
        let otp = if code.to_string() == TALARIA {
            "Error: cannot decrypt".to_string()
        } else {
            generate_otp(code.as_str())
        };
        println!("{alias}\t{otp}");
    }
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
    let password = x.as_bytes();
    let seconds: u64 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    totp_custom::<Sha512>(DEFAULT_STEP, 6, password, seconds)
}

fn input_password() -> String {
    let mut input = String::new();
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
        mcrypt.encrypt_str_to_base64(code)
    } else {
        let decrypted = match mcrypt.decrypt_base64_to_string(code) {
            Ok(decrypted) => decrypted,
            Err(_) => TALARIA.to_string(),
        };
        decrypted
    }
}
