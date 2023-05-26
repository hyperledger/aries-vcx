use serde::{Deserialize, Serialize};

use crate::{
    decorators::{thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

use super::ConnectionData;

pub type Request = MsgParts<RequestContent, RequestDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RequestContent {
    pub label: String,
    pub connection: ConnectionData,
}

impl RequestContent {
    pub fn new(label: String, connection: ConnectionData) -> Self {
        Self { label, connection }
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
    use did_doc::schema::did_doc::DidDocument;
    use did_resolver_sov::resolution::ExtraFieldsSov;
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{thread::tests::make_extended_thread, timing::tests::make_extended_timing},
        misc::test_utils,
        msg_types::connection::ConnectionTypeV1_0,
    };

    #[test]
    fn test_minimal_conn_request() {
        let did_doc = DidDocument::<ExtraFieldsSov>::builder(Default::default()).build();
        let conn_data = ConnectionData::new("test_did".to_owned(), did_doc);
        let content = RequestContent::new("test_request_label".to_owned(), conn_data);

        let decorators = RequestDecorators::default();

        let expected = json!({
            "label": content.label,
            "connection": content.connection
        });

        test_utils::test_msg(content, decorators, ConnectionTypeV1_0::Request, expected);
    }

    #[test]
    fn test_extended_conn_request() {
        let did_doc = DidDocument::<ExtraFieldsSov>::builder(Default::default()).build();
        let conn_data = ConnectionData::new("test_did".to_owned(), did_doc);
        let content = RequestContent::new("test_request_label".to_owned(), conn_data);

        let mut decorators = RequestDecorators::default();
        decorators.thread = Some(make_extended_thread());
        decorators.timing = Some(make_extended_timing());

        let expected = json!({
            "label": content.label,
            "connection": content.connection,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(content, decorators, ConnectionTypeV1_0::Request, expected);
    }
}
