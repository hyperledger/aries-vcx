use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{decorators::thread::Thread, msg_parts::MsgParts};

/// https://github.com/hyperledger/aries-rfcs/blob/main/features/0211-route-coordination/README.md#keylist-update
pub type KeylistUpdate = MsgParts<KeylistUpdateContent>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct KeylistUpdateContent {
    pub updates: Vec<KeylistUpdateItem>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct KeylistUpdateItem {
    pub recipient_key: String,
    pub action: KeylistUpdateItemAction,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum KeylistUpdateItemAction {
    #[serde(rename = "add")]
    Add,
    #[serde(rename = "remove")]
    Remove,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct KeylistUpdateDecorators {
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;
    use shared::misc::serde_ignored::SerdeIgnored as NoDecorators;

    use super::*;
    use crate::{
        misc::test_utils, msg_types::protocols::coordinate_mediation::CoordinateMediationTypeV1_0,
    };

    #[test]
    fn test_key_list_update() {
        let expected = json!(
            {
                "@id": "123456781",
                "@type": "https://didcomm.org/coordinate-mediation/1.0/keylist-update",
                "updates":[
                    {
                        "recipient_key": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
                        "action": "add"
                    }
                ]
            }
        );
        let update_item1 = KeylistUpdateItem::builder()
            .recipient_key("did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH".to_owned())
            .action(KeylistUpdateItemAction::Add)
            .build();
        let content = KeylistUpdateContent::builder()
            .updates(vec![update_item1])
            .build();
        test_utils::test_msg(
            content,
            NoDecorators,
            CoordinateMediationTypeV1_0::KeylistUpdate,
            expected,
        );
    }
}
