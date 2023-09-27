/// Client-side focused api accessible Aries Agent
use log::info;
use mediator::{agent::Agent, routes::client::build_client_router};

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    load_dot_env();
    setup_logging();
    let endpoint_root = std::env::var("ENDPOINT_ROOT").unwrap_or("127.0.0.1:3003".into());
    info!("Client endpoint root address {}", endpoint_root);
    let agent = Agent::new_demo_agent().await.unwrap();
    let app_router = build_client_router(agent).await;
    axum::Server::bind(
        &endpoint_root
            .parse()
            .expect("Pass an address to listen on like IP:PORT"),
    )
    .serve(app_router.into_make_service())
    .await
    .unwrap();
}

fn setup_logging() {
    let env = env_logger::Env::default().default_filter_or("info");
    env_logger::init_from_env(env);
}

fn load_dot_env() {
    let _ = dotenvy::dotenv();
}
