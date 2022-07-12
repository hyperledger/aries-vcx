use crate::{agency_settings, GeneralMessage, get_messages, MessageStatusCode};
use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::messages::get_messages::{AgencyMessage, AgencyMessageEncrypted};

pub async fn get_encrypted_connection_messages(pw_did: &str, pw_vk: &str, agent_did: &str, agent_vk: &str, msg_uid: Option<Vec<String>>, status_codes: Option<Vec<MessageStatusCode>>) -> AgencyClientResult<Vec<AgencyMessageEncrypted>> {
    trace!("get_connection_messages >>> pw_did: {}, pw_vk: {}, agent_vk: {}, msg_uid: {:?}",
           pw_did, pw_vk, agent_vk, msg_uid);

    let response = get_messages()
        .to(&pw_did)?
        .to_vk(&pw_vk)?
        .agent_did(&agent_did)?
        .agent_vk(&agent_vk)?
        .uid(msg_uid)?
        .status_codes(status_codes)?
        .send_secure()
        .await
        .map_err(|err| err.map(AgencyClientErrorKind::PostMessageFailed, "Cannot get messages"))?;

    Ok(response)
}