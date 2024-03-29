use clap::{Parser, Subcommand};
use std::time::{SystemTime, UNIX_EPOCH};
use totp_lite::{totp_custom, Sha1, DEFAULT_STEP};
use data_encoding::BASE32;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::io::{self, BufRead, BufReader, Write};
use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use dirs;

const FILE_CODEX: &str = "codex";
const PROJECT: &str = "hermes";
// Odyssea V 45
const TALARIA: &str = "immortales, aureos";
const DELIMETER: &str = ":";

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
    },
    /// Get codes for all/alias records
    Ls {
        #[clap(short = 'a', long)]
        alias: Option<String>,
        #[clap(short = 'u', long)]
        unencrypt: bool,
    },
    /// Show location of codex file
    Config {
    },
}

fn main() {
    // using dirs fn to get location of config directory
    let mut codex_path = dirs::config_dir()
        .expect("Failed to get config path");
    codex_path.push(PROJECT);
    codex_path.push(FILE_CODEX);
    
    let args = Args::parse();
    
    match &args.command {
        Commands::Add { alias, code, unencrypt } => {
            if code.is_some() && alias.is_some() {
                let code = code.as_ref().unwrap();
                let alias = alias.as_ref().unwrap();
                if !alias.contains(":") {
                    add(&codex_path, alias.as_str(), 
                        code.as_str(),
                        &unencrypt);
                } else {
                    println!("Don't use ':' in alias or code");
                    std::process::exit(1);
                }
            } else {
                println!("Please provide valid alias and code");
                println!("See 'hermes help add/update' for more information");
                std::process::exit(1);
            }
        },
        Commands::Remove { alias } => {
            if alias.is_some() {
                remove(&codex_path, alias.as_ref().unwrap().as_str());
            } else {
                println!("Error: no arguments for remove command");
                println!("See 'hermes help remove' for more information");
                std::process::exit(1);
            }
        },
        Commands::Update { alias, code, unencrypt } => {
            if alias.is_some() && code.is_some() {
                update_code(&codex_path, alias.as_ref().unwrap().as_str(), 
                    code.as_ref().unwrap().as_str(),
                    &unencrypt);
            }
        },
        Commands::Ls { alias, unencrypt } => {
            ls(&codex_path, alias, unencrypt);
        },
        Commands::Config { } => {
            if file_exists(&codex_path) {
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

fn add(codex_path: &PathBuf, alias: &str, code: &str, unencrypt: &bool) {
    // make code uppercase to solve the bug #1
    let binding = code.to_uppercase();
    let code = binding.as_str();

    // create a storage file if it does not exist
    let code_encrypted = if *unencrypt { 
        code.to_string()
    } else {
        crypt(true, 
            &code.to_string(), 
            &input_password())
    };
    let is_unencrypted = *unencrypt as u8;
    
    // validate code
    match BASE32.decode(code.as_bytes()) {
        Ok(_) => (),
        Err(e) => {
            println!("Error: {e}");
            std::process::exit(1);
        }
    }
    
    // sha for the future use
    let sha = "sha1";
    let data = format!("{}:{}:{}:{}\n", 
        alias,
        code_encrypted,
        is_unencrypted,
        sha);
    
    if file_exists(codex_path) {
        // check if alias already exists and return error message
        if alias_exists(&alias) == true {
            println!("Alias already exists, please select another one");
            std::process::exit(1);
        }
        let mut data_file = OpenOptions::new()
            .append(true)
            .open(codex_path)
            .expect("cannot open file");
        data_file
            .write(data.as_bytes())
            .expect("write failed");
    } else {
        if create_path(codex_path) {
            let msg = "Record saved to codex";
            write_to_file(codex_path, &data, &msg);
        }
    }
    let otp = generate_otp(code);
    println!("{otp}");
}

fn update_code(codex_path: &PathBuf, alias: &str, code: &str, unenc: &bool) {
    if remove(&codex_path, &alias) {
        add(&codex_path, &alias, &code, &unenc);
    } else {
        println!("No record for {alias} has been located in the codex file");
    }
}

fn remove(path: &PathBuf, alias: &str) -> bool {
    let lines = read_file_to_vec(&path);
    let mut data = "".to_owned();
    let mut f: bool = false;
    for l in lines {
        let x: Vec<&str> = l.split(DELIMETER).collect();
        if x[0] != alias {
            data = data + &l + "\n";
        } else {
            f = true;
        }
    }
    if f {
        let msg = format!("Record for {alias} is removed from codex");
        write_to_file(path, &data, &msg);
    }
    f
}

fn get(unenc: &bool, unencrypt_curr: &str, pass: &String, x: &str) -> String {
    let code = if unencrypt_curr == "0" {
        crypt(false,
            &x.to_string(),
            &pass)
    } else {
        x.to_string()
    };
    let otp = if *unenc && unencrypt_curr == "0" {
        "Cannot decrypt - provide a password".to_string()
    } else {
        generate_otp(code.as_str())
    };
    otp
}

fn ls(codex_path: &PathBuf, alias: &Option<String>, unencrypt: &bool) {
    let lines = read_file_to_vec(&codex_path);
    let pass = if *unencrypt {
        "".to_string()
    } else {
        input_password()
    };
    if alias.is_none() {
        println!("{0: <15} | {1: <15}", "Alias", "OTP");
    }
    for l in lines {
        let x: Vec<&str> = l.split(DELIMETER).collect();
        let alias_curr = x[0];
        let unencrypt_curr = x[2];
        if alias.is_some() {
            if alias.as_ref().unwrap() == alias_curr {
                let otp = get(&unencrypt, &unencrypt_curr, &pass, &x[1]);
                println!("{otp}");
                break;
            }
        } else {
            let otp = get(&unencrypt, &unencrypt_curr, &pass, &x[1]);
            println!("{0: <15} | {1: <15}", alias_curr, otp);
        }
    }
    std::process::exit(0);
}

fn read_file_to_vec(path: &PathBuf) -> Vec<String> {
    if file_exists(path) {
        let file = File::open(path)
            .expect("There is no codex file");
        let file = BufReader::new(file);
        file.lines()
            .map(|x| x.expect("Could not parse line"))
            .collect()
    } else {
        println!("{FILE_CODEX} file does not exist.\n\
            Please use hermes add command\n\
            or copy existing {FILE_CODEX} to a default location");
        std::process::exit(1);
    }
}

fn file_exists(path: &PathBuf) -> bool {
    Path::new(&path).exists()
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

fn write_to_file(path: &PathBuf, data: &str, msg: &str) {
    match std::fs::write(path, data) {
        Ok(_) => {
            println!("{msg}");
        }
        Err(e) => {
            eprintln!("Failed to save codex. Error: {e}");
        }
    }
}

fn create_path(path: &PathBuf) -> bool {
    let mut p: PathBuf = path.to_path_buf();
    p.pop();
    match std::fs::create_dir_all(&p) {
        Ok(_) => true,
        Err(_e) => false,
    }
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

    match BASE32.decode(x.as_bytes()) {
        Ok(x) => {

        let seconds: u64 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        totp_custom::<Sha1>(
            DEFAULT_STEP, 
            6, 
            &x, 
            seconds,)
        },
        Err(e) => format!("Error: {e:?}"),
    }
}

fn input_password() -> String {
    let password = rpassword::prompt_password("Enter password: ")
        .expect("Failed to read password");
    password
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
