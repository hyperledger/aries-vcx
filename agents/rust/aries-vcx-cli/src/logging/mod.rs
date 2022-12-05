use crate::configuration::AppConfig;

pub fn init_logger(config: &AppConfig) {
    env_logger::Builder::from_default_env()
        .parse_filters(config.log_level())
        .init();
}
