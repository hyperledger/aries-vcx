use serde::{de::IgnoredAny, Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NoDecorators;

/// Custom impl that, through [`Option`], handles the field not being
/// provided at all and, through [`IgnoredAny`], also ignores anything
/// that was provided for the field.
impl<'de> Deserialize<'de> for NoDecorators {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
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
impl Serialize for NoDecorators {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_none()
    }
}
