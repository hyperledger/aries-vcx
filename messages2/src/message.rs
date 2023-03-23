use serde::{Deserialize, Serialize};

use crate::{
    misc::no_decorators::NoDecorators,
    protocols::traits::{MessageContent, MessageWithKind},
};

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
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Message<C, D = NoDecorators> {
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

impl<C> Message<C> {
    pub fn new(id: String, content: C) -> Self {
        Self {
            id,
            content,
            decorators: NoDecorators,
        }
    }
}

impl<C, D> Message<C, D> {
    pub fn with_decorators(id: String, content: C, decorators: D) -> Self {
        Self {
            id,
            content,
            decorators,
        }
    }
}

/// Blanket impl, allowing all [`Message`] types where the content implements
/// [`MessageContent`] to access the [`MessageContent::Kind`].
impl<C, D> MessageWithKind for Message<C, D>
where
    C: MessageContent,
{
    type MsgKind = C::Kind;

    fn msg_kind() -> Self::MsgKind {
        C::kind()
    }
}
