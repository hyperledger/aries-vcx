use aries_vcx_core::wallet::base_wallet::BaseWallet;
use mediator::{aries_agent::ArcAgent, persistence::MediatorPersistence};
use messages::msg_fields::protocols::out_of_band::invitation::Invitation as OOBInvitation;
use serde_json::{json, Value};

pub async fn handle_register(
    agent: ArcAgent<impl BaseWallet + 'static, impl MediatorPersistence>,
    oob_invite: OOBInvitation,
) -> Result<Value, String> {
    let mut aries_transport = reqwest::Client::new();
    let state = agent
        .establish_connection(oob_invite, &mut aries_transport)
        .await
        .map_err(|err| format!("{err:?}"))?;
    Ok(json!({
        "status": "success",
        "connection_completed": state
    }))
}
