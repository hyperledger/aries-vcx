use std::convert::TryFrom;

use futures::stream::iter;
use futures::StreamExt;

use crate::error::prelude::*;
use crate::handlers::connection::cloud_agent::CloudAgentInfo;
use crate::libindy::utils::ledger::add_service;
use crate::messages::a2a::A2AMessage;
use crate::messages::connection::did::Did;
use crate::messages::connection::request::Request;
use crate::messages::connection::service::FullService;
use crate::protocols::connection::pairwise_info::PairwiseInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicAgent {
    source_id: String,
    agent_info: CloudAgentInfo,
    pairwise_info: PairwiseInfo,
    institution_did: Did,
}

impl PublicAgent {
    pub async fn create(source_id: &str, institution_did: &str) -> VcxResult<Self> {
        let pairwise_info = PairwiseInfo::create().await?;
        let agent_info = CloudAgentInfo::create(&pairwise_info).await?;
        let service = FullService::try_from((&pairwise_info, &agent_info))?;
        add_service(&institution_did, &service).await?;
        let institution_did = Did::new(institution_did)?;
        let source_id = String::from(source_id);
        Ok(Self { source_id, agent_info, pairwise_info, institution_did })
    }

    pub fn pairwise_info(&self) -> PairwiseInfo {
        self.pairwise_info.clone()
    }

    pub fn cloud_agent_info(&self) -> CloudAgentInfo {
        self.agent_info.clone()
    }

    pub fn did(&self) -> String {
        self.institution_did.to_string()
    }

    pub fn service(&self) -> VcxResult<FullService> {
        FullService::try_from(self)
    }

    pub async fn download_connection_requests(&self, uids: Option<Vec<String>>) -> VcxResult<Vec<Request>> {
        let connection_requests: Vec<Request> = iter(self.agent_info.get_messages_noauth(&self.pairwise_info, uids.clone())
            .await?
            .into_iter())
            .filter_map(|(uid, message)| async {
                match message {
                    // TODO: Rewrite once if let chains become stable: https://github.com/rust-lang/rust/issues/53667
                    A2AMessage::ConnectionRequest(request) => match &uids {
                        Some(uids) => if uids.contains(&uid) {
                            self.agent_info.update_message_status(&self.pairwise_info, uid).await.ok()?;
                            Some(request)
                        } else {
                            None
                        }
                        None => {
                            self.agent_info.update_message_status(&self.pairwise_info, uid).await.ok()?;
                            Some(request)
                        }
                    }
                    _ => {
                        self.agent_info.update_message_status(&self.pairwise_info, uid).await.ok()?;
                        None
                    }
                }
            })
            .collect()
            .await;
        Ok(connection_requests)
    }

    pub async fn download_message(&self, uid: &str) -> VcxResult<A2AMessage> {
        self.agent_info.get_messages_noauth(&self.pairwise_info, Some(vec![uid.to_string()]))
            .await?
            .into_iter()
            .find(|(uid_, _)| uid == uid_)
            .map(|(_, message)| message)
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidMessages, format!("Message not found for id: {:?}", uid)))
    }

    pub fn to_string(&self) -> VcxResult<String> {
        serde_json::to_string(&self)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize Agent: {:?}", err)))
    }

    pub fn from_string(agent_data: &str) -> VcxResult<Self> {
        serde_json::from_str(agent_data)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize Agent: {:?}", err)))
    }
}

impl From<&PublicAgent> for PairwiseInfo {
    fn from(agent: &PublicAgent) -> Self {
        agent.pairwise_info()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    static INSTITUTION_DID: &str = "2hoqvcwupRTUNkXn6ArYzs";

    pub fn _public_agent() -> PublicAgent {
        PublicAgent {
            source_id: "test-public-agent".to_string(),
            agent_info: CloudAgentInfo {
                agent_did: "NaMhQmSjkWoi5aVWEkA9ya".to_string(),
                agent_vk: "Cm2rgfweypyJ5u9h46ZnqcJrCVYvgau1DAuVJV6MgVBc".to_string(),
            },
            pairwise_info: PairwiseInfo {
                pw_did: "FgjjUduQaJnH4HiEVfViTp".to_string(),
                pw_vk: "91E5YBaQVnY2dLbv2mrfFQB1y2wPyYuYVPKziamrZiuS".to_string(),
            },
            institution_did: Did::new(INSTITUTION_DID).unwrap()
        }
    }
}
