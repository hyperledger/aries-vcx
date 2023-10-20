/// Client-side focused api accessible Aries Agent
#[cfg(feature = "client_http_api")]
#[tokio::main]
async fn main() {
    use mediator::{
        aries_agent::AgentBuilder,
        http_routes::client::build_client_router,
        utils::binary_utils::{load_dot_env, setup_logging},
    };

    load_dot_env();
    setup_logging();
    log::info!("Putting up local web interface controlling client");
    let endpoint_root = std::env::var("ENDPOINT_ROOT").unwrap_or("127.0.0.1:3003".into());
    log::info!("Client web endpoint root address {}", endpoint_root);
    let agent = AgentBuilder::new_demo_agent().await.unwrap();
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

#[cfg(not(feature = "client_http_api"))]
fn main() {
    print!(
        "This is a placeholder binary. Please enable \"client_tui\" feature to to build the \
         functional client_tui binary."
    )
}
