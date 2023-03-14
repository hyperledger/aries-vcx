use diddoc::aries::diddoc::AriesDidDoc;

use crate::{
    a2a::{message_family::MessageFamilies, message_type::MessageType, A2AMessage, MessageId},
    concepts::{ack::please_ack::PleaseAck, thread::Thread, timing::Timing},
    timing_optional,
};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Response {
    #[serde(rename = "@id")]
    pub id: MessageId,
    pub connection: ConnectionData,
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>,
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct ConnectionData {
    #[serde(rename = "DID")]
    pub did: String,
    #[serde(rename = "DIDDoc")]
    pub did_doc: AriesDidDoc,
}

please_ack!(Response);
threadlike!(Response);
timing_optional!(Response);

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct SignedResponse {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "connection~sig")]
    pub connection_sig: ConnectionSignature,
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

threadlike!(SignedResponse);
timing_optional!(SignedResponse);

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ConnectionSignature {
    #[serde(rename = "@type")]
    pub msg_type: MessageType,
    pub signature: String,
    pub sig_data: String,
    pub signer: String,
}

impl Response {
    pub fn create() -> Response {
        Response::default()
    }

    pub fn get_connection_data(&self) -> String {
        json!(self.connection).to_string()
    }

    pub fn set_did(mut self, did: String) -> Response {
        self.connection.did = did.clone();
        self.connection.did_doc.set_id(did);
        self
    }

    pub fn set_service_endpoint(mut self, service_endpoint: String) -> Response {
        self.connection.did_doc.set_service_endpoint(service_endpoint);
        self
    }

    pub fn set_keys(mut self, recipient_keys: Vec<String>, routing_keys: Vec<String>) -> Response {
        self.connection.did_doc.set_recipient_keys(recipient_keys);
        self.connection.did_doc.set_routing_keys(routing_keys);
        self
    }
}

a2a_message!(SignedResponse, ConnectionResponse);

impl Default for ConnectionSignature {
    fn default() -> ConnectionSignature {
        ConnectionSignature {
            msg_type: MessageType::build(MessageFamilies::Signature, "ed25519Sha512_single"),
            signature: String::new(),
            sig_data: String::new(),
            signer: String::new(),
        }
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use diddoc::aries::diddoc::test_utils::_did_doc_inlined_recipient_keys;

    use super::*;

    pub fn _did() -> String {
        String::from("VsKV7grR1BUE29mG2Fm2kX")
    }

    pub fn _key() -> String {
        String::from("CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW")
    }

    pub fn _thread() -> Thread {
        Thread::new().set_thid(String::from("testid"))
    }

    pub fn _thread_random() -> Thread {
        Thread::new().set_thid(uuid::Uuid::new_v4().to_string())
    }

    pub fn _thread_1() -> Thread {
        Thread::new().set_thid(String::from("testid_1"))
    }

    pub fn _thread_id() -> String {
        _thread().thid.unwrap()
    }

    pub fn _response() -> Response {
        Response {
            id: MessageId::id(),
            thread: _thread(),
            connection: ConnectionData {
                did: _did(),
                did_doc: _did_doc_inlined_recipient_keys(),
            },
            please_ack: None,
            timing: None,
        }
    }

    pub fn _signed_response() -> SignedResponse {
        SignedResponse {
            id: MessageId::id(),
            thread: _thread(),
            connection_sig: ConnectionSignature {
                signature: String::from(
                    "yeadfeBWKn09j5XU3ITUE3gPbUDmPNeblviyjrOIDdVMT5WZ8wxMCxQ3OpAnmq1o-Gz0kWib9zr0PLsbGc2jCA==",
                ),
                sig_data: serde_json::to_string(&_did_doc_inlined_recipient_keys()).unwrap(),
                signer: _key(),
                ..Default::default()
            },
            please_ack: None,
            timing: None,
        }
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use diddoc::aries::diddoc::test_utils::*;

    use super::*;
    use crate::protocols::connection::response::test_utils::{_did, _response, _thread_id};

    #[test]
    fn test_response_build_works() {
        let response: Response = Response::default()
            .set_did(_did())
            .set_thread_id(&_thread_id())
            .set_service_endpoint(_service_endpoint())
            .set_keys(_recipient_keys(), _routing_keys());

        assert_eq!(_response(), response);
    }
}
