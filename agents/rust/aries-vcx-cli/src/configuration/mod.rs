mod app_args;
mod app_config;
mod init_config;
mod kdf;
mod setup;

pub use app_config::AppConfig;

pub async fn configure_app() -> anyhow::Result<app_config::AppConfig> {
    let app_args = app_args::parse_cli_args();
    let init_config = init_config::load_init_config(&app_args)?;
    let trustee_seed = setup::assure_trustee_seed(&init_config).await?;
    let genesis_file = setup::assure_genesis_file(&init_config).await?;
    let config = app_config::load_app_config(init_config, trustee_seed, genesis_file);
    Ok(config)
}
