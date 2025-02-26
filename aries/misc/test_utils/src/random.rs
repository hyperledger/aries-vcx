use rand::{distr::Alphanumeric, Rng};

use crate::settings::DEFAULT_DID;

pub fn generate_random_schema_name() -> String {
    String::from_utf8(rand::rng().sample_iter(&Alphanumeric).take(25).collect()).unwrap()
}

pub fn generate_random_name() -> String {
    String::from_utf8(rand::rng().sample_iter(&Alphanumeric).take(25).collect()).unwrap()
}

pub fn generate_random_seed() -> String {
    String::from_utf8(rand::rng().sample_iter(&Alphanumeric).take(32).collect()).unwrap()
}

pub fn generate_random_schema_version() -> String {
    format!(
        "{}.{}",
        rand::rng().random::<u32>(),
        rand::rng().random::<u32>()
    )
}

pub fn generate_random_did() -> String {
    DEFAULT_DID.to_string()
}
