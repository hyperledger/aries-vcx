// Copyright 2023 Naian G.
// SPDX-License-Identifier: Apache-2.0

use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageJson {
    message: String,
}

#[derive(Serialize)]
pub struct MessageResponseJson {
    message: String,
    response: String,
}

pub async fn echo_message_json(Json(body): Json<MessageJson>) -> Json<MessageJson> {
    Json(body)
}

pub async fn respond_message_json(Json(body): Json<MessageJson>) -> Json<MessageResponseJson> {
    Json(MessageResponseJson {
        message: (body.message),
        response: ("I am groot".to_owned()),
    })
}
