use rand::Rng;
use rand::distributions::Alphanumeric;

pub fn generate_random_schema_name() -> String {
    rand::thread_rng().sample_iter(&Alphanumeric).take(25).collect::<String>()
}


pub fn generate_random_name() -> String {
    rand::thread_rng().sample_iter(&Alphanumeric).take(25).collect::<String>()
}

pub fn generate_random_seed() -> String {
    rand::thread_rng().sample_iter(&Alphanumeric).take(32).collect::<String>()
}

pub fn generate_random_schema_version() -> String {
    format!("{}.{}", rand::thread_rng().gen::<u32>().to_string(),
                                     rand::thread_rng().gen::<u32>().to_string())
}

pub fn generate_random_did() -> String {
    crate::settings::DEFAULT_DID.to_string()
}
