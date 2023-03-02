use serde::{de::IgnoredAny, Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

use crate::protocols::traits::MessageKind;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message<C, MD = Nothing> {
    #[serde(rename = "@id")]
    id: String,
    #[serde(flatten)]
    content: C,
    #[serde(flatten)]
    decorators: MD,
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

impl<C, MD> Message<C, MD> {
    pub fn with_decorators(content: C, decorators: MD) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content,
            decorators,
        }
    }
}

impl<C, MD> Message<C, MD>
where
    C: MessageKind,
{
    pub fn kind() -> C::Kind {
        C::kind()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Nothing;

/// Custom impl that, through [`Option`], handles the field not being
/// provided at all and, through [`IgnoredAny`], also ignores anything
/// that was provided for the field.
impl<'de> Deserialize<'de> for Nothing {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<IgnoredAny>::deserialize(deserializer)?;
        Ok(Self)
    }
}

/// Custom impl that always serializes to nothing, or `null`.
///
/// The really cool thing, though, is that flattening this actually
/// results in completely nothing, making a field of this type
/// to be completely ignored.
impl Serialize for Nothing {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_none()
    }
}
