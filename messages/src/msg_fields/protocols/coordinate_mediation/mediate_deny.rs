use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{decorators::thread::Thread, msg_parts::MsgParts};

pub type MediateDeny = MsgParts<MediateDenyContent, MediateDenyDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct MediateDenyContent {}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct MediateDenyDecorators {
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
    fn test_status_request() {
        let expected = json!(
            {
                "@id": "123456781",
                "@type": "https://didcomm.org/coordinate-mediation/1.0/mediate-deny",
            }
        );
        let content = MediateDenyContent::builder().build();
        let decorators = MediateDenyDecorators::builder().build();

        test_utils::test_msg(
            content,
            decorators,
            CoordinateMediationTypeV1_0::MediateDeny,
            expected,
        );
    }
}
