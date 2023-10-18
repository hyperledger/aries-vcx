use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::decorators::thread::Thread;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct PickupDecoratorsCommon {
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~transport")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<Transport>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct Transport {
    pub return_route: ReturnRoute,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_route_thread: Option<Thread>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
pub enum ReturnRoute {
    #[default]
    #[serde(rename = "none")]
    None,
    #[serde(rename = "all")]
    All,
    #[serde(rename = "thread")]
    Thread,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    #[test]
    fn test_transport_decorator() {
        let transport = Transport::builder().return_route(ReturnRoute::All).build();
        let decorators = PickupDecoratorsCommon::builder()
            .transport(transport)
            .build();
        let expected = json!({
            "~transport": {
                "return_route": "all"
            }
        });
        test_utils::test_serde(decorators, expected);
    }
}
