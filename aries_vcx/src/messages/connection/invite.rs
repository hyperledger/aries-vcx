use crate::messages::a2a::{A2AMessage, MessageId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Invitation {
    Pairwise(PairwiseInvitation),
    Public(PublicInvitation),
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
    pub did: String,
}

impl PairwiseInvitation {
    pub fn create() -> Self {
        Self::default()
    }

    pub fn set_label(mut self, label: String) -> Self {
        self.label = label;
        self
    }

    pub fn set_id(mut self, id: String) -> Self {
        self.id = MessageId(id);
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

    pub fn set_label(mut self, label: String) -> Self {
        self.label = label;
        self
    }

    pub fn set_id(mut self, id: String) -> Self {
        self.id = MessageId(id);
        self
    }

    pub fn set_public_did(mut self, public_did: String) -> Self {
        self.did = public_did;
        self
    }
}

a2a_message!(PairwiseInvitation, ConnectionInvitationPairwise);
a2a_message!(PublicInvitation, ConnectionInvitationPublic);

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use crate::messages::connection::did_doc::test_utils::*;

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
            did: _did(),
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
pub mod tests {
    use crate::messages::connection::did_doc::test_utils::*;
    use crate::messages::connection::invite::test_utils::{_pairwise_invitation, _public_invitation};

    use super::*;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_pairwise_invite_build_works() {
        let invitation: PairwiseInvitation = PairwiseInvitation::default()
            .set_label(_label())
            .set_service_endpoint(_service_endpoint())
            .set_recipient_keys(_recipient_keys())
            .set_routing_keys(_routing_keys());

        assert_eq!(_pairwise_invitation(), invitation);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_public_invite_build_works() {
        let invitation: PublicInvitation = PublicInvitation::default()
            .set_label(_label())
            .set_public_did(_did());

        assert_eq!(_public_invitation(), invitation);
    }
}
