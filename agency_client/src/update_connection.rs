use async_trait::async_trait;

use crate::{AgencyClientError, AgencyClientErrorKind, Client2AgencyMessage, DeleteConnectionBuilder, parse_response_from_agency, prepare_message_for_agent};
use crate::error::AgencyClientResult;
use crate::utils::comm::post_to_agency;

pub async fn send_delete_connection_message(_pw_did: &str, to_pw_vk: &str, agent_did: &str, agent_vk: &str) -> AgencyClientResult<()> {
    trace!("send_delete_connection_message >>>");
    let message = DeleteConnectionBuilder::create()
        .build();

    let data = prepare_message_for_agent(vec![Client2AgencyMessage::UpdateConnection(message)], to_pw_vk, agent_did, agent_vk).await?;
    let response = post_to_agency(&data).await?;
    let mut response = parse_response_from_agency(&response).await?;

    match response.remove(0) {
        Client2AgencyMessage::UpdateConnectionResponse(_) => Ok(()),
        _ => Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, "Message does not match any variant of UpdateConnectionResponse"))
    }
}
