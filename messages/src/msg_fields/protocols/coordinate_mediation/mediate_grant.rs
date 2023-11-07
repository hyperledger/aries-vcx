use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{decorators::thread::Thread, msg_parts::MsgParts};

pub type MediateGrant = MsgParts<MediateGrantContent, MediateGrantDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct MediateGrantContent {
    pub endpoint: String,
    pub routing_keys: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct MediateGrantDecorators {
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
                "@type": "https://didcomm.org/coordinate-mediation/1.0/mediate-grant",
                "endpoint": "http://mediators-r-us.com",
                "routing_keys": ["did:key:z6Mkfriq1MqLBoPWecGoDLjguo1sB9brj6wT3qZ5BxkKpuP6"]
            }
        );
        let content = MediateGrantContent::builder()
            .endpoint("http://mediators-r-us.com".to_owned())
            .routing_keys(vec![
                "did:key:z6Mkfriq1MqLBoPWecGoDLjguo1sB9brj6wT3qZ5BxkKpuP6".to_owned(),
            ])
            .build();
        let decorators = MediateGrantDecorators::builder().build();

        test_utils::test_msg(
            content,
            decorators,
            CoordinateMediationTypeV1_0::MediateGrant,
            expected,
        );
    }
}
