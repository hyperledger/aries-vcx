use serde::{Deserialize, Serialize};
use shared_vcx::maybe_known::MaybeKnown;
use typed_builder::TypedBuilder;

use super::ProtocolDescriptor;
use crate::{
    decorators::timing::Timing, msg_parts::MsgParts, msg_types::registry::PROTOCOL_REGISTRY,
};

pub type Query = MsgParts<QueryContent, QueryDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct QueryContent {
    pub query: String,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl QueryContent {
    /// Looks up into the [`PROTOCOL_REGISTRY`] and returns a [`Vec<ProtocolDescriptor`] matching
    /// the inner query.
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
                    let pd = ProtocolDescriptor::builder()
                        .pid(MaybeKnown::Known(entry.protocol))
                        .roles(entry.roles.clone())
                        .build();
                    protocols.push(pd);
                }
            }
        }

        protocols
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct QueryDecorators {
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::timing::tests::make_extended_timing,
        misc::test_utils,
        msg_types::{
            discover_features::DiscoverFeaturesTypeV1_0, protocols::connection::ConnectionTypeV1,
            traits::ProtocolVersion,
        },
    };

    #[test]
    fn test_minimal_query() {
        let content = QueryContent::builder().query("*".to_owned()).build();

        let decorators = QueryDecorators::default();

        let expected = json!({
            "query": content.query
        });

        test_utils::test_msg(
            content,
            decorators,
            DiscoverFeaturesTypeV1_0::Query,
            expected,
        );
    }

    #[test]
    fn test_extended_query() {
        let content = QueryContent::builder()
            .query("*".to_owned())
            .comment("test_comment".to_owned())
            .build();

        let decorators = QueryDecorators::builder()
            .timing(make_extended_timing())
            .build();

        let expected = json!({
            "query": content.query,
            "comment": content.comment,
            "~timing": decorators.timing
        });

        test_utils::test_msg(
            content,
            decorators,
            DiscoverFeaturesTypeV1_0::Query,
            expected,
        );
    }

    #[test]
    fn test_lookup_match_all() {
        let matched_all = QueryContent::builder()
            .query("*".to_owned())
            .build()
            .lookup();

        let mut protocols = Vec::new();

        for entries in PROTOCOL_REGISTRY.values() {
            for entry in entries {
                let pid = MaybeKnown::Known(entry.protocol);
                let pd = ProtocolDescriptor::builder()
                    .pid(pid)
                    .roles(entry.roles.clone())
                    .build();
                protocols.push(pd);
            }
        }

        assert_eq!(protocols, matched_all);
    }

    #[test]
    fn test_lookup_match_protocol() {
        let matched_protocol = QueryContent::builder()
            .query("https://didcomm.org/connections/*".to_owned())
            .build()
            .lookup();

        let pid = ConnectionTypeV1::new_v1_0();
        let roles = pid.roles();
        let pd = ProtocolDescriptor::builder()
            .pid(MaybeKnown::Known(pid.into()))
            .roles(roles)
            .build();

        let protocols = vec![pd];

        assert_eq!(protocols, matched_protocol);
    }

    #[test]
    fn test_lookup_match_version() {
        let matched_protocol = QueryContent::builder()
            .query("https://didcomm.org/connections/1.*".to_owned())
            .build()
            .lookup();

        let pid = ConnectionTypeV1::new_v1_0();
        let pd = ProtocolDescriptor::builder()
            .pid(MaybeKnown::Known(pid.into()))
            .roles(pid.roles())
            .build();

        let protocols = vec![pd];

        assert_eq!(protocols, matched_protocol);
    }

    #[test]
    fn test_lookup_match_none() {
        let matched_protocol = QueryContent::builder()
            .query("https://didcomm.org/non-existent/*".to_owned())
            .build()
            .lookup();
        let protocols = Vec::<ProtocolDescriptor>::new();

        assert_eq!(protocols, matched_protocol);
    }
}
