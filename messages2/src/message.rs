use serde::{Deserialize, Serialize};

use crate::{
    misc::nothing::Nothing,
    protocols::traits::{ConcreteMessage, HasKind},
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Message<C, D = Nothing> {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(flatten)]
    pub content: C,
    #[serde(flatten)]
    pub decorators: D,
}

impl<C> Message<C> {
    pub fn new(id: String, content: C) -> Self {
        Self {
            id,
            content,
            decorators: Nothing,
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

impl<C, D> HasKind for Message<C, D>
where
    C: ConcreteMessage,
{
    type KindType = C::Kind;

    fn kind_type() -> Self::KindType {
        C::kind()
    }
}
