use std::fmt::Debug;

use aries_vcx_core::wallet::base_wallet::BaseWallet;
use axum::{body::Bytes, extract::State, Json};
use messages::AriesMessage;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utils::prelude::*;

mod connection;
mod forward;
mod mediator_coord;
mod pickup;
mod utils;

use connection::handle_aries_connection;
use forward::handle_routing_forward;
use mediator_coord::handle_mediation_coord;
use pickup::handle_pickup_protocol;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum GeneralAriesMessage {
    AriesVCXSupported(AriesMessage),
}
pub fn unhandled_aries_message(message: impl Debug) -> String {
    format!("Don't know how to handle this message type {:#?}", message)
}

pub async fn handle_aries<T: BaseWallet + 'static, P: MediatorPersistence>(
    State(agent): State<ArcAgent<T, P>>,
    didcomm_msg: Bytes,
) -> Result<Json<Value>, String> {
    log::info!("processing message {:?}", &didcomm_msg);
    let unpacked = agent.unpack_didcomm(&didcomm_msg).await.unwrap();
    let aries_message: GeneralAriesMessage =
        serde_json::from_str(&unpacked.message).map_err(|e| e.to_string())?;
    let packed_response =
        if let GeneralAriesMessage::AriesVCXSupported(AriesMessage::Connection(conn)) =
            aries_message
        {
            handle_aries_connection(agent.clone(), conn).await?
        } else if let GeneralAriesMessage::AriesVCXSupported(AriesMessage::Routing(forward)) =
            aries_message
        {
            handle_routing_forward(agent.clone(), forward).await?;
            return Ok(Json(json!({})));
        } else {
            // Authenticated flow: Auth known VerKey then process account related messages
            let account_details = agent.auth_and_get_details(&unpacked.sender_verkey).await?;
            log::info!("Processing message for {:?}", account_details.account_name);
            let aries_response = match aries_message {
                GeneralAriesMessage::AriesVCXSupported(AriesMessage::Pickup(pickup_message)) => {
                    let pickup_response = handle_pickup_protocol(
                        &agent,
                        pickup_message,
                        &account_details.auth_pubkey,
                    )
                    .await?;
                    AriesMessage::Pickup(pickup_response)
                }
                GeneralAriesMessage::AriesVCXSupported(AriesMessage::CoordinateMediation(
                    coord_message,
                )) => {
                    let coord_response =
                        handle_mediation_coord(&agent, coord_message, &account_details.auth_pubkey)
                            .await?;
                    AriesMessage::CoordinateMediation(coord_response)
                }
                GeneralAriesMessage::AriesVCXSupported(aries_message) => {
                    Err(unhandled_aries_message(aries_message))?
                }
            };
            let aries_response_bytes =
                serde_json::to_vec(&aries_response).map_err(string_from_std_error)?;
            agent
                .pack_didcomm(
                    &aries_response_bytes,
                    &account_details.our_signing_key,
                    &account_details.their_did_doc,
                )
                .await?
        };
    let EncryptionEnvelope(packed_message_bytes) = packed_response;
    let packed_json = serde_json::from_slice(&packed_message_bytes[..]).unwrap();
    Ok(Json(packed_json))
}
