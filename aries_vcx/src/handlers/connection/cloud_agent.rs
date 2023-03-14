use std::{collections::HashMap, sync::Arc};

use agency_client::{
    agency_client::AgencyClient, api::downloaded_message::DownloadedMessageEncrypted,
    messages::update_message::UIDsByConn, wallet::base_agency_client_wallet::BaseAgencyClientWallet,
};
use messages::a2a::A2AMessage;

use crate::{
    agency_client::MessageStatusCode, errors::error::prelude::*, plugins::wallet::agency_client_wallet::ToBaseWallet,
    protocols::mediated_connection::pairwise_info::PairwiseInfo, utils::encryption_envelope::EncryptionEnvelope,
};

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CloudAgentInfo {
    pub agent_did: String,
    pub agent_vk: String,
}

pub async fn create_agent_keys(
    agency_client: &AgencyClient,
    source_id: &str,
    pw_did: &str,
    pw_verkey: &str,
) -> VcxResult<(String, String)> {
    debug!("creating pairwise keys on agent for connection {}", source_id);
    trace!(
        "create_agent_keys >>> source_id: {}, pw_did: {}, pw_verkey: {}",
        source_id,
        pw_did,
        pw_verkey
    );

    let (agent_did, agent_verkey) = agency_client.create_connection_agent(pw_did, pw_verkey).await?;

    trace!(
        "create_agent_keys <<< agent_did: {}, agent_verkey: {}",
        agent_did,
        agent_verkey
    );
    Ok((agent_did, agent_verkey))
}

fn _log_messages_optionally(_a2a_messages: &HashMap<String, A2AMessage>) {
    #[cfg(feature = "warnlog_fetched_messages")]
    {
        for message in _a2a_messages.values() {
            let serialized_msg = serde_json::to_string_pretty(message)
                .unwrap_or_else(|_err| String::from("Failed to serialize A2AMessage."));
            warn!("Fetched decrypted connection messages:\n{}", serialized_msg);
        }
    }
}

impl CloudAgentInfo {
    pub async fn create(agency_client: &AgencyClient, pairwise_info: &PairwiseInfo) -> VcxResult<CloudAgentInfo> {
        trace!("CloudAgentInfo::create >>> pairwise_info: {:?}", pairwise_info);
        let (agent_did, agent_vk) =
            create_agent_keys(agency_client, "", &pairwise_info.pw_did, &pairwise_info.pw_vk).await?;
        Ok(CloudAgentInfo { agent_did, agent_vk })
    }

    pub async fn destroy(&self, agency_client: &AgencyClient, pairwise_info: &PairwiseInfo) -> VcxResult<()> {
        trace!("CloudAgentInfo::delete >>>");
        agency_client
            .delete_connection_agent(
                &pairwise_info.pw_did,
                &pairwise_info.pw_vk,
                &self.agent_did,
                &self.agent_vk,
            )
            .await
            .map_err(|err| err.into())
    }

    // todo: eliminate this function
    pub fn service_endpoint(&self, agency_client: &AgencyClient) -> VcxResult<String> {
        Ok(agency_client.get_agency_url_full())
    }

    // todo: implement this in agency_client
    pub fn routing_keys(&self, agency_client: &AgencyClient) -> VcxResult<Vec<String>> {
        let agency_vk = agency_client.get_agency_vk();
        Ok(vec![self.agent_vk.to_string(), agency_vk])
    }

    pub async fn update_message_status(
        &self,
        agency_client: &AgencyClient,
        pairwise_info: &PairwiseInfo,
        uid: String,
    ) -> VcxResult<()> {
        trace!("CloudAgentInfo::update_message_status >>> uid: {:?}", uid);

        let messages_to_update = vec![UIDsByConn {
            pairwise_did: pairwise_info.pw_did.clone(),
            uids: vec![uid],
        }];

        agency_client
            .update_messages(MessageStatusCode::Reviewed, messages_to_update)
            .await
            .map_err(|err| err.into())
    }

    pub async fn download_encrypted_messages(
        &self,
        agency_client: &AgencyClient,
        msg_uid: Option<Vec<String>>,
        status_codes: Option<Vec<MessageStatusCode>>,
        pairwise_info: &PairwiseInfo,
    ) -> VcxResult<Vec<DownloadedMessageEncrypted>> {
        trace!("CloudAgentInfo::download_encrypted_messages >>>");
        agency_client
            .get_encrypted_connection_messages(
                &pairwise_info.pw_did,
                &pairwise_info.pw_vk,
                &self.agent_did,
                &self.agent_vk,
                msg_uid,
                status_codes,
            )
            .await
            .map_err(|err| err.into())
    }

    pub async fn get_messages(
        &self,
        agency_client: &AgencyClient,
        expect_sender_vk: &str,
        pairwise_info: &PairwiseInfo,
    ) -> VcxResult<HashMap<String, A2AMessage>> {
        trace!(
            "CloudAgentInfo::get_messages >>> expect_sender_vk: {}",
            expect_sender_vk
        );
        let messages = self
            .download_encrypted_messages(
                agency_client,
                None,
                Some(vec![MessageStatusCode::Received]),
                pairwise_info,
            )
            .await?;
        debug!("CloudAgentInfo::get_messages >>> obtained {} messages", messages.len());
        let a2a_messages = self
            .decrypt_decode_messages(&agency_client.get_wallet(), &messages, expect_sender_vk)
            .await?;
        _log_messages_optionally(&a2a_messages);
        Ok(a2a_messages)
    }

    pub async fn get_messages_noauth(
        &self,
        agency_client: &AgencyClient,
        pairwise_info: &PairwiseInfo,
        uids: Option<Vec<String>>,
    ) -> VcxResult<HashMap<String, A2AMessage>> {
        trace!("CloudAgentInfo::get_messages_noauth >>>");
        let messages = self
            .download_encrypted_messages(
                agency_client,
                uids,
                Some(vec![MessageStatusCode::Received]),
                pairwise_info,
            )
            .await?;
        debug!(
            "CloudAgentInfo::get_messages_noauth >>> obtained {} messages",
            messages.len()
        );
        let a2a_messages = self
            .decrypt_decode_messages_noauth(&agency_client.get_wallet(), &messages)
            .await?;
        _log_messages_optionally(&a2a_messages);
        Ok(a2a_messages)
    }

    pub async fn get_message_by_id(
        &self,
        agency_client: &AgencyClient,
        msg_id: &str,
        expected_sender_vk: &str,
        pairwise_info: &PairwiseInfo,
    ) -> VcxResult<A2AMessage> {
        trace!("CloudAgentInfo::get_message_by_id >>> msg_id: {:?}", msg_id);
        let mut messages = self
            .download_encrypted_messages(agency_client, Some(vec![msg_id.to_string()]), None, pairwise_info)
            .await?;
        let message = messages.pop().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidMessages,
            format!("Message not found for id: {:?}", msg_id),
        ))?;
        let message = self
            .decrypt_decode_message(&agency_client.get_wallet(), &message, expected_sender_vk)
            .await?;
        Ok(message)
    }

    async fn decrypt_decode_messages(
        &self,
        wallet: &Arc<dyn BaseAgencyClientWallet>,
        messages: &Vec<DownloadedMessageEncrypted>,
        expected_sender_vk: &str,
    ) -> VcxResult<HashMap<String, A2AMessage>> {
        let mut a2a_messages: HashMap<String, A2AMessage> = HashMap::new();
        for message in messages {
            a2a_messages.insert(
                message.uid.clone(),
                self.decrypt_decode_message(wallet, message, expected_sender_vk).await?,
            );
        }
        Ok(a2a_messages)
    }

    async fn decrypt_decode_messages_noauth(
        &self,
        wallet: &Arc<dyn BaseAgencyClientWallet>,
        messages: &Vec<DownloadedMessageEncrypted>,
    ) -> VcxResult<HashMap<String, A2AMessage>> {
        let mut a2a_messages: HashMap<String, A2AMessage> = HashMap::new();
        for message in messages {
            a2a_messages.insert(
                message.uid.clone(),
                self.decrypt_decode_message_noauth(wallet, message).await?,
            );
        }
        Ok(a2a_messages)
    }

    async fn decrypt_decode_message(
        &self,
        wallet: &Arc<dyn BaseAgencyClientWallet>,
        message: &DownloadedMessageEncrypted,
        expected_sender_vk: &str,
    ) -> VcxResult<A2AMessage> {
        EncryptionEnvelope::auth_unpack(&wallet.to_base_wallet(), message.payload()?, expected_sender_vk).await
    }

    async fn decrypt_decode_message_noauth(
        &self,
        wallet: &Arc<dyn BaseAgencyClientWallet>,
        message: &DownloadedMessageEncrypted,
    ) -> VcxResult<A2AMessage> {
        Ok(
            EncryptionEnvelope::anon_unpack(&wallet.to_base_wallet(), message.payload()?)
                .await?
                .0,
        )
    }
}
