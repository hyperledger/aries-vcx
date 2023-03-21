use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use super::ProtocolDescriptor;
use crate::{
    decorators::{thread::Thread, timing::Timing},
    maybe_known::MaybeKnown,
    message::Message,
    msg_types::{registry::PROTOCOL_REGISTRY, types::discover_features::DiscoverFeaturesV1_0Kind},
};

pub type Disclose = Message<DiscloseContent, DiscloseDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "DiscoverFeaturesV1_0Kind::Disclose")]
pub struct DiscloseContent {
    pub protocols: Vec<ProtocolDescriptor>,
}

impl DiscloseContent {
    pub fn new() -> Self {
        let mut protocols = Vec::new();

        for entries in PROTOCOL_REGISTRY.clone().into_values() {
            for entry in entries {
                let pid = MaybeKnown::Known(entry.protocol);
                let mut pd = ProtocolDescriptor::new(pid);
                pd.roles = Some(entry.actors);
                protocols.push(pd);
            }
        }

        Self { protocols }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
pub struct DiscloseDecorators {
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
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
        maybe_known::MaybeKnown,
        misc::test_utils,
    };

    #[test]
    fn test_minimal_disclose() {
        let content = DiscloseContent::new();

        let decorators = DiscloseDecorators::default();

        let json = json!({
            "protocols": content.protocols
        });

        test_utils::test_msg::<DiscloseContent, _, _>(content, decorators, json);
    }

    #[test]
    fn test_extensive_disclose() {
        let mut content = DiscloseContent::new();
        content.protocols.pop();
        content.protocols.pop();
        content.protocols.pop();

        let dummy_protocol_descriptor = ProtocolDescriptor::new(MaybeKnown::Unknown("test_dummy_pid".to_owned()));
        content.protocols.push(dummy_protocol_descriptor);

        let mut decorators = DiscloseDecorators::default();
        decorators.thread = Some(make_extended_thread());
        decorators.timing = Some(make_extended_timing());

        let json = json!({
            "protocols": content.protocols,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg::<DiscloseContent, _, _>(content, decorators, json);
    }
}
