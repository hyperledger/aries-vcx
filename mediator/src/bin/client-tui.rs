/// Aries Agent TUI
use mediator::{agent::Agent, tui};

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    load_dot_env();
    setup_logging();
    let agent = Agent::new_demo_agent().await.unwrap();
    tui::init_tui(agent).await;
}

fn setup_logging() {
    let env = env_logger::Env::default().default_filter_or("info");
    env_logger::init_from_env(env);
}

fn load_dot_env() {
    let _ = dotenvy::dotenv();
}
