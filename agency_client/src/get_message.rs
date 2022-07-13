use crate::{agency_settings, Client2AgencyMessage, GetMessagesBuilder, MessageStatusCode, parse_response_from_agency, prepare_message_for_agent};
use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::messages::get_messages::{DownloadedMessage, DownloadedMessageEncrypted};
use crate::utils::comm::post_to_agency;

pub async fn get_encrypted_connection_messages(_pw_did: &str, to_pw_vk: &str, agent_did: &str, agent_vk: &str, msg_uid: Option<Vec<String>>, status_codes: Option<Vec<MessageStatusCode>>) -> AgencyClientResult<Vec<DownloadedMessageEncrypted>> {
    trace!("get_connection_messages >>> pw_vk: {}, agent_vk: {}, msg_uid: {:?}",
           to_pw_vk, agent_vk, msg_uid);

    let message = Client2AgencyMessage::GetMessages(
        GetMessagesBuilder::create()
            .uid(msg_uid)?
            .status_codes(status_codes)?
            .build()
    );

    let data = prepare_message_for_agent(vec![message], &to_pw_vk, &agent_did, &agent_vk).await?;
    let response = post_to_agency(&data).await?;
    let mut response = parse_response_from_agency(&response).await?;

    match response.remove(0) {
        Client2AgencyMessage::GetMessagesResponse(res) => {
            trace!("Interpreting response as V2");
            Ok(res.msgs)
        }
        _ => Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, "Message does not match any variant of GetMessagesResponse"))
    }
}