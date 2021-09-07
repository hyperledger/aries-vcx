use std::collections::HashMap;

use crate::agency_client::get_message::{get_connection_messages, Message};
use crate::agency_client::MessageStatusCode;
use crate::agency_client::update_connection::send_delete_connection_message;
use crate::agency_client::update_message::{UIDsByConn, update_messages as update_messages_status};
use crate::error::prelude::*;
use crate::handlers::connection::pairwise_info::PairwiseInfo;
use crate::messages::a2a::A2AMessage;
use crate::settings;
use crate::utils::encryption_envelope::EncryptionEnvelope;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CloudAgentInfo {
    pub agent_did: String,
    pub agent_vk: String,
}

impl Default for CloudAgentInfo {
    fn default() -> CloudAgentInfo {
        CloudAgentInfo {
            agent_did: String::new(),
            agent_vk: String::new(),
        }
    }
}

pub fn create_agent_keys(source_id: &str, pw_did: &str, pw_verkey: &str) -> VcxResult<(String, String)> {
    debug!("creating pairwise keys on agent for connection {}", source_id);
    trace!("create_agent_keys >>> source_id: {}, pw_did: {}, pw_verkey: {}", source_id, pw_did, pw_verkey);

    let (agent_did, agent_verkey) = agency_client::create_keys()
        .for_did(pw_did)?
        .for_verkey(pw_verkey)?
        .send_secure()
        .map_err(|err| err.extend("Cannot create pairwise keys"))?;

    trace!("create_agent_keys <<< agent_did: {}, agent_verkey: {}", agent_did, agent_verkey);
    Ok((agent_did, agent_verkey))
}

fn _log_messages_optionally(_a2a_messages: &HashMap<String, A2AMessage>) {
    #[cfg(feature = "warnlog_fetched_messages")]
        {
            for message in _a2a_messages.values() {
                let serialized_msg = serde_json::to_string_pretty(message).unwrap_or_else(|_err| String::from("Failed to serialize A2AMessage."));
                warn!("Fetched decrypted connection messages:\n{}", serialized_msg);
            }
        }
}

impl CloudAgentInfo {
    pub fn create(pairwise_info: &PairwiseInfo) -> VcxResult<CloudAgentInfo> {
        trace!("CloudAgentInfo::create >>> pairwise_info: {:?}", pairwise_info);
        let (agent_did, agent_vk) = create_agent_keys("", &pairwise_info.pw_did, &pairwise_info.pw_vk)?;
        Ok(CloudAgentInfo { agent_did, agent_vk })
    }

    pub fn destroy(&self, pairwise_info: &PairwiseInfo) -> VcxResult<()> {
        trace!("CloudAgentInfo::delete >>>");
        send_delete_connection_message(&pairwise_info.pw_did, &pairwise_info.pw_vk, &self.agent_did, &self.agent_vk)
            .map_err(|err| err.into())
    }

    pub fn service_endpoint(&self) -> VcxResult<String> {
        settings::get_agency_client()?.get_agency_url()
            .map_err(|err| err.into())
    }

    pub fn routing_keys(&self) -> VcxResult<Vec<String>> {
        let agency_vk = &settings::get_agency_client()?.get_agency_vk()?;
        Ok(vec![self.agent_vk.to_string(), agency_vk.to_string()])
    }

    pub fn update_message_status(&self, pairwise_info: &PairwiseInfo, uid: String) -> VcxResult<()> {
        trace!("CloudAgentInfo::update_message_status >>> uid: {:?}", uid);

        let messages_to_update = vec![UIDsByConn {
            pairwise_did: pairwise_info.pw_did.clone(),
            uids: vec![uid],
        }];

        update_messages_status(MessageStatusCode::Reviewed, messages_to_update)
            .map_err(|err| err.into())
    }

    pub fn reject_message(&self, pairwise_info: &PairwiseInfo, uid: String) -> VcxResult<()> {
        trace!("CloudAgentInfo::reject_message >>> uid: {:?}", uid);

        let messages_to_reject = vec![UIDsByConn {
            pairwise_did: pairwise_info.pw_did.clone(),
            uids: vec![uid],
        }];

        update_messages_status(MessageStatusCode::Rejected, messages_to_reject)
            .map_err(|err| err.into())
    }

    pub fn download_encrypted_messages(&self, msg_uid: Option<Vec<String>>, status_codes: Option<Vec<MessageStatusCode>>, pairwise_info: &PairwiseInfo) -> VcxResult<Vec<Message>> {
        trace!("CloudAgentInfo::download_encrypted_messages >>>");
        get_connection_messages(&pairwise_info.pw_did, &pairwise_info.pw_vk, &self.agent_did, &self.agent_vk, msg_uid, status_codes)
            .map_err(|err| err.into())
    }

    pub fn get_messages(&self, expect_sender_vk: &str, pairwise_info: &PairwiseInfo) -> VcxResult<HashMap<String, A2AMessage>> {
        trace!("CloudAgentInfo::get_messages >>> expect_sender_vk: {}", expect_sender_vk);
        let messages = self.download_encrypted_messages(None, Some(vec![MessageStatusCode::Received]), pairwise_info)?;
        debug!("CloudAgentInfo::get_messages >>> obtained {} messages", messages.len());
        let a2a_messages = self.decrypt_decode_messages(&messages, expect_sender_vk)?;
        _log_messages_optionally(&a2a_messages);
        Ok(a2a_messages)
    }

    pub fn get_messages_noauth(&self, pairwise_info: &PairwiseInfo) -> VcxResult<HashMap<String, A2AMessage>> {
        trace!("CloudAgentInfo::get_messages_noauth >>>");
        let messages = self.download_encrypted_messages(None, Some(vec![MessageStatusCode::Received]), pairwise_info)?;
        debug!("CloudAgentInfo::get_messages_noauth >>> obtained {} messages", messages.len());
        let a2a_messages = self.decrypt_decode_messages_noauth(&messages)?;
        _log_messages_optionally(&a2a_messages);
        Ok(a2a_messages)
    }

    pub fn get_message_by_id(&self, msg_id: &str, expected_sender_vk: &str, pairwise_info: &PairwiseInfo) -> VcxResult<A2AMessage> {
        trace!("CloudAgentInfo::get_message_by_id >>> msg_id: {:?}", msg_id);
        let mut messages = self.download_encrypted_messages(Some(vec![msg_id.to_string()]), None, pairwise_info)?;
        let message = messages
            .pop()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidMessages, format!("Message not found for id: {:?}", msg_id)))?;
        let message = self.decrypt_decode_message(&message, &expected_sender_vk)?;
        Ok(message)
    }

    fn decrypt_decode_messages(&self, messages: &Vec<Message>, expected_sender_vk: &str) -> VcxResult<HashMap<String, A2AMessage>> {
        let mut a2a_messages: HashMap<String, A2AMessage> = HashMap::new();
        for message in messages {
            a2a_messages.insert(message.uid.clone(), self.decrypt_decode_message(&message, expected_sender_vk)?);
        }
        return Ok(a2a_messages);
    }

    fn decrypt_decode_messages_noauth(&self, messages: &Vec<Message>) -> VcxResult<HashMap<String, A2AMessage>> {
        let mut a2a_messages: HashMap<String, A2AMessage> = HashMap::new();
        for message in messages {
            a2a_messages.insert(message.uid.clone(), self.decrypt_decode_message_noauth(&message)?);
        }
        return Ok(a2a_messages);
    }

    fn decrypt_decode_message(&self, message: &Message, expected_sender_vk: &str) -> VcxResult<A2AMessage> {
        EncryptionEnvelope::auth_unpack(message.payload()?, &expected_sender_vk)
    }

    fn decrypt_decode_message_noauth(&self, message: &Message) -> VcxResult<A2AMessage> {
        EncryptionEnvelope::anon_unpack(message.payload()?)
    }
}
