use aries_vcx_core::wallet::base_wallet::BaseWallet;
use messages::msg_fields::protocols::coordinate_mediation::{
    CoordinateMediation, MediateGrant, MediateGrantContent, MediateGrantDecorators,
};
use uuid::Uuid;

use super::utils::prelude::*;

pub async fn handle_mediation_coord(
    agent: &ArcAgent<impl BaseWallet + 'static, impl MediatorPersistence>,
    coord_msg: CoordinateMediation,
    auth_pubkey: &str,
) -> Result<CoordinateMediation, String> {
    if let CoordinateMediation::MediateRequest(_mediate_request) = coord_msg {
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
        let mediate_grant_content = MediateGrantContent {
            endpoint: service.service_endpoint.to_string(),
            routing_keys,
        };
        let mediate_grant = MediateGrant::builder()
            .content(mediate_grant_content)
            .decorators(MediateGrantDecorators::default())
            .id(Uuid::new_v4().to_string())
            .build();
        let coord_response = CoordinateMediation::MediateGrant(mediate_grant);
        return Ok(coord_response);
    };
    let coord_response = crate::mediation::coordination::handle_coord_authenticated(
        agent.get_persistence_ref(),
        coord_msg,
        auth_pubkey,
    )
    .await;
    Ok(coord_response)
}
