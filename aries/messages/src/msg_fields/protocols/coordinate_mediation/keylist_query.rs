use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{decorators::thread::Thread, misc::NoDecorators, msg_parts::MsgParts};

/// https://github.com/hyperledger/aries-rfcs/blob/main/features/0211-route-coordination/README.md#key-list-query
pub type KeylistQuery = MsgParts<KeylistQueryContent, NoDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct KeylistQueryContent {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    paginate: Option<KeylistQueryPaginateParams>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct KeylistQueryPaginateParams {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u64>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct KeylistQueryDecorators {
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
    fn test_keylist_query() {
        let expected = json!(
            {
                "@id": "123456781",
                "@type": "https://didcomm.org/coordinate-mediation/1.0/keylist-query",
                "paginate": {
                    "limit": 30,
                    "offset": 0
                }
            }
        );
        let paginate_params = KeylistQueryPaginateParams::builder()
            .limit(30)
            .offset(0)
            .build();
        let content = KeylistQueryContent::builder()
            .paginate(paginate_params)
            .build();

        test_utils::test_msg(
            content,
            NoDecorators,
            CoordinateMediationTypeV1_0::KeylistQuery,
            expected,
        );
    }
}
