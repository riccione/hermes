use data_encoding::BASE32;
use std::path::{PathBuf};
use crate::file;
use crate::otp;

const DELIMETER: &str = ":";

pub fn add(codex_path: &PathBuf, alias: &str, code: &str, unencrypt: &bool, password: &Option<String>) {
    // make code uppercase to solve the bug #1
    let binding = code.to_uppercase();
    let code = binding.as_str();

    // create a storage file if it does not exist
    let code_encrypted = if *unencrypt { 
        code.to_string()
    } else {

        match password {
            Some(p) => {
                otp::crypt(true, &code.to_string(), p)
            },
            _ => {
                otp::crypt(true, 
                    &code.to_string(), 
                    &input_password())
            }
        }
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
            eprintln!("Alias already exists, please select another one");
            std::process::exit(1);
        }
        file::write(codex_path, &data);
    } else {
        if file::create_path(codex_path) {
            let msg = "Record saved to codex";
            file::write_to_file(codex_path, &data, &msg);
        }
    }
    let otp = otp::generate_otp(code);
    println!("{otp}");
}

pub fn update_code(codex_path: &PathBuf, alias: &str, code: &str, unenc: &bool, 
    password: &Option<String>) {
    if remove(&codex_path, &alias) {
        add(&codex_path, &alias, &code, &unenc, &password);
    } else {
        println!("No record for {alias} has been located in the codex file");
    }
}

pub fn remove(path: &PathBuf, alias: &str) -> bool {
    let lines = file::read_file_to_vec(&path);
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
        let msg = format!("Record for {alias} has been removed from codex");
        file::write_to_file(path, &data, &msg);
    }
    f
}

fn get(unenc: &bool, unencrypt_curr: &str, pass: &String, x: &str) -> String {
    let code = if unencrypt_curr == "0" {
        otp::crypt(false,
            &x.to_string(),
            &pass)
    } else {
        x.to_string()
    };
    let otp = if *unenc && unencrypt_curr == "0" {
        "Cannot decrypt - provide a password".to_string()
    } else {
        otp::generate_otp(code.as_str())
    };
    otp
}

pub fn ls(codex_path: &PathBuf, alias: &Option<String>, unencrypt: &bool, password: &Option<String>) {
    let lines = file::read_file_to_vec(&codex_path);
    let pass: String = if *unencrypt {
        "".to_string()
    } else {
        
        match password {
            Some(p) => p.to_string(),
            _ => input_password()
        }
        
        // input_password()
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

fn input_password() -> String {
    let password = rpassword::prompt_password("Enter password: ")
        .expect("Failed to read password");
    password
}
