use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{decorators::thread::Thread, msg_parts::MsgParts};

/// https://github.com/hyperledger/aries-rfcs/blob/main/features/0211-route-coordination/README.md#key-list
pub type Keylist = MsgParts<KeylistContent, KeylistDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct KeylistContent {
    pub keys: Vec<KeylistItem>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pagination: Option<KeylistPagination>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct KeylistItem {
    pub recipient_key: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct KeylistPagination {
    count: u64,
    offset: u64,
    remaining: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct KeylistDecorators {
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
    fn test_keylist() {
        let expected = json!(
            {
                "@id": "123456781",
                "@type": "https://didcomm.org/coordinate-mediation/1.0/keylist",
                "keys": [
                    {
                        "recipient_key": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH"
                    }
                ],
                "pagination": {
                    "count": 30,
                    "offset": 30,
                    "remaining": 100
                }
            }
        );
        let key_item = KeylistItem::builder()
            .recipient_key("did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH".to_owned())
            .build();
        let pagination_state = KeylistPagination::builder()
            .count(30)
            .offset(30)
            .remaining(100)
            .build();
        let content = KeylistContent::builder()
            .pagination(pagination_state)
            .keys(vec![key_item])
            .build();
        let decorators = KeylistDecorators::builder().build();

        test_utils::test_msg(
            content,
            decorators,
            CoordinateMediationTypeV1_0::Keylist,
            expected,
        );
    }
}
