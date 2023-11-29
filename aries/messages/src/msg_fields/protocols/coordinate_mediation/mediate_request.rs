use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{decorators::thread::Thread, msg_parts::MsgParts};

/// https://github.com/hyperledger/aries-rfcs/blob/main/features/0211-route-coordination/README.md#mediation-request
pub type MediateRequest = MsgParts<MediateRequestContent>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct MediateRequestContent {}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct MediateRequestDecorators {
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
    fn test_mediate_request() {
        let expected = json!(
            {
                "@id": "123456781",
                "@type": "https://didcomm.org/coordinate-mediation/1.0/mediate-request",
            }
        );
        let content = MediateRequestContent::builder().build();
        test_utils::test_msg(
            content,
            NoDecorators,
            CoordinateMediationTypeV1_0::MediateRequest,
            expected,
        );
    }
}
