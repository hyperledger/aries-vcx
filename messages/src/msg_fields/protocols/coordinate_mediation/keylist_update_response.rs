use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::keylist_update::KeylistUpdateItemAction;
use crate::{decorators::thread::Thread, msg_parts::MsgParts};

/// https://github.com/hyperledger/aries-rfcs/blob/main/features/0211-route-coordination/README.md#keylist-update-response
pub type KeylistUpdateResponse =
    MsgParts<KeylistUpdateResponseContent, KeylistUpdateResponseDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct KeylistUpdateResponseContent {
    pub updated: Vec<KeylistUpdateResponseItem>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct KeylistUpdateResponseItem {
    pub recipient_key: String,
    pub action: KeylistUpdateItemAction,
    pub result: KeylistUpdateItemResult,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum KeylistUpdateItemResult {
    #[serde(rename = "client_error")]
    ClientError,
    #[serde(rename = "server_error")]
    ServerError,
    #[serde(rename = "no_change")]
    NoChange,
    #[serde(rename = "success")]
    Success,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct KeylistUpdateResponseDecorators {
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        misc::test_utils, msg_types::protocols::coordinate_mediation::CoordinateMediationTypeV1_0,
    };

    #[test]
    fn test_keylist_update_response() {
        let expected = json!(
            {
                "@id": "123456781",
                "@type": "https://didcomm.org/coordinate-mediation/1.0/keylist-update-response",
                "updated": [
                    {
                        "recipient_key": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
                        "action": "add", // "add" or "remove"
                        "result": "client_error" // [client_error | server_error | no_change | success]
                    }
                ]
            }
        );
        let update_item1 = KeylistUpdateResponseItem::builder()
            .recipient_key("did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH".to_owned())
            .action(KeylistUpdateItemAction::Add)
            .result(KeylistUpdateItemResult::ClientError)
            .build();
        let content = KeylistUpdateResponseContent::builder()
            .updated(vec![update_item1])
            .build();
        let decorators = KeylistUpdateResponseDecorators::builder().build();

        test_utils::test_msg(
            content,
            decorators,
            CoordinateMediationTypeV1_0::KeylistUpdateResponse,
            expected,
        );
    }
}
