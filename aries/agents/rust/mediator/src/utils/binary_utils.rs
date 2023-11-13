pub fn setup_logging() {
    let env = env_logger::Env::default().default_filter_or("info");
    env_logger::init_from_env(env);
}

pub fn load_dot_env() {
    let _ = dotenvy::dotenv();
}
