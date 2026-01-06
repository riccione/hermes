use crate::args::OutputFormat;
use crate::file;
use crate::models::Record;
use crate::otp;
use data_encoding::BASE32_NOPAD;
use std::io;
use std::path::PathBuf;

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

fn get(is_unencrypted: &bool, record: &Record, pass: &String) -> String {
    let secret = if !record.is_unencrypted {
        otp::crypt(false, &record.secret, pass)
    } else {
        record.secret.clone()
    };

    if *is_unencrypted && !record.is_unencrypted {
        "Cannot decrypt - provide a password".to_string()
    } else {
        match otp::generate_otp(secret.as_str()) {
            Ok(code) => code,
            Err(_) => "Error: Invalid secret or decryption failed".to_string(),
        }
    }
}

pub fn add(
    codex_path: &PathBuf,
    alias: &str,
    code: &str,
    unencrypt: &bool,
    password: &Option<String>,
) -> Result<(), String> {
    // make code uppercase to solve the bug #1
    let binding = code.to_uppercase().replace("=", "");
    let clean_code = binding.as_str();

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
    if let Err(e) = BASE32_NOPAD.decode(clean_code.as_bytes()) {
        return Err(format!("Error data-encoding BASE32: {e}"));
    }

    // encrypt if necessary
    let secret = if *unencrypt {
        clean_code.to_string()
    } else {
        let pass = get_effective_password(password);
        otp::crypt(true, &clean_code.to_string(), &pass)
    };

    // serialize and save
    let record = Record::new(alias.to_string(), secret.to_string(), *unencrypt);
    let json_data = serde_json::to_string(&record).map_err(|e| e.to_string())?;

    if file::file_exists(codex_path) {
        // check for duplicates
        if file::alias_exists(alias, codex_path) {
            return Err(format!("Alias '{}' already exists.", alias));
        }

        // create backup
        file::create_routine_backup(codex_path)
            .map_err(|e| format!("Warning: Backup failed: {}", e))?;

        file::write(codex_path, &json_data).map_err(|e| e.to_string())?;
    } else {
        file::create_path(codex_path).map_err(|e| e.to_string())?;

        file::write_to_file(codex_path, &json_data, "Record saved to new codex")
            .map_err(|e| e.to_string())?;
    }

    match otp::generate_otp(clean_code) {
        Ok(code) => println!("{code}"),
        Err(_) => println!("Error: failed to generate OTP"),
    }

    Ok(())
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
        let _ = add(codex_path, alias, &sanitized_code, unenc, &effective_pass);
        println!("Record for '{alias}' successfully updated.");
    }
}

pub fn remove(path: &PathBuf, alias: &str) -> bool {
    // create backup
    if let Err(e) = file::create_routine_backup(path) {
        eprintln!("Warning: Could not create backup file: {e}");
    }

    let lines = file::read_file_to_vec(&path).unwrap_or_default();
    let mut new_lines: Vec<String> = Vec::new();
    let mut found: bool = false;

    for l in lines {
        // hybrid parser from Record struct
        if let Some(record) = Record::from_line(&l) {
            if record.alias == alias {
                found = true;
                continue; // Skip this one
            }
        }
        new_lines.push(l);
    }

    if found {
        let data = new_lines.join("\n") + "\n";
        file::write_to_file(path, &data, "Record removed").expect("Failed to update codex");
        println!("Record for {alias} removed.");
    }
    found
}

pub fn ls(
    codex_path: &PathBuf,
    alias_filter: &Option<String>,
    unencrypt: &bool,
    password: &Option<String>,
    format: &OutputFormat,
) {
    let lines = file::read_file_to_vec(codex_path).unwrap_or_else(|_| {
        eprintln!("Codex not found.");
        std::process::exit(1);
    });

    let records: Vec<Record> = lines.iter().filter_map(|l| Record::from_line(l)).collect();

    // apply search filter
    let filtered_records: Vec<&Record> = records
        .iter()
        .filter(|r| match alias_filter {
            // partial match
            Some(f) => r.alias.to_lowercase().contains(&f.to_lowercase()),
            // display everything
            None => true,
        })
        .collect();

    if filtered_records.is_empty() {
        if alias_filter.is_some() {
            eprintln!("Alias not found.");
            std::process::exit(1);
        }
        return;
    }

    let needs_password = !*unencrypt && filtered_records.iter().any(|r| !r.is_unencrypted);

    let pass = if needs_password {
        get_effective_password(password)
    } else {
        String::new()
    };

    let rem = otp::get_remaining_seconds();

    let mut output_data = Vec::new();
    for record in &filtered_records {
        let otp = get(unencrypt, record, &pass);
        output_data.push((record, otp));
    }

    // handle output based on format
    match format {
        OutputFormat::Json => {
            let json_list: Vec<serde_json::Value> = output_data
                .iter()
                .map(|(r, otp)| {
                    serde_json::json!({
                        "alias": r.alias,
                        "otp": otp,
                        "remaining_secs": rem,
                        "is_encrypted": !r.is_unencrypted,
                        "created_at": r.created_at
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_list).unwrap());
        }
        OutputFormat::Table => {
            if alias_filter.is_some() && filtered_records.len() == 1 {
                // if specific alias requested, print only the OTP raw
                if let Some((_, otp)) = output_data.first() {
                    println!("{otp}");
                }
            } else {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                    .as_secs();

                println!("Current System Unix Time: {}", now);
                println!("Hint: If this is off by >30s, OTP codes will fail.");
                println!("Compare this at https://www.unixtimestamp.com if codes fail.\n");

                // print the full table
                println!("{0: <15} | {1: <10} | {2: <4}", "Alias", "OTP", "Rem");
                println!("{:-<15}-|-{:-<10}-|-{:-<4}", "", "", "");

                for (record, otp) in output_data {
                    println!("{0: <15} | {1: <10} | {2:}s", record.alias, otp, rem);
                }
            }
        }
    }
}

pub fn migrate(path: &PathBuf) -> io::Result<()> {
    // create backup
    let backup_path = file::create_snapshot_backup(path)?;
    println!("Backup created at {:?}", backup_path);

    // read and parse everything using the hybrid parser
    let lines = file::read_file_to_vec(path)?;
    let mut migrated_records = Vec::new();
    let mut count = 0;

    for line in lines {
        if let Some(record) = Record::from_line(&line) {
            // re-serialize to JSON string
            let json = serde_json::to_string(&record).expect("Failed to serialize");
            migrated_records.push(json);
            count += 1;
        }
    }

    // write back to the original file
    let new_content = migrated_records.join("\n") + "\n";
    file::write_to_file(path, &new_content, "Migration data prepared.")?;

    println!("Successfully migrated {count} records to JSON format.");
    Ok(())
}

pub fn rename(path: &PathBuf, old_alias: &str, new_alias: &str) -> Result<(), String> {
    if file::alias_exists(new_alias, path) {
        return Err(format!(
            "The alias '{}' already exists. Choose a unique name.",
            new_alias
        ));
    }

    // For Legacy file format
    if new_alias.contains(':') {
        return Err("The new alias cannot contain ':'".to_string());
    }

    // read the file
    let lines = file::read_file_to_vec(path).map_err(|e| e.to_string())?;
    let mut found = false;
    let mut updated_lines = Vec::new();

    // process lines
    for line in lines {
        if let Some(mut record) = Record::from_line(&line) {
            if record.alias == old_alias {
                record.alias = new_alias.to_string(); // change the name
                updated_lines.push(record.to_string()); // save the changed record
                found = true;
                continue;
            }
        }
        updated_lines.push(line);
    }

    if !found {
        return Err(format!("Alias '{}' not found.", old_alias));
    }

    // create backup
    if let Err(e) = file::create_routine_backup(path) {
        eprintln!("Warning: Could not create backup file: {e}");
    }

    // save back to file
    let data = updated_lines.join("\n") + "\n";
    file::write_to_file(path, &data, "Alias renamed").map_err(|e| e.to_string())?;

    println!("Successfully renamed '{}' to '{}'", old_alias, new_alias);
    Ok(())
}
