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
            connection::handle_aries_connection(agent.clone(), conn).await?
        // } else if GeneralAriesMessage::XumCoord(MediatorCoordMsgEnum::MediateRequest) {
        //     // Need diddoc of initiator.
        //     // We would have to save/persist it somewhere temporarily between dicomm cconnection
        //     // establishment, and mediation request. In memory, or in database in
        //     // separate ephemeral connections table.
        //     let service_endpoint = agent.get_service_ref().unwrap().service_endpoint;
        //     let routing_keys = agent.get_service_ref().unwrap().routing_keys;

        //     mediation::routes::coordination::handle_mediate_request(
        //         agent.get_persistence_ref(),
        //         &unpacked
        //             .sender_verkey
        //             .ok_or("Can't register anon aries peer")?,
        //         did_doc,
        //         &unpacked.recipient_verkey,
        //         mediation::didcomm_types::mediator_coord_structs::MediateGrantData{
        // endpoint: service_endpoint.into(), routing_keys},     );
        //     todo!()
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
                        mediator_coord::handle_mediation_coord(&agent, coord_message, &auth_pubkey)
                            .await?;
                    let aries_response =
                        serde_json::to_vec(&coord_response).map_err(string_from_std_error)?;
                    agent
                        .pack_didcomm(&aries_response, &our_signing_key, &their_diddoc)
                        .await?
                }
                GeneralAriesMessage::XumPickup(pickup_message) => {
                    pickup::handle_pickup_protocol(agent, pickup_message).await?;
                    todo!();
                }
            }
        };
    let EncryptionEnvelope(packed_message_bytes) = packed_response;
    let packed_json = serde_json::from_slice(&packed_message_bytes[..]).unwrap();
    Ok(Json(packed_json))
}
