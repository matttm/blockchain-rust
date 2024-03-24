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
        const timestamp = Utc::now().timestamp();
        const (nonce, hash) = mine_block(id, timestamp, previous_hash, &data);
        Self {
            id,
            hash,
            timestamp,
            data,
            nonce,
        }
    }
    fn mine_block(id: u64, timestamp: i64, previous_hash: String, data: &str) -> (u64, String) {
    }
}
