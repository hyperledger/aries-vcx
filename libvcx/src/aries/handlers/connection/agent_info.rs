use std::collections::HashMap;

use aries::messages::a2a::A2AMessage;
use aries::messages::connection::did_doc::DidDoc;
use aries::utils::encryption_envelope::EncryptionEnvelope;
use connection::create_agent_keys;
use error::prelude::*;
use messages::get_message::{get_connection_messages, Message};
use messages::MessageStatusCode;
use messages::update_connection::send_delete_connection_message;
use messages::update_message::{UIDsByConn, update_messages as update_messages_status};
use settings;
use settings::ProtocolTypes;
use utils::httpclient;
use utils::libindy::signus::create_and_store_my_did;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub pw_did: String,
    pub pw_vk: String,
    pub agent_did: String,
    pub agent_vk: String,
}

impl Default for AgentInfo {
    fn default() -> AgentInfo {
        AgentInfo {
            pw_did: String::new(),
            pw_vk: String::new(),
            agent_did: String::new(),
            agent_vk: String::new(),
        }
    }
}

fn _log_messages_optionally(a2a_messages: &HashMap<String, A2AMessage>) {
    #[cfg(feature = "warnlog_fetched_messages")]
        {
            for message in a2a_messages.values() {
                let serialized_msg = serde_json::to_string_pretty(message).unwrap_or_else(|_err| String::from("Failed to serialize A2AMessage."));
                warn!("Fetched decrypted connection messages:\n{}", serialized_msg);
            }
        }
}

impl AgentInfo {
    /**
    Create connection agent in one's agency
     */
    pub fn create_agent(&self) -> VcxResult<AgentInfo> {
        trace!("Agent::create_agent >>>");

        let method_name = settings::get_config_value(settings::CONFIG_DID_METHOD).ok();
        let (pw_did, pw_vk) = create_and_store_my_did(None, method_name.as_ref().map(String::as_str))?;

        /*
            Create User Pairwise Agent in old way.
            Send Messages corresponding to V2 Protocol to avoid code changes on Agency side.
        */
        let (agent_did, agent_vk) = create_agent_keys("", &pw_did, &pw_vk)?;

        Ok(AgentInfo { pw_did, pw_vk, agent_did, agent_vk })
    }

    /**
    Builds one's agency's URL endpoint
     */
    pub fn agency_endpoint(&self) -> VcxResult<String> {
        settings::get_config_value(settings::CONFIG_AGENCY_ENDPOINT)
            .map(|str| format!("{}/agency/msg", str))
    }

    pub fn routing_keys(&self) -> VcxResult<Vec<String>> {
        let agency_vk = settings::get_config_value(settings::CONFIG_AGENCY_VERKEY)?;
        Ok(vec![self.agent_vk.to_string(), agency_vk])
    }

    pub fn recipient_keys(&self) -> Vec<String> {
        vec![self.pw_vk.to_string()]
    }

    pub fn update_message_status(&self, uid: String) -> VcxResult<()> {
        trace!("Agent::update_message_status >>> uid: {:?}", uid);

        let messages_to_update = vec![UIDsByConn {
            pairwise_did: self.pw_did.clone(),
            uids: vec![uid],
        }];

        update_messages_status(MessageStatusCode::Reviewed, messages_to_update)
    }

    fn _download_encrypted_messages(&self, msg_uid: Option<Vec<String>>, status_codes: Option<Vec<MessageStatusCode>>) -> VcxResult<Vec<Message>> {
        get_connection_messages(&self.pw_did, &self.pw_vk, &self.agent_did, &self.agent_vk,
                                msg_uid, status_codes, &Some(ProtocolTypes::V2),
        )
    }

    pub fn get_messages(&self, expect_sender_vk: &str) -> VcxResult<HashMap<String, A2AMessage>> {
        trace!("Agent::get_messages >>> expect_sender_vk={}", expect_sender_vk);
        let messages = self._download_encrypted_messages(None, Some(vec![MessageStatusCode::Received]))?;
        debug!("Agent::get_messages >>> obtained {} messages", messages.len());
        let a2a_messages = self.decrypt_decode_messages(&messages, expect_sender_vk)?;
        _log_messages_optionally(&a2a_messages);
        Ok(a2a_messages)
    }

    pub fn get_messages_noauth(&self) -> VcxResult<HashMap<String, A2AMessage>> {
        trace!("Agent::get_messages_noauth >>>");
        let messages = self._download_encrypted_messages(None, Some(vec![MessageStatusCode::Received]))?;
        debug!("Agent::get_messages_noauth >>> obtained {} messages", messages.len());
        let a2a_messages = self.decrypt_decode_messages_noauth(&messages)?;
        _log_messages_optionally(&a2a_messages);
        Ok(a2a_messages)
    }

    pub fn get_message_by_id(&self, msg_id: &str, expected_sender_vk: &str) -> VcxResult<A2AMessage> {
        trace!("Agent::get_message_by_id >>> msg_id: {:?}", msg_id);
        let mut messages = self._download_encrypted_messages(Some(vec![msg_id.to_string()]), None)?;
        let message = messages
            .pop()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidMessages, format!("Message not found for id: {:?}", msg_id)))?;
        let message = self.decode_message(&message, &expected_sender_vk)?;
        Ok(message)
    }

    fn decrypt_decode_messages(&self, messages: &Vec<Message>, expected_sender_vk: &str) -> VcxResult<HashMap<String, A2AMessage>> {
        let mut a2a_messages: HashMap<String, A2AMessage> = HashMap::new();
        for message in messages {
            a2a_messages.insert(message.uid.clone(), self.decode_message(&message, expected_sender_vk)?);
        }
        return Ok(a2a_messages)
    }

    fn decrypt_decode_messages_noauth(&self, messages: &Vec<Message>) -> VcxResult<HashMap<String, A2AMessage>> {
        let mut a2a_messages: HashMap<String, A2AMessage> = HashMap::new();
        for message in messages {
            a2a_messages.insert(message.uid.clone(), self.decode_message_noauth(&message)?);
        }
        return Ok(a2a_messages)
    }

    fn decode_message(&self, message: &Message, expected_sender_vk: &str) -> VcxResult<A2AMessage> {
        trace!("Agent::decode_message >>> message = {:?}", json!(&message).to_string());
        EncryptionEnvelope::auth_unpack(message.payload()?, &expected_sender_vk)
    }

    fn decode_message_noauth(&self, message: &Message) -> VcxResult<A2AMessage> {
        trace!("Agent::decode_message_noatuh >>> message = {:?}", json!(&message).to_string());
        EncryptionEnvelope::anon_unpack(message.payload()?)
    }

    /**
    Sends authenticated message to connection counterparty
     */
    pub fn send_message(&self, message: &A2AMessage, did_dod: &DidDoc) -> VcxResult<()> {
        trace!("Agent::send_message >>> message: {:?}, did_doc: {:?}", message, did_dod);
        let envelope = EncryptionEnvelope::create(&message, Some(&self.pw_vk), &did_dod)?;
        httpclient::post_message(&envelope.0, &did_dod.get_endpoint())?;
        Ok(())
    }

    /**
    Sends anonymous message to connection counterparty
     */
    pub fn send_message_anonymously(message: &A2AMessage, did_dod: &DidDoc) -> VcxResult<()> {
        trace!("Agent::send_message_anonymously >>> message: {:?}, did_doc: {:?}", message, did_dod);
        let envelope = EncryptionEnvelope::create(&message, None, &did_dod)?;
        httpclient::post_message(&envelope.0, &did_dod.get_endpoint())?;
        Ok(())
    }

    /**
    Sends message to one's agency signalling resources related to this connection agent can be deleted.
     */
    pub fn delete(&self) -> VcxResult<()> {
        trace!("Agent::delete >>>");
        send_delete_connection_message(&self.pw_did, &self.pw_vk, &self.agent_did, &self.agent_vk)
    }
}
