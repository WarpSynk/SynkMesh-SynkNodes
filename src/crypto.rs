use sha2::{Digest, Sha256};
use rand::Rng;
use hex;

pub fn generate_node_id() -> String {
    let mut rng = rand::thread_rng();
    let n: u128 = rng.gen();
    let mut hasher = Sha256::new();
    hasher.update(n.to_le_bytes());
    let res = hasher.finalize();
    hex::encode(res)
}

pub fn hash_data(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}