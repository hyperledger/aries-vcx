// Copyright 2023 Naian G.
// SPDX-License-Identifier: Apache-2.0
// use crate::routes::coordination::handle_coord;
use crate::routes::forward::handle_forward;
use crate::routes::hello_world;
use crate::routes::json;
use crate::routes::json::respond_message_json;
use crate::routes::pickup::handle_pickup;
use crate::storage;
use axum::{routing::get, routing::post, Router};
use std::sync::Arc;

pub async fn create_router() -> Router {
    // Initialize and get a storage struct that implements MediatorPersistence trait
    // (this handles connection to storage backend)
    let storage = storage::get_persistence().await;
    Router::new()
        .route(
            "/",
            get(hello_world::handle_get).post(hello_world::handle_get),
        )
        .route(
            "/json",
            get(json::echo_message_json).post(respond_message_json),
        )
        .route("/forward", post(handle_forward))
        .route("/pickup", post(handle_pickup))
        // .route("/coord", post(handle_coord))
        .with_state(Arc::new(storage))
}
