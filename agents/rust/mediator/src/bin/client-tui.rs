#[cfg(feature = "client_tui")]
use log::info;
/// Aries Agent TUI
#[cfg(feature = "client_tui")]
use mediator::{aries_agent::AgentMaker, tui};

#[cfg(feature = "client_tui")]
#[tokio::main]
async fn main() {
    fn setup_logging() {
        let env = env_logger::Env::default().default_filter_or("info");
        env_logger::init_from_env(env);
    }

    fn load_dot_env() {
        let _ = dotenvy::dotenv();
    }
    info!("TUI initializing!");
    load_dot_env();
    setup_logging();
    let agent = AgentMaker::new_demo_agent().await.unwrap();
    tui::init_tui(agent).await;
}

#[cfg(not(feature = "client_tui"))]
fn main() {
    print!("This is a placeholder binary. Please enable \"client_tui\" feature to to build the functional client_tui binary.")
}
