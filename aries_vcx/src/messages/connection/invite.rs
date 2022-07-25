use std::convert::TryFrom;
use futures::executor::block_on;

use crate::error::prelude::*;
use crate::handlers::out_of_band::OutOfBandInvitation;
use crate::messages::a2a::{A2AMessage, MessageId};
use crate::messages::connection::did::Did;
use crate::messages::connection::service::ServiceResolvable;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Invitation {
    Pairwise(PairwiseInvitation),
    Public(PublicInvitation),
    OutOfBand(OutOfBandInvitation)
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
            Self::OutOfBand(invite) => Ok(invite.id.0.clone())
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
    fn try_from(service: &ServiceResolvable) -> Result<Self, Self::Error> {
        let full_service = block_on(service.resolve())?;
        Ok(Self::create()
            .set_recipient_keys(full_service.recipient_keys)
            .set_routing_keys(full_service.routing_keys)
            .set_service_endpoint(full_service.service_endpoint))
    }
}

a2a_message!(PairwiseInvitation, ConnectionInvitationPairwise);
a2a_message!(PublicInvitation, ConnectionInvitationPublic);

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use crate::messages::connection::did_doc::test_utils::*;
    use crate::utils::uuid;
    use super::*;

    pub fn _pairwise_invitation() -> PairwiseInvitation {
        PairwiseInvitation {
            id: MessageId::id(),
            label: _label(),
            recipient_keys: _recipient_keys(),
            routing_keys: _routing_keys(),
            service_endpoint: _service_endpoint(),
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
    use crate::messages::connection::did_doc::test_utils::*;
    use crate::messages::connection::invite::test_utils::{_pairwise_invitation, _public_invitation};
    use super::*;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_pairwise_invite_build_works() {
        let invitation: PairwiseInvitation = PairwiseInvitation::default()
            .set_label(&_label())
            .set_service_endpoint(_service_endpoint())
            .set_recipient_keys(_recipient_keys())
            .set_routing_keys(_routing_keys());

        assert_eq!(_pairwise_invitation(), invitation);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_public_invite_build_works() {
        let invitation: PublicInvitation = PublicInvitation::default()
            .set_label(&_label())
            .set_public_did(&_did()).unwrap();

        assert_eq!(_public_invitation(), invitation);
    }
}
