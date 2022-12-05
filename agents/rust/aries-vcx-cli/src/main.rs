use std::sync::{Arc, RwLock};

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

mod agent;
mod commands;
mod configuration;
mod logging;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = configuration::configure_app().await?;
    logging::init_logger(&config);
    let agent = agent::initialize_agent(&config).await?;
    let agentrc = Arc::new(RwLock::new(agent));
    let server = server::run_server(&config, agentrc.clone())?;
    let user_loop = commands::root_command_loop(agentrc);
    tokio::select! {
        res = server => match res {
            Ok(()) => {
                info!("Server finished");
                Ok(())
            }
            Err(err) => {
                info!("Server exited with error");
                Err(anyhow!("Error: {}", err))
            }
        },
        res = user_loop => match res {
            Ok(()) => {
                info!("User loop finished");
                Ok(())
            }
            Err(err) => {
                info!("User loop exited with error");
                Err(anyhow!("Error: {}", err))
            }
        }
    }
}
