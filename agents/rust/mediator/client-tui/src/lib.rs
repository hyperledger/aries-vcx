use std::collections::VecDeque;

use aries_vcx_core::wallet::base_wallet::BaseWallet;
use mediation::storage::MediatorPersistence;
use mediator::aries_agent::{client::transports::AriesReqwest, ArcAgent};
use messages::msg_fields::protocols::out_of_band::invitation::Invitation as OOBInvitation;
use serde_json::{json, Value};

pub async fn handle_register(
    agent: ArcAgent<impl BaseWallet + 'static, impl MediatorPersistence>,
    oob_invite: OOBInvitation,
) -> Result<Value, String> {
    let mut aries_transport = AriesReqwest {
        response_queue: VecDeque::new(),
        client: reqwest::Client::new(),
    };
    let state = agent
        .establish_connection(oob_invite, &mut aries_transport)
        .await
        .map_err(|err| format!("{err:?}"))?;
    Ok(json!({
        "status": "success",
        "connection_completed": state
    }))
}
