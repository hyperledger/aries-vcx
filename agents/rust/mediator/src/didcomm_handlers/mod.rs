use std::fmt::Debug;

use axum::{body::Bytes, extract::State, Json};
use messages::AriesMessage;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utils::prelude::*;

mod connection;
mod mediator_coord;
mod pickup;
mod utils;

use connection::handle_aries_connection;
use mediator_coord::handle_mediation_coord;
use pickup::handle_pickup_protocol;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum GeneralAriesMessage {
    AriesVCXSupported(AriesMessage),
    XumPickup(mediation::didcomm_types::PickupMsgEnum),
    XumCoord(mediation::didcomm_types::mediator_coord_structs::MediatorCoordMsgEnum),
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
        serde_json::from_str(&unpacked.message).expect("Decoding unpacked message as AriesMessage");
    let packed_response =
        if let GeneralAriesMessage::AriesVCXSupported(AriesMessage::Connection(conn)) =
            aries_message
        {
            handle_aries_connection(agent.clone(), conn).await?
        } else {
            let (account_name, our_signing_key, their_diddoc) =
                agent.auth_and_get_details(&unpacked.sender_verkey).await?;
            let auth_pubkey = unpacked
                .sender_verkey
                .expect("Sender key authenticated above, so it must be present..");
            log::info!("Processing message for {:?}", account_name);
            match aries_message {
                GeneralAriesMessage::AriesVCXSupported(aries_message) => {
                    Err(unhandled_aries_message(aries_message))?
                }
                GeneralAriesMessage::XumCoord(coord_message) => {
                    let coord_response =
                        handle_mediation_coord(&agent, coord_message, &auth_pubkey).await?;
                    let aries_response =
                        serde_json::to_vec(&coord_response).map_err(string_from_std_error)?;
                    agent
                        .pack_didcomm(&aries_response, &our_signing_key, &their_diddoc)
                        .await?
                }
                GeneralAriesMessage::XumPickup(pickup_message) => {
                    handle_pickup_protocol(agent, pickup_message).await?;
                    todo!();
                }
            }
        };
    let EncryptionEnvelope(packed_message_bytes) = packed_response;
    let packed_json = serde_json::from_slice(&packed_message_bytes[..]).unwrap();
    Ok(Json(packed_json))
}
