use rand::{distributions::Alphanumeric, Rng};

pub fn random_seed() -> String {
    rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}
