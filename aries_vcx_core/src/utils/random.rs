use rand::distributions::Alphanumeric;
use rand::Rng;

pub fn generate_random_name() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(25)
        .collect::<String>()
}

pub fn generate_random_seed() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .collect::<String>()
}

pub fn generate_random_did() -> String {
    crate::global::settings::DEFAULT_DID.to_string()
}
