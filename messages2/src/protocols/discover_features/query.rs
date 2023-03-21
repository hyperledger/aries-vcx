use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use super::ProtocolDescriptor;
use crate::{
    decorators::timing::Timing,
    message::Message,
    msg_types::{registry::PROTOCOL_REGISTRY, types::discover_features::DiscoverFeaturesV1_0Kind},
};

pub type Query = Message<QueryContent, QueryDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "DiscoverFeaturesV1_0Kind::Query")]
pub struct QueryContent {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl QueryContent {
    pub fn new(query: String) -> Self {
        Self { query, comment: None }
    }

    pub fn lookup(&self) -> Vec<ProtocolDescriptor> {
        let mut protocols = Vec::new();
        let query = self
            .query
            .split('*')
            .next()
            .expect("query must have at least an empty string before *");

        for entries in PROTOCOL_REGISTRY.values() {
            for entry in entries {
                if entry.str_pid.starts_with(query) {
                    let mut pd = ProtocolDescriptor::new(entry.protocol.into());
                    pd.roles = Some(entry.actors.clone());
                    protocols.push(pd);
                }
            }
        }

        protocols
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
pub struct QueryDecorators {
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
    use crate::{decorators::timing::tests::make_extended_timing, misc::test_utils};

    #[test]
    fn test_minimal_query() {
        let content = QueryContent::new("*".to_owned());

        let decorators = QueryDecorators::default();

        let json = json!({
            "query": content.query
        });

        test_utils::test_msg::<QueryContent, _, _>(content, decorators, json);
    }

    #[test]
    fn test_extensive_query() {
        let mut content = QueryContent::new("*".to_owned());
        content.comment = Some("test_comment".to_owned());

        let mut decorators = QueryDecorators::default();
        decorators.timing = Some(make_extended_timing());

        let json = json!({
            "query": content.query,
            "comment": content.comment,
            "~timing": decorators.timing
        });

        test_utils::test_msg::<QueryContent, _, _>(content, decorators, json);
    }
}
