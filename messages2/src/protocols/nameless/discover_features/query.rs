use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use super::ProtocolDescriptor;
use crate::{
    decorators::timing::Timing,
    maybe_known::MaybeKnown,
    message::Message,
    msg_types::{registry::PROTOCOL_REGISTRY, types::discover_features::DiscoverFeaturesV1_0},
};

pub type Query = Message<QueryContent, QueryDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "DiscoverFeaturesV1_0::Query")]
pub struct QueryContent {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl QueryContent {
    pub fn new(query: String) -> Self {
        Self { query, comment: None }
    }

    /// Looks up into the [`PROTOCOL_REGISTRY`] and returns a [`Vec<ProtocolDescriptor`] matching the inner query.
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
                    let pid = MaybeKnown::Known(entry.protocol);
                    let mut pd = ProtocolDescriptor::new(pid);
                    pd.roles = Some(entry.roles.clone());
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
    use crate::{
        decorators::timing::tests::make_extended_timing,
        misc::test_utils,
        msg_types::{traits::ProtocolVersion, types::connection::ConnectionV1},
    };

    #[test]
    fn test_minimal_query() {
        let content = QueryContent::new("*".to_owned());

        let decorators = QueryDecorators::default();

        let expected = json!({
            "query": content.query
        });

        test_utils::test_msg::<QueryContent, _, _>(content, decorators, expected);
    }

    #[test]
    fn test_extended_query() {
        let mut content = QueryContent::new("*".to_owned());
        content.comment = Some("test_comment".to_owned());

        let mut decorators = QueryDecorators::default();
        decorators.timing = Some(make_extended_timing());

        let expected = json!({
            "query": content.query,
            "comment": content.comment,
            "~timing": decorators.timing
        });

        test_utils::test_msg::<QueryContent, _, _>(content, decorators, expected);
    }

    #[test]
    fn test_lookup_match_all() {
        let matched_all = QueryContent::new("*".to_owned()).lookup();

        let mut protocols = Vec::new();

        for entries in PROTOCOL_REGISTRY.values() {
            for entry in entries {
                let pid = MaybeKnown::Known(entry.protocol);
                let mut pd = ProtocolDescriptor::new(pid);
                pd.roles = Some(entry.roles.clone());
                protocols.push(pd);
            }
        }

        assert_eq!(protocols, matched_all);
    }

    #[test]
    fn test_lookup_match_protocol() {
        let matched_protocol = QueryContent::new("https://didcomm.org/connections/*".to_owned()).lookup();

        let pid = ConnectionV1::new_v1_0();
        let roles = pid.roles();
        let mut pd = ProtocolDescriptor::new(MaybeKnown::Known(pid.into()));
        pd.roles = Some(roles);

        let protocols = vec![pd];

        assert_eq!(protocols, matched_protocol);
    }

    #[test]
    fn test_lookup_match_version() {
        let matched_protocol = QueryContent::new("https://didcomm.org/connections/1.*".to_owned()).lookup();

        let pid = ConnectionV1::new_v1_0();
        let roles = pid.roles();
        let mut pd = ProtocolDescriptor::new(MaybeKnown::Known(pid.into()));
        pd.roles = Some(roles);

        let protocols = vec![pd];

        assert_eq!(protocols, matched_protocol);
    }

    #[test]
    fn test_lookup_match_none() {
        let matched_protocol = QueryContent::new("https://didcomm.org/non-existent/*".to_owned()).lookup();
        let protocols = Vec::<ProtocolDescriptor>::new();

        assert_eq!(protocols, matched_protocol);
    }
}
