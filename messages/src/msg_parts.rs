use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

// Bind `shared_vcx::misc::serde_ignored::SerdeIgnored` type as `NoDecorators`.
use shared_vcx::misc::serde_ignored::SerdeIgnored as NoDecorators;

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
pub struct MsgParts<C, D = NoDecorators> {
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

impl<C, D> MsgParts<C, D> {
    /// Create a builder for building `MsgParts`.
    /// On the builder, call `.id(...)`, `.content(...)`, `.decorators(...)` to set the values of the fields.
    /// Finally, call `.build()` to create the instance of `MsgParts`.
    pub fn builder() -> MsgPartsBuilder<C, D, ((), (), ())> {
        MsgPartsBuilder {
            fields: ((), (), ()),
            phantom: PhantomData,
        }
    }
}

#[must_use]
#[doc(hidden)]
/// A builder not unlike the ones derived through [`typed_builder::TypedBuilder`], with the caveat
/// that this one supports the decorators not being set, in which case it resorts to [`NoDecorators`].
pub struct MsgPartsBuilder<C, D = NoDecorators, TypedBuilderFields = ((), (), ())> {
    fields: TypedBuilderFields,
    phantom: PhantomData<(C, D)>,
}

impl<C, D, TypedBuilderFields> Clone for MsgPartsBuilder<C, D, TypedBuilderFields>
where
    TypedBuilderFields: Clone,
{
    fn clone(&self) -> Self {
        Self {
            fields: self.fields.clone(),
            phantom: PhantomData,
        }
    }
}

#[allow(dead_code, non_camel_case_types, missing_docs)]
impl<C, D, __content, __decorators> MsgPartsBuilder<C, D, ((), __content, __decorators)> {
    pub fn id(self, id: String) -> MsgPartsBuilder<C, D, ((String,), __content, __decorators)> {
        let id = (id,);
        let ((), content, decorators) = self.fields;
        MsgPartsBuilder {
            fields: (id, content, decorators),
            phantom: self.phantom,
        }
    }
}

#[allow(dead_code, non_camel_case_types, missing_docs)]
impl<C, D, __id, __decorators> MsgPartsBuilder<C, D, (__id, (), __decorators)> {
    pub fn content(self, content: C) -> MsgPartsBuilder<C, D, (__id, (C,), __decorators)> {
        let content = (content,);
        let (id, (), decorators) = self.fields;

        MsgPartsBuilder {
            fields: (id, content, decorators),
            phantom: self.phantom,
        }
    }
}

#[allow(dead_code, non_camel_case_types, missing_docs)]
impl<C, D, __id, __content> MsgPartsBuilder<C, D, (__id, __content, ())> {
    pub fn decorators<D2>(self, decorators: D2) -> MsgPartsBuilder<C, D2, (__id, __content, (D2,))> {
        let decorators = (decorators,);
        let (id, content, _) = self.fields;
        MsgPartsBuilder {
            fields: (id, content, decorators),
            phantom: PhantomData,
        }
    }
}

#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, non_snake_case)]
pub enum MsgPartsBuilderInvalid {}

#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, missing_docs)]
impl<C, D, __content, __decorators> MsgPartsBuilder<C, D, ((String,), __content, __decorators)> {
    #[deprecated(note = "Repeated field id")]
    pub fn id(self, _: MsgPartsBuilderInvalid) -> MsgPartsBuilder<C, D, ((String,), __content, __decorators)> {
        self
    }
}

#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, missing_docs)]
impl<C, D, __id, __decorators> MsgPartsBuilder<C, D, (__id, (C,), __decorators)> {
    #[deprecated(note = "Repeated field content")]
    pub fn content(self, _: MsgPartsBuilderInvalid) -> MsgPartsBuilder<C, D, (__id, (C,), __decorators)> {
        self
    }
}

#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, missing_docs)]
impl<C, D, __id, __content> MsgPartsBuilder<C, D, (__id, __content, (D,))> {
    #[deprecated(note = "Repeated field decorators")]
    pub fn decorators(self, _: MsgPartsBuilderInvalid) -> MsgPartsBuilder<C, D, (__id, __content, (D,))> {
        self
    }
}

#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, missing_docs, clippy::panic)]
impl<C, D, __content, __decorators> MsgPartsBuilder<C, D, ((), __content, __decorators)> {
    #[deprecated(note = "Missing required field id")]
    pub fn build(self, _: MsgPartsBuilderInvalid) -> ! {
        panic!()
    }
}

#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, missing_docs, clippy::panic)]
impl<C, D, __decorators> MsgPartsBuilder<C, D, ((String,), (), __decorators)> {
    #[deprecated(note = "Missing required field content")]
    pub fn build(self, _: MsgPartsBuilderInvalid) -> ! {
        panic!()
    }
}

#[allow(dead_code, non_camel_case_types, missing_docs)]
impl<C> MsgPartsBuilder<C, NoDecorators, ((String,), (C,), ())> {
    pub fn build(self) -> MsgParts<C> {
        let (id, content, _) = self.fields;
        let id = id.0;
        let content = content.0;

        MsgParts {
            id,
            content,
            decorators: NoDecorators,
        }
    }
}
#[allow(dead_code, non_camel_case_types, missing_docs)]
impl<C, D> MsgPartsBuilder<C, D, ((String,), (C,), (D,))> {
    pub fn build(self) -> MsgParts<C, D> {
        let (id, content, decorators) = self.fields;
        let id = id.0;
        let content = content.0;
        let decorators = decorators.0;

        MsgParts {
            id,
            content,
            decorators,
        }
    }
}
