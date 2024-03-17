mod constants

pub fn hash_to_binary(hash: &[u8]) -> String {
    let mut result = String::default();
    for c in hash {
        result.push_str(&format!("{:b}", c));
    }
    result
}
