// Copyright 2023 Naian G.
// SPDX-License-Identifier: Apache-2.0
use std::sync::Arc;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use log::info;
use messages::{
    decorators::attachment::{Attachment, AttachmentData, AttachmentType},
    msg_fields::protocols::pickup::{
        Delivery, DeliveryContent, DeliveryRequestContent, Pickup, Status, StatusContent,
        StatusDecorators, StatusRequestContent,
    },
};
use uuid::Uuid;

use crate::persistence::MediatorPersistence;

pub async fn handle_pickup_authenticated<T: MediatorPersistence>(
    storage: Arc<T>,
    pickup_message: Pickup,
    auth_pubkey: &str,
) -> Pickup {
    match &pickup_message {
        Pickup::StatusRequest(status_request) => {
            handle_pickup_status_req(&status_request.content, storage, auth_pubkey).await
        }
        // Why is client sending us status? That's server's job.
        Pickup::Status(_status) =>
        // StatusCode::BAD_REQUEST,
        {
            handle_pickup_default_status(storage, auth_pubkey).await
        }

        Pickup::DeliveryRequest(delivery_request) => {
            handle_pickup_delivery_req(&delivery_request.content, storage, auth_pubkey).await
        }
        _ => {
            info!("Received {:#?}", &pickup_message);
            // StatusCode::NOT_IMPLEMENTED,
            handle_pickup_default_status(storage, auth_pubkey).await
        }
    }
}

async fn handle_pickup_status_req<T: MediatorPersistence>(
    status_request: &StatusRequestContent,
    storage: Arc<T>,
    auth_pubkey: &str,
) -> Pickup {
    info!("Received {:#?}", &status_request);
    let message_count = storage
        .retrieve_pending_message_count(auth_pubkey, status_request.recipient_key.as_ref())
        .await
        .unwrap();
    let status_content = if let Some(recipient_key) = status_request.recipient_key.clone() {
        StatusContent::builder()
            .message_count(message_count)
            .recipient_key(recipient_key)
            .build()
    } else {
        StatusContent::builder()
            .message_count(message_count)
            .build()
    };
    let status = Status::builder()
        .content(status_content)
        .decorators(StatusDecorators::default())
        .id(Uuid::new_v4().to_string())
        .build();

    info!("Sending {:#?}", &status);
    Pickup::Status(status)
}

async fn handle_pickup_delivery_req<T: MediatorPersistence>(
    delivery_request: &DeliveryRequestContent,
    storage: Arc<T>,
    auth_pubkey: &str,
) -> Pickup {
    info!("Received {:#?}", &delivery_request);
    let messages = storage
        .retrieve_pending_messages(
            auth_pubkey,
            delivery_request.limit,
            delivery_request.recipient_key.as_ref(),
        )
        .await
        .unwrap();
    // for (message_id, message_content) in messages.into_iter() {
    //     info!("Message {:#?} {:#?}", message_id, String::from_utf8(message_content).unwrap())
    // }
    let attach: Vec<Attachment> = messages
        .into_iter()
        .map(|(message_id, message_content)| {
            Attachment::builder()
                .id(message_id)
                .data(
                    AttachmentData::builder()
                        .content(AttachmentType::Base64(
                            URL_SAFE_NO_PAD.encode(&message_content),
                        ))
                        .build(),
                )
                .build()
        })
        .collect();
    if !attach.is_empty() {
        Pickup::Delivery(
            Delivery::builder()
                .content(DeliveryContent {
                    recipient_key: delivery_request.recipient_key.to_owned(),
                    attach,
                })
                .id(Uuid::new_v4().to_string())
                .build(),
        )
    } else {
        // send default status message instead
        handle_pickup_default_status(storage, auth_pubkey).await
    }
}
// Returns global status message for user (not restricted to recipient key)
// async fn handle_pickup_default<T: MediatorPersistence>(
//     storage: Arc<T>,
// ) -> Json<PickupMsgEnum> {

//     let message_count = storage
//         .retrieve_pending_message_count(None)
//         .await;
//     let status = PickupStatusMsg {
//         message_count,
//         recipient_key: None
//     };
//     info!("Sending {:#?}", &status);
//     Json(PickupMsgEnum::PickupStatusMsg(status))
// }

/// Return status by default
async fn handle_pickup_default_status(
    storage: Arc<impl MediatorPersistence>,
    auth_pubkey: &str,
) -> Pickup {
    info!("Default behavior: responding with status");
    let status_request = StatusRequestContent::builder().build();
    handle_pickup_status_req(&status_request, storage, auth_pubkey).await
}
