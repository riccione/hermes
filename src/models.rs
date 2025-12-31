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
