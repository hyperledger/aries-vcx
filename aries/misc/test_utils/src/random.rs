use rand::{distributions::Alphanumeric, Rng};

use crate::settings::DEFAULT_DID;

pub fn generate_random_schema_name() -> String {
    String::from_utf8(
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(25)
            .collect(),
    )
    .unwrap()
}

pub fn generate_random_name() -> String {
    String::from_utf8(
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(25)
            .collect(),
    )
    .unwrap()
}

pub fn generate_random_seed() -> String {
    String::from_utf8(
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .collect(),
    )
    .unwrap()
}

pub fn generate_random_schema_version() -> String {
    format!(
        "{}.{}",
        rand::thread_rng().gen::<u32>(),
        rand::thread_rng().gen::<u32>()
    )
}

pub fn generate_random_did() -> String {
    DEFAULT_DID.to_string()
}
