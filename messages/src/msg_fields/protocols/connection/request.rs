use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

use super::ConnectionData;

pub type Request = MsgParts<RequestContent, RequestDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct RequestContent {
    pub label: String,
    pub connection: ConnectionData,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct RequestDecorators {
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use diddoc_legacy::aries::diddoc::AriesDidDoc;
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{thread::tests::make_extended_thread, timing::tests::make_extended_timing},
        misc::test_utils,
        msg_types::connection::ConnectionTypeV1_0,
    };

    #[test]
    fn test_minimal_conn_request() {
        let did_doc = AriesDidDoc::default(); // We really need to improve this creation.
        let conn_data = ConnectionData::new("test_did".to_owned(), did_doc);
        let content = RequestContent::builder()
            .label("test_request_label".to_owned())
            .connection(conn_data)
            .build();

        let decorators = RequestDecorators::default();

        let expected = json!({
            "label": content.label,
            "connection": content.connection
        });

        test_utils::test_msg(content, decorators, ConnectionTypeV1_0::Request, expected);
    }

    #[test]
    fn test_extended_conn_request() {
        let did_doc = AriesDidDoc::default(); // We really need to improve this creation.
        let conn_data = ConnectionData::new("test_did".to_owned(), did_doc);
        let content = RequestContent::builder()
            .label("test_request_label".to_owned())
            .connection(conn_data)
            .build();

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
