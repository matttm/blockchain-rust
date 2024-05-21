use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub id: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: i64,
    pub data: String,
    pub nonce: u64,
}

impl Block {
    pub fn new(id: u64, previous_hash: String, data: String) -> Self {
        let timestamp: i64 = Utc::now().timestamp();
        let (nonce, hash): (u64, String) = (1, String::from("test"));
        Self {
            id,
            hash,
            timestamp,
            previous_hash,
            data,
            nonce,
        }
    }
}
