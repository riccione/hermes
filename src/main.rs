use std::time::{SystemTime, UNIX_EPOCH};
use totp_lite::{totp_custom, Sha512, DEFAULT_STEP};

fn main() {
    let password: &[u8] = b"secret";
    let seconds: u64 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let result: String = totp_custom::<Sha512>(DEFAULT_STEP, 6, password, seconds);
    println!("{}", result);
}
