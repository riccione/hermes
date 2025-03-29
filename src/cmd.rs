use data_encoding::BASE32_NOPAD;
use std::path::{PathBuf};
use crate::file;
use crate::otp;

const DELIMETER: &str = ":";

fn input_password() -> String {
    let password = rpassword::prompt_password("Enter password: ")
        .expect("Failed to read password");
    password
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
    
    /* Validate code - check if it is a valid base32
     * Here I beleive it is necessary to add some explanation for base32 and TOTP.
     * Overtime I forgot what it does and my code comments are not good :/
     * This is how it works:
     * 1. user enters username and password on website
     * 2. website asks for second factor (TOTP)
     * 3. TOTP app generates 6-digit (usually) code based on the secret key and current time using
     *    SHA1 (usually):
     *   - Secret key (code) is base32 encoded
     *   - base32 should be valid
     *   - base32 based on RFC 4648 https://datatracker.ietf.org/doc/html/rfc4648
     *   - it uses alphabet of 32 digits: A-Z, 2-7
     *   - in some cases padding (=) used - the length of the string % 8 (every 5 bits to 8 bit
     *   output)
     *   - correct base32 encoded string should decode without errors
     * 4. users enters the TOTP code into the website
     * 5. website verifies the code using the same secret key and TOTP generation algorithm (SHA1)
     * 6. success or fail
    */
    match BASE32_NOPAD.decode(code.as_bytes()) {
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

pub fn ls(codex_path: &PathBuf, alias: &Option<String>, unencrypt: &bool, password: &Option<String>) {
    let lines = file::read_file_to_vec(&codex_path);
    let pass: String = if *unencrypt {
        "".to_string()
    } else {
        match password {
            Some(p) => p.to_string(),
            _ => input_password()
        }
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
