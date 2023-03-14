use crate::{protocols::traits::{ConcreteMessage, HasKind}, misc::nothing::Nothing};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message<C, D = Nothing> {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(flatten)]
    pub content: C,
    #[serde(flatten)]
    pub decorators: D,
}

impl<C> Message<C> {
    pub fn new(content: C) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content,
            decorators: Nothing,
        }
    }
}

impl<C, D> Message<C, D> {
    pub fn with_decorators(content: C, decorators: D) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
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
