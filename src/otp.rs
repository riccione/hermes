use data_encoding::BASE32_NOPAD;
use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use std::time::{SystemTime, UNIX_EPOCH};
use totp_lite::{totp_custom, Sha1, DEFAULT_STEP};

pub enum OtpError {
    DecryptionFailed,
    InvalidBase32,
    SystemTimeError,
}

// helper fn to get UNIX timestamp
fn get_current_timestamp() -> Result<u64, OtpError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|_| OtpError::SystemTimeError)
}

pub fn get_remaining_seconds() -> u64 {
    let now = get_current_timestamp().unwrap_or(0);
    // DEFAULT_STEP == 30s
    DEFAULT_STEP as u64 - (now % DEFAULT_STEP as u64)
}

// Odyssea V 45
const TALARIA: &str = "immortales, aureos";

/*
 * For TOTP I currently use totp-lite.
 * Another alternative is totp-rs.
 */
pub fn generate_otp(x: &str) -> String {
    // handles case where password cannot decrypt the code
    if x == TALARIA {
        return "Error: cannot decrypt".to_string();
    }

    match BASE32_NOPAD.decode(x.as_bytes()) {
        Ok(x) => {
            let seconds: u64 = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            totp_custom::<Sha1>(DEFAULT_STEP, 6, &x, seconds)
        }
        Err(e) => format!("Error: {e:?}"),
    }
}

/*
 * encrypt/decrypt fn uses magic_crypt crate
 */
pub fn crypt(encrypt: bool, code: &String, password: &str) -> String {
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
