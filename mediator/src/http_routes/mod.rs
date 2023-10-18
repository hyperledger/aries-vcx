use std::{fmt::Debug, sync::Arc};

use aries_vcx::utils::encryption_envelope::EncryptionEnvelope;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use axum::{
    body::Bytes,
    extract::State,
    http::header::{HeaderMap, ACCEPT},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use log::info;
use messages::{msg_fields::protocols::connection::Connection, AriesMessage};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use xum_test_server::{
    didcomm_types::{
        mediator_coord_structs::{MediateGrantData, MediatorCoordMsgEnum},
        PickupMsgEnum,
    },
    storage::MediatorPersistence,
};

use crate::{aries_agent::Agent, utils::string_from_std_error};
type ArcAgent<T, P> = Arc<Agent<T, P>>;

pub mod client;
pub mod tui;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum GeneralAriesMessage {
    AriesVCXSupported(AriesMessage),
    XumPickup(xum_test_server::didcomm_types::PickupMsgEnum),
    XumCoord(xum_test_server::didcomm_types::mediator_coord_structs::MediatorCoordMsgEnum),
}
pub fn unhandled_aries(message: impl Debug) -> String {
    format!("Don't know how to handle this message type {:#?}", message)
}
pub async fn handle_aries_connection<T: BaseWallet + 'static, P: MediatorPersistence>(
    agent: ArcAgent<T, P>,
    connection: Connection,
) -> Result<EncryptionEnvelope, String> {
    match connection {
        Connection::Invitation(_invite) => {
            Err("Mediator does not handle random invites. Sorry.".to_owned())
        }
        Connection::Request(register_request) => {
            agent.handle_connection_req(register_request).await
        }
        _ => Err(unhandled_aries(connection)),
    }
}
pub async fn handle_mediation_coord(
    agent: &ArcAgent<impl BaseWallet + 'static, impl MediatorPersistence>,
    coord_msg: MediatorCoordMsgEnum,
    auth_pubkey: &str,
) -> Result<MediatorCoordMsgEnum, String> {
    if let MediatorCoordMsgEnum::MediateRequest = coord_msg {
        let service = agent
            .get_service_ref()
            .ok_or("Mediation agent must have service defined.")?;
        let mut routing_keys = Vec::new();
        routing_keys.extend_from_slice(&service.routing_keys);
        routing_keys.push(
            service
                .recipient_keys
                .first()
                .expect("Service must have recipient key")
                .to_owned(),
        );
        let coord_response = MediatorCoordMsgEnum::MediateGrant(MediateGrantData {
            endpoint: service.service_endpoint.to_string(),
            routing_keys,
        });
        return Ok(coord_response);
    };
    let Json(coord_response) = xum_test_server::routes::coordination::handle_coord_authenticated(
        State(agent.get_persistence_ref()),
        Json(coord_msg),
        auth_pubkey,
    )
    .await;
    Ok(coord_response)
    // match coord_msg {
    //     MediatorCoordMsgEnum::MediateRequest(request_data) => {
    //         let Json(response) =
    // xum_test_server::routes::coordination::handle_coord(State(agent.get_persistence_ref()),
    // Json(coord_msg)).await;         todo!()
    //     },
    //     _ => Err(unhandled_aries(coord_msg)),
    // }
}

pub async fn handle_pickup_protocol(
    _agent: ArcAgent<impl BaseWallet + 'static, impl MediatorPersistence>,
    _pickup_msg: PickupMsgEnum,
) -> Result<EncryptionEnvelope, String> {
    todo!()
}

pub async fn handle_aries<T: BaseWallet + 'static, P: MediatorPersistence>(
    State(agent): State<ArcAgent<T, P>>,
    didcomm_msg: Bytes,
) -> Result<Json<Value>, String> {
    info!("processing message {:?}", &didcomm_msg);
    let unpacked = agent.unpack_didcomm(&didcomm_msg).await.unwrap();
    let aries_message: GeneralAriesMessage =
        serde_json::from_str(&unpacked.message).expect("Decoding unpacked message as AriesMessage");
    let packed_response =
        if let GeneralAriesMessage::AriesVCXSupported(AriesMessage::Connection(conn)) =
            aries_message
        {
            handle_aries_connection(agent.clone(), conn).await?
        // } else if GeneralAriesMessage::XumCoord(MediatorCoordMsgEnum::MediateRequest) {
        //     // Need diddoc of initiator.
        //     // We would have to save/persist it somewhere temporarily between dicomm cconnection
        //     // establishment, and mediation request. In memory, or in database in
        //     // separate ephemeral connections table.
        //     let service_endpoint = agent.get_service_ref().unwrap().service_endpoint;
        //     let routing_keys = agent.get_service_ref().unwrap().routing_keys;

        //     xum_test_server::routes::coordination::handle_mediate_request(
        //         agent.get_persistence_ref(),
        //         &unpacked
        //             .sender_verkey
        //             .ok_or("Can't register anon aries peer")?,
        //         did_doc,
        //         &unpacked.recipient_verkey,
        //         xum_test_server::didcomm_types::mediator_coord_structs::MediateGrantData{
        // endpoint: service_endpoint.into(), routing_keys},     );
        //     todo!()
        } else {
            let (account_name, our_signing_key, their_diddoc) =
                agent.auth_and_get_details(&unpacked.sender_verkey).await?;
            let auth_pubkey = unpacked
                .sender_verkey
                .expect("Sender key authenticated above, so it must be present..");
            info!("Processing message for {:?}", account_name);
            match aries_message {
                GeneralAriesMessage::AriesVCXSupported(aries_message) => {
                    Err(unhandled_aries(aries_message))?
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
pub async fn oob_invite_qr(
    headers: HeaderMap,
    State(agent): State<ArcAgent<impl BaseWallet + 'static, impl MediatorPersistence>>,
) -> Response {
    let Json(oob_json) = oob_invite_json(State(agent)).await;
    let preferred_mimetype = headers
        .get(ACCEPT)
        .map(|s| s.to_str().unwrap_or_default())
        .unwrap_or_default();
    match preferred_mimetype {
        "application/json" => Json(oob_json).into_response(),
        _ => {
            let oob_string = serde_json::to_string_pretty(&oob_json).unwrap();
            let qr = fast_qr::QRBuilder::new(oob_string.clone()).build().unwrap();
            let oob_qr_svg = fast_qr::convert::svg::SvgBuilder::default().to_str(&qr);
            Html(format!(
                "<style>
                        svg {{
                            width: 50%;
                            height: 50%;
                        }}
                    </style>
                    {oob_qr_svg} <br>
                    <pre>{oob_string}</pre>"
            ))
            .into_response()
        }
    }
}

pub async fn oob_invite_json(
    State(agent): State<ArcAgent<impl BaseWallet + 'static, impl MediatorPersistence>>,
) -> Json<Value> {
    let oob = agent.get_oob_invite().unwrap();
    Json(serde_json::to_value(oob).unwrap())
}

pub async fn readme() -> Html<String> {
    Html("<p>Please refer to the API section of <a>readme</a> for usage. Thanks. </p>".into())
}

pub async fn build_router<T: BaseWallet + 'static, P: MediatorPersistence>(
    agent: Agent<T, P>,
) -> Router {
    Router::default()
        .route("/", get(readme))
        .route("/register", get(oob_invite_qr))
        .route("/register.json", get(oob_invite_json))
        .route("/aries", get(handle_aries).post(handle_aries))
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        .with_state(Arc::new(agent))
}
