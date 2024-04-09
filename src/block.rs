use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::constants::DIFFICULTY_PREFIX;
use crate::utilities::{calculate_hash, hash_to_binary};

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
        let (nonce, hash): (u64, String) = Block::mine_block(id, timestamp, &previous_hash, &data);
        Self {
            id,
            hash,
            timestamp,
            previous_hash,
            data,
            nonce,
        }
    }
    fn mine_block(id: u64, timestamp: i64, previous_hash: &str, data: &str) -> (u64, String) {
        println!("Attempting to mine block {}", id);
        let nonce = 0;
        loop {
            let hash = calculate_hash(id, timestamp, &previous_hash, data, nonce);
            let binary = hash_to_binary(&hash);
            if binary.starts_with(DIFFICULTY_PREFIX) {
                let encoded = hex::encode(&hash);
                println!("Mined block {id}! nonce: {nonce}, hash: {encoded}, binary hash: {binary}");
                return (nonce, encoded);
            }
        }
    }
}
