#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let app_router = mediator::routes::build_router().await;
    axum::Server::bind(&"0.0.0.0:8005".parse().unwrap())
        .serve(app_router.into_make_service())
        .await
        .unwrap();
}
