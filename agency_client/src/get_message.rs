use crate::{agency_settings, GeneralMessage, get_messages, MessageStatusCode};
use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::messages::get_messages::{Message, MessageByConnection};

pub async fn get_connection_messages(pw_did: &str, pw_vk: &str, agent_did: &str, agent_vk: &str, msg_uid: Option<Vec<String>>, status_codes: Option<Vec<MessageStatusCode>>) -> AgencyClientResult<Vec<Message>> {
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

    trace!("message returned: {:?}", response);
    Ok(response)
}

pub fn parse_status_codes(status_codes: Option<Vec<String>>) -> AgencyClientResult<Option<Vec<MessageStatusCode>>> {
    match status_codes {
        Some(codes) => {
            let codes = codes
                .iter()
                .map(|code|
                    ::serde_json::from_str::<MessageStatusCode>(&format!("\"{}\"", code))
                        .map_err(|err| AgencyClientError::from_msg(AgencyClientErrorKind::InvalidJson, format!("Cannot parse message status code: {}", err)))
                ).collect::<AgencyClientResult<Vec<MessageStatusCode>>>()?;
            Ok(Some(codes))
        }
        None => Ok(None)
    }
}

pub fn parse_connection_handles(conn_handles: Vec<String>) -> AgencyClientResult<Vec<u32>> {
    trace!("parse_connection_handles >>> conn_handles: {:?}", conn_handles);
    let codes = conn_handles
        .iter()
        .map(|handle|
            ::serde_json::from_str::<u32>(handle)
                .map_err(|err| AgencyClientError::from_msg(AgencyClientErrorKind::InvalidJson, format!("Cannot parse connection handles: {}", err)))
        ).collect::<AgencyClientResult<Vec<u32>>>()?;
    Ok(codes)
}