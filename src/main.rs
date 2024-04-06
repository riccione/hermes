use clap::{Parser, Subcommand};
use std::time::{SystemTime, UNIX_EPOCH};
use totp_lite::{totp_custom, Sha1, DEFAULT_STEP};
use data_encoding::BASE32;
use std::path::{PathBuf};
use magic_crypt::{new_magic_crypt, MagicCryptTrait};

mod config;
mod file;

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
    // let mut codex_path = dirs::config_dir()
    //     .expect("Failed to get config path");
    // codex_path.push(file::PROJECT);
    // codex_path.push(file::FILE_CODEX);
    let codex_path: PathBuf = file::get_codex_path();

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
    
    if file::file_exists(codex_path) {
        // check if alias already exists and return error message
        if file::alias_exists(&alias, &codex_path) == true {
            println!("Alias already exists, please select another one");
            std::process::exit(1);
        }
        file::write(codex_path, &data);
        // let mut data_file = OpenOptions::new()
        //     .append(true)
        //     .open(codex_path)
        //     .expect("cannot open file");
        // data_file
        //     .write(data.as_bytes())
        //     .expect("write failed");
    } else {
        if file::create_path(codex_path) {
            let msg = "Record saved to codex";
            file::write_to_file(codex_path, &data, &msg);
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
    let lines = file::read_file_to_vec(&path);
    let mut data = "".to_owned();
    let mut f: bool = false;
    for l in lines {
        let x: Vec<&str> = l.split(config::DELIMETER).collect();
        if x[0] != alias {
            data = data + &l + "\n";
        } else {
            f = true;
        }
    }
    if f {
        let msg = format!("Record for {alias} is removed from codex");
        file::write_to_file(path, &data, &msg);
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
    let lines = file::read_file_to_vec(&codex_path);
    let pass = if *unencrypt {
        "".to_string()
    } else {
        input_password()
    };
    if alias.is_none() {
        println!("{0: <15} | {1: <15}", "Alias", "OTP");
    }
    for l in lines {
        let x: Vec<&str> = l.split(config::DELIMETER).collect();
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

fn generate_otp(x: &str) -> String {
    // handles case where password cannot decrypt the code
    if x == config::TALARIA {
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
            Err(_) => config::TALARIA.to_string(),
        };
        decrypted
    }
}
