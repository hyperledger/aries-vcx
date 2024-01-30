use aries_vcx_core::wallet::base_wallet::BaseWallet;
use messages::msg_fields::protocols::{notification::ack::Ack, routing::Forward};

use super::{utils::prelude::*, ArcAgent};
use crate::mediation::forward::handle_forward;

pub async fn handle_routing_forward(
    agent: ArcAgent<impl MediatorPersistence>,
    forward: Forward,
) -> Result<Ack, String> {
    info!("{:?}", forward);
    let ack = handle_forward(agent.get_persistence_ref(), forward).await;

    Ok(ack)
}
