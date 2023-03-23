use diddoc::aries::diddoc::AriesDidDoc;
use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{thread::Thread, timing::Timing},
    message::Message,
    msg_types::types::connection::ConnectionV1_0,
};

pub type Request = Message<RequestContent, RequestDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "ConnectionV1_0::Request")]
pub struct RequestContent {
    pub label: String,
    pub connection: ConnectionData,
}

impl RequestContent {
    pub fn new(label: String, connection: ConnectionData) -> Self {
        Self { label, connection }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ConnectionData {
    #[serde(rename = "DID")]
    pub did: String,
    #[serde(rename = "DIDDoc")]
    pub did_doc: AriesDidDoc,
}

impl ConnectionData {
    pub fn new(did: String, did_doc: AriesDidDoc) -> Self {
        Self { did, did_doc }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
pub struct RequestDecorators {
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{thread::tests::make_extended_thread, timing::tests::make_extended_timing},
        misc::test_utils,
    };

    #[test]
    fn test_minimal_conn_request() {
        let did_doc = AriesDidDoc::default(); // We really need to improve this creation.
        let conn_data = ConnectionData::new("test_did".to_owned(), did_doc);
        let content = RequestContent::new("test_request_label".to_owned(), conn_data);

        let decorators = RequestDecorators::default();

        let json = json!({
            "label": content.label,
            "connection": content.connection
        });

        test_utils::test_msg::<RequestContent, _, _>(content, decorators, json);
    }

    #[test]
    fn test_extended_conn_request() {
        let did_doc = AriesDidDoc::default(); // We really need to improve this creation.
        let conn_data = ConnectionData::new("test_did".to_owned(), did_doc);
        let content = RequestContent::new("test_request_label".to_owned(), conn_data);

        let mut decorators = RequestDecorators::default();
        decorators.thread = Some(make_extended_thread());
        decorators.timing = Some(make_extended_timing());

        let json = json!({
            "label": content.label,
            "connection": content.connection,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg::<RequestContent, _, _>(content, decorators, json);
    }
}
