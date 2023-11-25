use display_as_json::Display;
use serde::{Deserialize, Serialize};
// Bind `shared::misc::serde_ignored::SerdeIgnored` type as `NoDecorators`.
use typed_builder::TypedBuilder;

/// Struct representing a complete message (apart from the `@type` field) as defined in a protocol
/// RFC. The purpose of this type is to allow decomposition of certain message parts so they can be
/// independently processed, if needed.
///
/// This allows separating, for example, the protocol specific fields from the decorators
/// used in a message without decomposing the entire message into individual fields.
///
/// Note that there's no hard rule about what field goes where. There are decorators, such as
/// `~attach` used in some messages that are in fact part of the protocol itself and are
/// instrumental to the message processing, not an appendix to the message (such as `~thread` or
/// `~timing`).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder, Display)]
#[builder(build_method(vis = "", name = __build))]
pub struct MsgParts<C, D> {
    /// All standalone messages have an `id` field.
    #[serde(rename = "@id")]
    pub id: String,
    /// The protocol specific fields provided as a standalone type.
    #[serde(flatten)]
    pub content: C,
    /// The decorators this message uses, provided as a standalone type.
    #[serde(flatten)]
    pub decorators: D,
}

/// Allows building message without decorators being specified.
#[allow(dead_code, non_camel_case_types, missing_docs)]
impl<C, D> MsgPartsBuilder<C, D, ((String,), (), (D,))>
where
    C: Default,
{
    pub fn build<T>(self) -> T
    where
        MsgParts<C, D>: Into<T>,
    {
        self.content(Default::default()).__build().into()
    }
}

/// Allows building message without decorators being specified.
#[allow(dead_code, non_camel_case_types, missing_docs)]
impl<C, D> MsgPartsBuilder<C, D, ((String,), (C,), ())>
where
    D: Default,
{
    pub fn build<T>(self) -> T
    where
        MsgParts<C, D>: Into<T>,
    {
        self.decorators(Default::default()).__build().into()
    }
}

#[allow(dead_code, non_camel_case_types, missing_docs)]
impl<C, D> MsgPartsBuilder<C, D, ((String,), (C,), (D,))> {
    pub fn build<T>(self) -> T
    where
        MsgParts<C, D>: Into<T>,
    {
        self.__build().into()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
pub mod tests {
    use crate::msg_fields::protocols::did_exchange::request::{
        tests::request_content, Request, RequestDecorators,
    };

    #[test]
    fn test_print_message() {
        let msg: Request = Request::builder()
            .id("test_id".to_owned())
            .content(request_content())
            .decorators(RequestDecorators::default())
            .build();
        let printed = format!("{}", msg);
        let expected = r#"{"@id":"test_id","label":"test_request_label","goal_code":"aries.rel.build","goal":"test_goal","did":"","did_doc~attach":{"data":{"json":{"@context":"https://w3id.org/did/v1","authentication":[],"id":"","publicKey":[],"service":[{"id":"did:example:123456789abcdefghi;indy","priority":0,"recipientKeys":[],"routingKeys":[],"serviceEndpoint":"https://dummy.dummy/dummy","type":"IndyAgent"}]}}}}"#;
        assert_eq!(expected, printed);
    }
}
