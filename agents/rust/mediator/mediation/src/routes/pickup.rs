// Copyright 2023 Naian G.
// SPDX-License-Identifier: Apache-2.0
use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use log::info;

use crate::{
    didcomm_types::{
        pickup_delivery_message_structs::*, PickupDeliveryReqMsg, PickupMsgEnum, PickupStatusMsg,
        PickupStatusReqMsg, ProblemReportMsg,
    },
    storage::MediatorPersistence,
};

pub async fn handle_pickup<T: MediatorPersistence>(
    State(storage): State<Arc<T>>,
    Json(pickup_message): Json<PickupMsgEnum>,
) -> (StatusCode, Json<PickupMsgEnum>) {
    match &pickup_message {
        PickupMsgEnum::PickupStatusReqMsg(status_request) => (
            StatusCode::OK,
            handle_pickup_status_req(status_request, storage).await,
        ),
        // Why is client sending us status? That's server's job.
        PickupMsgEnum::PickupStatusMsg(_status) => (
            StatusCode::BAD_REQUEST,
            handle_pickup_type_not_implemented().await,
        ),
        PickupMsgEnum::PickupDeliveryReq(delivery_request) => {
            handle_pickup_delivery_req(delivery_request, storage).await
        }
        _ => {
            info!("Received {:#?}", &pickup_message);
            (
                StatusCode::NOT_IMPLEMENTED,
                handle_pickup_type_not_implemented().await,
            )
        }
    }
}

async fn handle_pickup_status_req<T: MediatorPersistence>(
    status_request: &PickupStatusReqMsg,
    storage: Arc<T>,
) -> Json<PickupMsgEnum> {
    info!("Received {:#?}", &status_request);
    let auth_pubkey = &status_request.auth_pubkey;
    let message_count = storage
        .retrieve_pending_message_count(auth_pubkey, status_request.recipient_key.as_ref())
        .await
        .unwrap();
    let status = PickupStatusMsg {
        message_count,
        recipient_key: status_request.recipient_key.to_owned(),
    };
    info!("Sending {:#?}", &status);
    Json(PickupMsgEnum::PickupStatusMsg(status))
}

async fn handle_pickup_delivery_req<T: MediatorPersistence>(
    delivery_request: &PickupDeliveryReqMsg,
    storage: Arc<T>,
) -> (StatusCode, Json<PickupMsgEnum>) {
    info!("Received {:#?}", &delivery_request);
    let auth_pubkey = &delivery_request.auth_pubkey;
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
    let attach: Vec<PickupDeliveryMsgAttach> = messages
        .into_iter()
        .map(|(message_id, message_content)| PickupDeliveryMsgAttach {
            id: message_id,
            data: PickupDeliveryMsgAttachData {
                base64: message_content,
            },
        })
        .collect();
    if !attach.is_empty() {
        (
            StatusCode::OK,
            Json(PickupMsgEnum::PickupDelivery(PickupDeliveryMsg {
                recipient_key: delivery_request.recipient_key.to_owned(),
                attach,
            })),
        )
    } else {
        // send status message instead
        (
            StatusCode::OK,
            Json(PickupMsgEnum::PickupStatusMsg(PickupStatusMsg {
                message_count: 0,
                recipient_key: delivery_request.recipient_key.to_owned(),
            })),
        )
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

async fn handle_pickup_type_not_implemented() -> Json<PickupMsgEnum> {
    let problem = ProblemReportMsg {
        description: "This pickup request type not yet implemented.\n Please try again later"
            .to_owned(),
    };
    info!("Sending {:#?}", &problem);
    Json(PickupMsgEnum::ProblemReport(problem))
}
