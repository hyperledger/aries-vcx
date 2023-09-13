use serde::{Deserialize, Serialize};
use shared_vcx::maybe_known::MaybeKnown;
use typed_builder::TypedBuilder;

use super::ProtocolDescriptor;
use crate::{
    decorators::{thread::Thread, timing::Timing},
    msg_parts::MsgParts,
    msg_types::registry::PROTOCOL_REGISTRY,
};

pub type Disclose = MsgParts<DiscloseContent, DiscloseDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct DiscloseContent {
    pub protocols: Vec<ProtocolDescriptor>,
}

impl Default for DiscloseContent {
    fn default() -> Self {
        let mut protocols = Vec::new();

        for entries in PROTOCOL_REGISTRY.clone().into_values() {
            for entry in entries {
                let pd = ProtocolDescriptor::builder()
                    .pid(MaybeKnown::Known(entry.protocol))
                    .roles(entry.roles)
                    .build();
                protocols.push(pd);
            }
        }

        Self { protocols }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct DiscloseDecorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{thread::tests::make_extended_thread, timing::tests::make_extended_timing},
        misc::test_utils,
        msg_types::discover_features::DiscoverFeaturesTypeV1_0,
    };
    use shared_vcx::maybe_known::MaybeKnown;

    #[test]
    fn test_minimal_disclose() {
        let content = DiscloseContent::default();

        let decorators = DiscloseDecorators::builder().thread(make_extended_thread()).build();

        let expected = json!({
            "protocols": content.protocols,
            "~thread": decorators.thread
        });

        test_utils::test_msg(content, decorators, DiscoverFeaturesTypeV1_0::Disclose, expected);
    }

    #[test]
    fn test_extended_disclose() {
        let mut content = DiscloseContent::default();
        content.protocols.pop();
        content.protocols.pop();
        content.protocols.pop();

        let dummy_protocol_descriptor = ProtocolDescriptor::builder()
            .pid(MaybeKnown::Unknown("test_dummy_pid".to_owned()))
            .build();
        content.protocols.push(dummy_protocol_descriptor);

        let decorators = DiscloseDecorators::builder()
            .thread(make_extended_thread())
            .timing(make_extended_timing())
            .build();

        let expected = json!({
            "protocols": content.protocols,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(content, decorators, DiscoverFeaturesTypeV1_0::Disclose, expected);
    }
}
