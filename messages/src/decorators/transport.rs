use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::decorators::thread::Thread;

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
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    #[test]
    fn test_transport_minimals() {
        // all variant
        let transport = Transport::builder().return_route(ReturnRoute::All).build();
        let expected = json!({
                "return_route": "all"
        });
        test_utils::test_serde(transport, expected);
        // none variant
        let transport = Transport::builder().return_route(ReturnRoute::None).build();
        let expected = json!({
                "return_route": "none"
        });
        test_utils::test_serde(transport, expected);
    }
    #[test]
    fn test_transport_extended() {
        // thread variant
        let thread = Thread::builder().thid("<thread id>".to_string()).build();
        let transport = Transport::builder()
            .return_route(ReturnRoute::Thread)
            .return_route_thread(thread)
            .build();
        let expected = json!({
                "return_route": "thread",
                "return_route_thread": { "thid": "<thread id>" }
        });
        test_utils::test_serde(transport, expected);
    }
}
