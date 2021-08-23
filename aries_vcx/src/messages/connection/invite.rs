use crate::messages::a2a::{A2AMessage, MessageId};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct Invitation {
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
    #[serde(skip_serializing_if = "String::is_empty")]
    pub did: String
}

impl Invitation {
    pub fn create() -> Invitation {
        Invitation::default()
    }

    pub fn set_label(mut self, label: String) -> Invitation {
        self.label = label;
        self
    }

    pub fn set_id(mut self, id: String) -> Invitation {
        self.id = MessageId(id);
        self
    }

    pub fn set_service_endpoint(mut self, service_endpoint: String) -> Invitation {
        self.service_endpoint = service_endpoint;
        self
    }

    pub fn set_recipient_keys(mut self, recipient_keys: Vec<String>) -> Invitation {
        self.recipient_keys = recipient_keys;
        self
    }

    pub fn set_routing_keys(mut self, routing_keys: Vec<String>) -> Invitation {
        self.routing_keys = routing_keys;
        self
    }

    pub fn set_did(mut self, did: String) -> Invitation {
        self.did = did;
        self
    }
}

a2a_message!(Invitation, ConnectionInvitation);

// #[cfg(test)]
pub mod tests {
    use super::*;
    use crate::messages::connection::did_doc::tests::*;

    pub fn _invitation() -> Invitation {
        Invitation {
            id: MessageId::id(),
            label: _label(),
            recipient_keys: _recipient_keys(),
            routing_keys: _routing_keys(),
            service_endpoint: _service_endpoint(),
            did: _did()
        }
    }

    pub fn _invitation_json() -> String {
        serde_json::to_string(&_invitation().to_a2a_message()).unwrap()
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_request_build_works() {
        let invitation: Invitation = Invitation::default()
            .set_label(_label())
            .set_service_endpoint(_service_endpoint())
            .set_recipient_keys(_recipient_keys())
            .set_routing_keys(_routing_keys());

        assert_eq!(_invitation(), invitation);
    }
}
