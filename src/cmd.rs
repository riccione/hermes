use crate::file;
use crate::otp;
use data_encoding::BASE32_NOPAD;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

const DELIMETER: &str = ":";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Record {
    pub alias: String,
    pub secret: String,
    pub is_unencrypted: bool, // only for DEBUG, store secret unencrypted
    pub algorithm: String,
    pub created_at: u64, // Unix timestamp in sec
}

impl Record {
    pub fn new(alias: String, secret: String, is_unencrypted: bool) -> Self {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        Self {
            alias,
            secret,
            is_unencrypted,
            algorithm: "sha1".to_string(),
            created_at: since_the_epoch,
        }
    }
}

fn input_password() -> String {
    let password = rpassword::prompt_password("Enter password: ").expect("Failed to read password");
    password
}

fn get_effective_password(password: &Option<String>) -> String {
    password
        .clone()
        .or_else(|| std::env::var("HERMES_PASSWORD").ok())
        .unwrap_or_else(input_password)
}

fn get(unenc: &bool, unencrypt_curr: &str, pass: &String, x: &str) -> String {
    let code = if unencrypt_curr == "0" {
        otp::crypt(false, &x.to_string(), &pass)
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

pub fn add(
    codex_path: &PathBuf,
    alias: &str,
    code: &str,
    unencrypt: &bool,
    password: &Option<String>,
) {
    // make code uppercase to solve the bug #1
    let binding = code.to_uppercase();
    let code = binding.as_str();

    // create a storage file if it does not exist
    let code_encrypted = if *unencrypt {
        code.to_string()
    } else {
        let pass = get_effective_password(password);
        otp::crypt(true, &code.to_string(), &pass)
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
     *
     * The issue #3 was related to BASE32 method from data-encoding crate.
     * BASE32 has auto padding
     * BASE32_NOPAD - no padding
     * I did test code and notice that some codes produce errors - Invalid length with BASE32,
     * switching to BASE32_NOPAD fixed the issue.
     * It is interesting, I tried 2 crates: base32 and data-encoding.
     * base32 produces same results with padding set to true/false?!
     * data-encoding - different results.
     * I stick for now with data-encoding only because it more popular.
     */
    match BASE32_NOPAD.decode(code.as_bytes()) {
        Ok(_) => (),
        Err(e) => {
            println!("Error data-encoding BASE32: {e}");
            std::process::exit(1);
        }
    }

    // sha for the future use
    let sha = "sha1";
    let data = format!("{}:{}:{}:{}\n", alias, code_encrypted, is_unencrypted, sha);

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

pub fn update_code(
    codex_path: &PathBuf,
    alias: &str,
    new_code: &str,
    unenc: &bool,
    password: &Option<String>,
) {
    // Validate the NEW code first (Fail fast)
    let sanitized_code = new_code.to_uppercase().replace("=", "");
    if let Err(e) = BASE32_NOPAD.decode(sanitized_code.as_bytes()) {
        eprintln!("Error: Invalid Base32 code provided. Update aborted. ({e})");
        return;
    }

    // Check if the alias even exists before we do anything else
    if !file::alias_exists(alias, codex_path) {
        eprintln!("No record for '{alias}' has been located in the codex file.");
        return;
    }
    
    // Resolve password once (if needed)
    let effective_pass = if *unenc {
        None
    } else {
        Some(get_effective_password(password))
    };
    
    // Do the swap
    if remove(codex_path, alias) {
        add(codex_path, alias, &sanitized_code, unenc, &effective_pass);
        println!("Record for '{alias}' successfully updated.");
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

pub fn ls(
    codex_path: &PathBuf,
    alias: &Option<String>,
    unencrypt: &bool,
    password: &Option<String>,
) {
    let lines = file::read_file_to_vec(&codex_path);
    let pass: String = if *unencrypt {
        "".to_string()
    } else {
        get_effective_password(password)
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
