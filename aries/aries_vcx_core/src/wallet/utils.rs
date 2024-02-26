use public_key::Key;
use rand::{distributions::Alphanumeric, Rng};

#[allow(dead_code)]
pub fn random_seed() -> String {
    rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

#[allow(dead_code)]
pub fn did_from_key(key: Key) -> String {
    key.base58()[0..16].to_string()
}
