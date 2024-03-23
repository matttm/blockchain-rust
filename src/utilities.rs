use crate::constants::DIFFICULTY_PREFIX;
use serde_json::json;
use sha2::{Digest, Sha256};

pub fn hash_to_binary(hash: &[u8]) -> String {
    let mut result = String::default();
    for c in hash {
        result.push_str(&format!("{:b}", c));
    }
    result
}

pub fn calculate_hash(
    id: u64,
    timestamp: i64,
    previous_hash: &str,
    data: &str,
    nonce: u64,
) -> Vec<u8> {
    let data = json!({
        "id": id,
        "previous_hash": previous_hash,
        "data": data,
        "timestamp": timestamp,
        "nonce": nonce
    });
    let mut hasher = Sha256::new();
    hasher.update(data.to_string().as_bytes());
    hasher.finalize().as_slice().to_owned()
}
