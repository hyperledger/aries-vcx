use log::info;
use mediator::aries_agent::AgentMaker;

#[tokio::main]
async fn main() {
    info!("Starting up mediator! ⚙️⚙️");
    load_dot_env();
    setup_logging();
    let endpoint_root = std::env::var("ENDPOINT_ROOT").unwrap_or("127.0.0.1:8005".into());
    info!("Mediator endpoint root address {}", endpoint_root);
    let mut agent = AgentMaker::new_demo_agent().await.unwrap();
    agent
        .init_service(
            vec![],
            format!("http://{endpoint_root}/didcomm").parse().unwrap(),
        )
        .await
        .unwrap();
    let app_router = mediator::http_routes::build_router(agent).await;
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
