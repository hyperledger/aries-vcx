use crate::messages::a2a::{A2AMessage, MessageId};
use crate::did_doc::DidDoc;
use crate::messages::thread::Thread;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Request {
    #[serde(rename = "@id")]
    pub id: MessageId,
    pub label: String,
    pub connection: ConnectionData,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct ConnectionData {
    #[serde(rename = "DID")]
    pub did: String,
    #[serde(rename = "DIDDoc")]
    pub did_doc: DidDoc,
}

impl Request {
    pub fn create() -> Request {
        Request::default()
    }

    pub fn set_did(mut self, did: String) -> Request {
        self.connection.did = did.clone();
        self.connection.did_doc.set_id(did);
        self
    }

    pub fn set_label(mut self, label: String) -> Request {
        self.label = label;
        self
    }

    pub fn set_service_endpoint(mut self, service_endpoint: String) -> Request {
        self.connection.did_doc.set_service_endpoint(service_endpoint);
        self
    }

    pub fn set_keys(mut self, recipient_keys: Vec<String>, routing_keys: Vec<String>) -> Request {
        self.connection.did_doc.set_recipient_keys(recipient_keys);
        self.connection.did_doc.set_routing_keys(routing_keys);
        self
    }
}

a2a_message!(Request, ConnectionRequest);
threadlike_optional!(Request);

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use crate::did_doc::test_utils::*;

    use super::*;

    fn _did() -> String {
        String::from("VsKV7grR1BUE29mG2Fm2kX")
    }

    pub fn _request() -> Request {
        Request {
            id: MessageId::id(),
            label: _label(),
            connection: ConnectionData {
                did: _did(),
                did_doc: _did_doc_inlined_recipient_keys(),
            },
            thread: None,
        }
    }

    #[test]
    fn test_request_build_works() {
        let request: Request = Request::default()
            .set_did(_did())
            .set_label(_label())
            .set_service_endpoint(_service_endpoint())
            .set_keys(_recipient_keys(), _routing_keys());

        assert_eq!(_request(), request);
    }
}
