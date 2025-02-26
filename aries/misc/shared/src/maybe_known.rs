use serde::{Deserialize, Serialize};

/// Enum used to encapsulate values of any type that may have variants we haven't implemented yet.
///
/// Deserialization will be first attempted to the [`MaybeKnown::Known`] variant
/// and then, if that fails, to the [`MaybeKnown::Unknown`] variant.
///
/// This enum provides a flexible way to handle values that can be fully understood or may have
/// unknown variants. By default, the `MaybeKnown` enum is designed to work with `string-like`
/// types, represented by the type `U` as `String`. However, you can use any type `U` that suits
/// your requirements.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum MaybeKnown<T, U = String> {
    Known(T),
    Unknown(U),
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use serde_json::json;

    use super::*;

    #[derive(Serialize, Deserialize)]
    enum TestEnum {
        First,
        Second,
    }

    #[test]
    fn test_maybe_known_enum_known() {
        let json = json!("First");
        let maybe_known = MaybeKnown::<TestEnum>::deserialize(&json).unwrap();
        assert!(matches!(maybe_known, MaybeKnown::Known(_)));

        let json = json!("Second");
        let maybe_known = MaybeKnown::<TestEnum>::deserialize(&json).unwrap();
        assert!(matches!(maybe_known, MaybeKnown::Known(_)));
    }

    #[test]
    fn test_maybe_known_enum_unknown() {
        let json = json!("Some Random Value");

        let maybe_known = MaybeKnown::<TestEnum, String>::deserialize(&json).unwrap();
        assert!(matches!(maybe_known, MaybeKnown::Unknown(_)));
    }
}
