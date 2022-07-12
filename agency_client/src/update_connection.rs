use async_trait::async_trait;

use crate::{delete_connection, GeneralMessage};
use crate::error::AgencyClientResult;

pub async fn send_delete_connection_message(pw_did: &str, pw_verkey: &str, agent_did: &str, agent_vk: &str) -> AgencyClientResult<()> {
    trace!("send_delete_connection_message >>>");

    delete_connection()
        .to(pw_did)?
        .to_vk(pw_verkey)?
        .agent_did(agent_did)?
        .agent_vk(agent_vk)?
        .send_secure()
        .await
        .map_err(|err| err.extend("Cannot delete connection"))
}
