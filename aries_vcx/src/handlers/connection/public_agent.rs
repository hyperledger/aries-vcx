use crate::error::prelude::*;
use crate::handlers::connection::cloud_agent::CloudAgentInfo;
use crate::handlers::connection::pairwise_info::PairwiseInfo;
use crate::messages::connection::invite::PublicInvitation;
use crate::messages::connection::did_doc::{DidDoc, Service};
use crate::libindy::utils::ledger::add_service;
use crate::settings::get_agency_client;
use crate::agency_client::get_message::{get_connection_messages, Message};
use crate::messages::connection::request::Request;
use crate::messages::a2a::A2AMessage;

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicAgent {
    source_id: String,
    agent_info: CloudAgentInfo,
    pairwise_info: PairwiseInfo,
    institution_did: String
}

impl PublicAgent {
    pub fn create(source_id: &str, institution_did: &str) -> VcxResult<Self> {
        let pairwise_info = PairwiseInfo::create()?;
        let agent_info = CloudAgentInfo::create(&pairwise_info)?;
        let institution_did = String::from(institution_did);
        let source_id = String::from(source_id);
        let service = Service {
            service_endpoint: get_agency_client()?.get_agency_url()?,
            recipient_keys: vec![pairwise_info.pw_vk.clone()],
            routing_keys: agent_info.routing_keys()?,
            ..Default::default()
        };
        add_service(&institution_did, &service)?;
        Ok(Self { source_id, agent_info, pairwise_info, institution_did })
    }

    pub fn pairwise_info(&self) -> PairwiseInfo {
        self.pairwise_info.clone()
    }

    pub fn cloud_agent_info(&self) -> CloudAgentInfo {
        self.agent_info.clone()
    }

    pub fn generate_public_invite(&self, label: &str) -> VcxResult<PublicInvitation> {
        let invite: PublicInvitation = PublicInvitation::create()
            .set_label(label.to_string())
            .set_public_did(self.institution_did.to_string());
        Ok(invite)
    }

    pub fn download_connection_requests(&self) -> VcxResult<Vec<Request>> {
        let connection_requests: Vec<Request> = self.agent_info.get_messages_noauth(&self.pairwise_info)?
            .into_iter()
            .filter_map(|(uid, message)| {
                match message {
                    A2AMessage::ConnectionRequest(request) => {
                        self.agent_info.update_message_status(&self.pairwise_info, uid).ok()?;
                        Some(request)
                    }
                    _ => None
                }
            })
            .collect();
       Ok(connection_requests) 
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

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::utils::devsetup::*;

    use crate::messages::a2a::MessageId;

    static INSTITUTION_DID: &str = "2hoqvcwupRTUNkXn6ArYzs";
    static LABEL: &str = "hello";

    pub fn _public_agent() -> PublicAgent {
        PublicAgent {
            source_id: "test-public-agent".to_string(),
            agent_info: CloudAgentInfo {
                agent_did: "NaMhQmSjkWoi5aVWEkA9ya".to_string(),
                agent_vk: "Cm2rgfweypyJ5u9h46ZnqcJrCVYvgau1DAuVJV6MgVBc".to_string()
            },
            pairwise_info: PairwiseInfo {
                pw_did: "FgjjUduQaJnH4HiEVfViTp".to_string(),
                pw_vk: "91E5YBaQVnY2dLbv2mrfFQB1y2wPyYuYVPKziamrZiuS".to_string()
            },
            institution_did: INSTITUTION_DID.to_string()
        }
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_generate_public_invite() {
        let _setup = SetupMocks::init();
        let expected_invite = PublicInvitation {
            id: MessageId("testid".to_string()),
            label: "hello".to_string(),
            did: "2hoqvcwupRTUNkXn6ArYzs".to_string()
        };
        let agent = PublicAgent::create("testid", INSTITUTION_DID).unwrap();
        let invite = agent.generate_public_invite(LABEL).unwrap();
        assert_eq!(expected_invite, invite);
    }
}
