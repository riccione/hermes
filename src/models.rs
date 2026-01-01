use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

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

    /// Attempts to parse line into a Record, supports both JSON and Legacy (text)
    pub fn from_line(line: &str) -> Option<Self> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        // try JSON first
        if let Ok(record) = serde_json::from_str::<Record>(trimmed) {
            return Some(record);
        }

        // fallback to Legacy (text), divider is colon
        let parts: Vec<&str> = trimmed.split(':').collect();
        if parts.len() >= 4 {
            return Some(Record {
                alias: parts[0].to_string(),
                secret: parts[1].to_string(),
                is_unencrypted: parts[2] == "1",
                algorithm: parts[3].to_string(),
                created_at: 0,
            });
        }
        None
    }
}
