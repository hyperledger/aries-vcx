// Copyright 2023 Naian G.
// SPDX-License-Identifier: Apache-2.0

use log::info;
use mediation::logging;
use mediation::server;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    logging::init_logger();
    info!("Hello, world!");
    server::run_server().await;
}
