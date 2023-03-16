use serde::{de::IgnoredAny, Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Nothing;

/// Custom impl that, through [`Option`], handles the field not being
/// provided at all and, through [`IgnoredAny`], also ignores anything
/// that was provided for the field.
impl<'de> Deserialize<'de> for Nothing {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {   println!("before deserialize");
        Option::<IgnoredAny>::deserialize(deserializer)?;
        println!("after deserialize");
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
        S: serde::Serializer,
    {
        s.serialize_none()
    }
}
