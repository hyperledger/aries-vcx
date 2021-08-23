use crate::error::prelude::*;
use crate::handlers::connection::cloud_agent::CloudAgentInfo;
use crate::handlers::connection::pairwise_info::PairwiseInfo;
use crate::messages::connection::invite::Invitation;
use crate::messages::connection::did_doc::{DidDoc, Service};
use crate::libindy::utils::ledger::add_attr;
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
        let service = serde_json::to_string(&service)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to serialize service before writing to ledger: {:?}", err)))?;
        add_attr(&institution_did, "service", &service)?;
        Ok(Self { agent_info, institution_did })
    }

    pub fn generate_public_invite(&self, source_id: &str, label: &str) -> VcxResult<Invitation> {
        let invite: Invitation = Invitation::create()
            .set_label(label.to_string())
            .set_id(source_id.to_string())
            .set_did(self.institution_did.to_string());
        Ok(invite)
    }
}

