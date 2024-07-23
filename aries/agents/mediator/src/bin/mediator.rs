use aries_vcx_wallet::wallet::indy::{indy_wallet_config::IndyWalletConfig, IndySdkWallet};
use log::info;
use mediator::aries_agent::AgentBuilder;

#[tokio::main]
async fn main() {
    load_dot_env();
    setup_logging();
    info!("Starting up mediator! ⚙️⚙️");
    let endpoint_root = std::env::var("ENDPOINT_ROOT").unwrap_or("127.0.0.1:8005".into());
    info!("Mediator endpoint root address: {}", endpoint_root);
    let indy_wallet_config_json = std::env::var("INDY_WALLET_CONFIG").unwrap_or(
        "{
            \"wallet_name\": \"demo-wallet\",
            \"wallet_key\" : \"8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY\",
            \"wallet_key_derivation\": \"RAW\"
        }"
        .to_string(),
    );
    let wallet_config: IndyWalletConfig = serde_json::from_str(&indy_wallet_config_json).unwrap();
    info!("Wallet Config: {:?}", wallet_config);
    let mut agent = AgentBuilder::<IndySdkWallet>::new_from_wallet_config(wallet_config)
        .await
        .unwrap();
    agent
        .init_service(
            vec![],
            format!("http://{endpoint_root}/didcomm").parse().unwrap(),
        )
        .await
        .unwrap();
    let app_router = mediator::http_routes::build_router(agent).await;
    info!("Starting server");
    let listener = tokio::net::TcpListener::bind(&endpoint_root).await.unwrap();
    axum::serve(listener, app_router.into_make_service())
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
