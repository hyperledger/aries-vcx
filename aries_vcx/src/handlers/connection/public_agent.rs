use crate::error::prelude::*;
use crate::handlers::connection::cloud_agent::CloudAgentInfo;
use crate::handlers::connection::pairwise_info::PairwiseInfo;
use crate::messages::connection::invite::PublicInvitation;
use crate::messages::connection::did_doc::{DidDoc, Service};
use crate::libindy::utils::ledger::add_service;
use crate::settings::get_agency_client;

pub struct PublicAgent {
    agent_info: CloudAgentInfo,
    institution_did: String
}

impl PublicAgent {
    pub fn create(institution_did: &str) -> VcxResult<Self> {
        let pairwise_info = PairwiseInfo::create()?;
        let agent_info = CloudAgentInfo::create(&pairwise_info)?;
        let institution_did = String::from(institution_did);
        let service = Service {
            service_endpoint: get_agency_client()?.get_agency_url()?,
            recipient_keys: vec![pairwise_info.pw_vk.clone()],
            routing_keys: agent_info.routing_keys()?,
            ..Default::default()
        };
        add_service(&institution_did, &service)?;
        Ok(Self { agent_info, institution_did })
    }

    pub fn generate_public_invite(&self, label: &str) -> VcxResult<PublicInvitation> {
        let invite: PublicInvitation = PublicInvitation::create()
            .set_label(label.to_string())
            .set_public_did(self.institution_did.to_string());
        Ok(invite)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::utils::devsetup::*;

    use crate::messages::a2a::MessageId;

    static INSTITUTION_DID: &str = "2hoqvcwupRTUNkXn6ArYzs";
    static LABEL: &str = "hello";

    #[test]
    #[cfg(feature = "general_test")]
    fn test_generate_public_invite() {
        let _setup = SetupMocks::init();
        let expected_invite = PublicInvitation {
            id: MessageId("testid".to_string()),
            label: "hello".to_string(),
            did: "2hoqvcwupRTUNkXn6ArYzs".to_string()
        };
        let agent = PublicAgent::create(INSTITUTION_DID).unwrap();
        let invite = agent.generate_public_invite(LABEL).unwrap();
        assert_eq!(expected_invite, invite);
    }
}
