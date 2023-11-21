// Copyright 2023 Naian G.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use log::{debug, info};
use mediation::storage::MediatorPersistence;
use messages::{
    decorators::thread::Thread,
    msg_fields::protocols::{
        notification::ack::{Ack, AckContent, AckDecorators, AckStatus},
        routing::Forward,
    },
};
use uuid::Uuid;

pub async fn handle_forward<T>(storage: Arc<T>, forward_msg: Forward) -> Ack
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
    Ack::builder()
        .content(ack_content)
        .decorators(ack_deco)
        .id(Uuid::new_v4().to_string())
        .build()
}
