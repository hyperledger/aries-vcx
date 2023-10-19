use axum::{extract::State, Json};
use mediation::didcomm_types::mediator_coord_structs::{MediateGrantData, MediatorCoordMsgEnum};

use super::utils::prelude::*;

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
    let Json(coord_response) = mediation::routes::coordination::handle_coord_authenticated(
        State(agent.get_persistence_ref()),
        Json(coord_msg),
        auth_pubkey,
    )
    .await;
    Ok(coord_response)
    // match coord_msg {
    //     MediatorCoordMsgEnum::MediateRequest(request_data) => {
    //         let Json(response) =
    // mediation::routes::coordination::handle_coord(State(agent.get_persistence_ref()),
    // Json(coord_msg)).await;         todo!()
    //     },
    //     _ => Err(unhandled_aries(coord_msg)),
    // }
}
