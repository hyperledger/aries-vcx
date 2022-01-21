use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::{A2AMessage, A2AMessageKinds, A2AMessageV2, delete_connection, GeneralMessage, parse_response_from_agency, prepare_message_for_agent};
use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::message_type::MessageTypes;
use crate::utils::comm::post_to_agency;

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConnection {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "statusCode")]
    status_code: ConnectionStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConnectionStatus {
    AlreadyConnected,
    NotConnected,
    Deleted,
}

impl Serialize for ConnectionStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let value = match self {
            ConnectionStatus::AlreadyConnected => "CS-101",
            ConnectionStatus::NotConnected => "CS-102",
            ConnectionStatus::Deleted => "CS-103",
        };
        serde_json::Value::String(value.to_string()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ConnectionStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;
        match value.as_str() {
            Some("CS-101") => Ok(ConnectionStatus::AlreadyConnected),
            Some("CS-102") => Ok(ConnectionStatus::NotConnected),
            Some("CS-103") => Ok(ConnectionStatus::Deleted),
            _ => Err(de::Error::custom("Unexpected message type."))
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct UpdateConnectionResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "statusCode")]
    status_code: ConnectionStatus,
}

#[derive(Debug)]
pub struct DeleteConnectionBuilder {
    to_did: String,
    to_vk: String,
    status_code: ConnectionStatus,
    agent_did: String,
    agent_vk: String,
}

impl DeleteConnectionBuilder {
    pub fn create() -> DeleteConnectionBuilder {
        trace!("DeleteConnection::create_message >>>");

        DeleteConnectionBuilder {
            to_did: String::new(),
            to_vk: String::new(),
            status_code: ConnectionStatus::Deleted,
            agent_did: String::new(),
            agent_vk: String::new(),
        }
    }

    pub async fn send_secure(&mut self) -> AgencyClientResult<()> {
        trace!("DeleteConnection::send >>>");

        let data = self.prepare_request()?;

        let response = post_to_agency(&data).await?;

        self.parse_response(&response)
    }

    fn parse_response(&self, response: &Vec<u8>) -> AgencyClientResult<()> {
        trace!("parse_response >>>");

        let mut response = parse_response_from_agency(response)?;

        match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::UpdateConnectionResponse(_)) => Ok(()),
            _ => Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, "Message does not match any variant of UpdateConnectionResponse"))
        }
    }
}

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

//TODO Every GeneralMessage extension, duplicates code
impl GeneralMessage for DeleteConnectionBuilder {
    type Msg = DeleteConnectionBuilder;

    fn set_to_vk(&mut self, to_vk: String) { self.to_vk = to_vk; }

    fn set_to_did(&mut self, to_did: String) { self.to_did = to_did; }

    fn set_agent_did(&mut self, did: String) {
        self.agent_did = did;
    }
    fn set_agent_vk(&mut self, vk: String) {
        self.agent_vk = vk;
    }

    fn prepare_request(&mut self) -> AgencyClientResult<Vec<u8>> {
        let message = A2AMessage::Version2(
            A2AMessageV2::UpdateConnection(
                UpdateConnection {
                    msg_type: MessageTypes::build(A2AMessageKinds::UpdateConnectionStatus),
                    status_code: self.status_code.clone(),
                }
            )
        );

        prepare_message_for_agent(vec![message], &self.to_vk, &self.agent_did, &self.agent_vk)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::utils::test_constants::DELETE_CONNECTION_DECRYPTED_RESPONSE;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_deserialize_delete_connection_payload() {
        let delete_connection_payload: UpdateConnectionResponse = serde_json::from_str(DELETE_CONNECTION_DECRYPTED_RESPONSE).unwrap();
        assert_eq!(delete_connection_payload.status_code, ConnectionStatus::Deleted);
    }
}
