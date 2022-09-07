use std::convert::TryFrom;

use futures::executor::block_on;

use crate::did_doc::service_aries::AriesService;
use crate::did_doc::DidDoc;
use crate::error::prelude::*;
use crate::handlers::out_of_band::OutOfBandInvitation;
use crate::libindy::utils::ledger;
use crate::messages::a2a::{A2AMessage, MessageId};
use crate::messages::connection::did::Did;
use crate::messages::timing::Timing;
use crate::timing_optional;
use crate::utils::service_resolvable::ServiceResolvable;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Invitation {
    Pairwise(PairwiseInvitation),
    Public(PublicInvitation),
    OutOfBand(OutOfBandInvitation),
}

// TODO: Make into TryFrom
impl From<Invitation> for DidDoc {
    fn from(invitation: Invitation) -> DidDoc {
        let mut did_doc: DidDoc = DidDoc::default();
        let (service_endpoint, recipient_keys, routing_keys) = match invitation {
            Invitation::Public(invitation) => {
                did_doc.set_id(invitation.did.to_string());
                let pool_handle = crate::global::pool::get_main_pool_handle().unwrap();
                let service = block_on(ledger::get_service(pool_handle, &invitation.did)).unwrap_or_else(|err| {
                    error!("Failed to obtain service definition from the ledger: {}", err);
                    AriesService::default()
                });
                (service.service_endpoint, service.recipient_keys, service.routing_keys)
            }
            Invitation::Pairwise(invitation) => {
                did_doc.set_id(invitation.id.0.clone());
                (
                    invitation.service_endpoint.clone(),
                    invitation.recipient_keys,
                    invitation.routing_keys,
                )
            }
            Invitation::OutOfBand(invitation) => {
                did_doc.set_id(invitation.id.0.clone());
                let service = block_on(invitation.services[0].resolve()).unwrap_or_else(|err| {
                    error!("Failed to obtain service definition from the ledger: {}", err);
                    AriesService::default()
                });
                (service.service_endpoint, service.recipient_keys, service.routing_keys)
            }
        };
        did_doc.set_service_endpoint(service_endpoint);
        did_doc.set_recipient_keys(recipient_keys);
        did_doc.set_routing_keys(routing_keys);
        did_doc
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct PairwiseInvitation {
    #[serde(rename = "@id")]
    pub id: MessageId,
    pub label: String,
    #[serde(rename = "recipientKeys")]
    pub recipient_keys: Vec<String>,
    #[serde(default)]
    #[serde(rename = "routingKeys")]
    pub routing_keys: Vec<String>,
    #[serde(rename = "serviceEndpoint")]
    pub service_endpoint: String,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

timing_optional!(PairwiseInvitation);

impl From<DidDoc> for PairwiseInvitation {
    fn from(did_doc: DidDoc) -> PairwiseInvitation {
        let recipient_keys = did_doc.recipient_keys();
        let routing_keys = did_doc.routing_keys();

        PairwiseInvitation::create()
            .set_id(&did_doc.id)
            .set_service_endpoint(did_doc.get_endpoint())
            .set_recipient_keys(recipient_keys)
            .set_routing_keys(routing_keys)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct PublicInvitation {
    #[serde(rename = "@id")]
    pub id: MessageId,
    pub label: String,
    pub did: Did,
}

impl Invitation {
    pub fn get_id(&self) -> VcxResult<String> {
        match self {
            Self::Pairwise(invite) => Ok(invite.id.0.clone()),
            Self::Public(invite) => Ok(invite.id.0.clone()),
            Self::OutOfBand(invite) => Ok(invite.id.0.clone()),
        }
    }
}

impl PairwiseInvitation {
    pub fn create() -> Self {
        Self::default()
    }

    pub fn set_label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    pub fn set_id(mut self, id: &str) -> Self {
        self.id = MessageId(id.to_string());
        self
    }

    pub fn set_service_endpoint(mut self, service_endpoint: String) -> Self {
        self.service_endpoint = service_endpoint;
        self
    }

    pub fn set_recipient_keys(mut self, recipient_keys: Vec<String>) -> Self {
        self.recipient_keys = recipient_keys;
        self
    }

    pub fn set_routing_keys(mut self, routing_keys: Vec<String>) -> Self {
        self.routing_keys = routing_keys;
        self
    }
}

impl PublicInvitation {
    pub fn create() -> Self {
        Self::default()
    }

    pub fn set_label(mut self, label: &str) -> Self {
        self.label = label.into();
        self
    }

    pub fn set_id(mut self, id: &str) -> Self {
        self.id = MessageId(id.into());
        self
    }

    pub fn set_public_did(mut self, public_did: &str) -> VcxResult<Self> {
        self.did = Did::new(public_did)?;
        Ok(self)
    }
}

impl TryFrom<&ServiceResolvable> for PairwiseInvitation {
    type Error = VcxError;
    fn try_from(service_resolvable: &ServiceResolvable) -> Result<Self, Self::Error> {
        let service = block_on(service_resolvable.resolve())?;
        Ok(Self::create()
            .set_recipient_keys(service.recipient_keys)
            .set_routing_keys(service.routing_keys)
            .set_service_endpoint(service.service_endpoint))
    }
}

a2a_message!(PairwiseInvitation, ConnectionInvitationPairwise);
a2a_message!(PublicInvitation, ConnectionInvitationPublic);

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use crate::did_doc::test_utils::*;
    use crate::utils::uuid;

    use super::*;

    pub fn _pairwise_invitation() -> PairwiseInvitation {
        PairwiseInvitation {
            id: MessageId::id(),
            label: _label(),
            recipient_keys: _recipient_keys(),
            routing_keys: _routing_keys(),
            service_endpoint: _service_endpoint(),
            timing: None,
        }
    }

    pub fn _public_invitation() -> PublicInvitation {
        PublicInvitation {
            id: MessageId::id(),
            label: _label(),
            did: Did::new(&_did()).unwrap(),
        }
    }

    pub fn _pairwise_invitation_random_id() -> PairwiseInvitation {
        PairwiseInvitation {
            id: MessageId(uuid::uuid()),
            .._pairwise_invitation()
        }
    }

    pub fn _public_invitation_random_id() -> PublicInvitation {
        PublicInvitation {
            id: MessageId(uuid::uuid()),
            .._public_invitation()
        }
    }

    pub fn _pairwise_invitation_json() -> String {
        serde_json::to_string(&_pairwise_invitation().to_a2a_message()).unwrap()
    }

    pub fn _public_invitation_json() -> String {
        serde_json::to_string(&_public_invitation().to_a2a_message()).unwrap()
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use crate::did_doc::test_utils::*;
    use crate::messages::connection::invite::test_utils::{_pairwise_invitation, _public_invitation};

    use super::*;

    #[test]
    fn test_pairwise_invite_build_works() {
        let invitation: PairwiseInvitation = PairwiseInvitation::default()
            .set_label(&_label())
            .set_service_endpoint(_service_endpoint())
            .set_recipient_keys(_recipient_keys())
            .set_routing_keys(_routing_keys());

        assert_eq!(_pairwise_invitation(), invitation);
    }

    #[test]
    fn test_public_invite_build_works() {
        let invitation: PublicInvitation = PublicInvitation::default()
            .set_label(&_label())
            .set_public_did(&_did())
            .unwrap();

        assert_eq!(_public_invitation(), invitation);
    }

    #[test]
    fn test_did_doc_from_invitation_works() {
        let mut did_doc = DidDoc::default();
        did_doc.set_id(MessageId::id().0);
        did_doc.set_service_endpoint(_service_endpoint());
        did_doc.set_recipient_keys(_recipient_keys());
        did_doc.set_routing_keys(_routing_keys());

        assert_eq!(did_doc, DidDoc::from(Invitation::Pairwise(_pairwise_invitation())));
    }
}
