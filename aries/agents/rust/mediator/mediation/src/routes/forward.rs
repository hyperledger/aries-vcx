// Copyright 2023 Naian G.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use axum::{extract::State, Json};
use log::{debug, info};
use messages::{
    decorators::thread::Thread,
    msg_fields::protocols::{
        notification::ack::{Ack, AckContent, AckDecorators, AckStatus},
        routing::Forward,
    },
};
use uuid::Uuid;

use crate::storage::MediatorPersistence;

pub async fn handle_forward<T>(
    State(storage): State<Arc<T>>,
    Json(forward_msg): Json<Forward>,
) -> Json<Ack>
where
    T: MediatorPersistence,
{
    info!("Persisting forward message");
    debug!("{forward_msg:#?}");
    let _ack_status = match storage
        .persist_forward_message(
            &forward_msg.content.to,
            &serde_json::to_string(&forward_msg.content.msg).unwrap(),
        )
        .await
    {
        Ok(_) => {
            info!("Persisted forward");
            AckStatus::Ok
        }
        Err(e) => {
            info!("Error when persisting forward: {}", e);
            AckStatus::Pending
        }
    };
    let ack_content = AckContent::builder().status(AckStatus::Ok).build();
    let ack_deco = AckDecorators::builder()
        .thread(Thread::builder().thid(forward_msg.id).build())
        .build();
    let ack = Ack::builder()
        .content(ack_content)
        .decorators(ack_deco)
        .id(Uuid::new_v4().to_string())
        .build();
    Json(ack)
}
