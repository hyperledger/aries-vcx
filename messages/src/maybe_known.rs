/// Enum used to encapsulate `string-like` types which may
/// have variants we haven't implement yet.
///
/// Deserialization will be first attempted to the [`MaybeKnown::Known`] variant
/// and then, if that fails, to the [`MaybeKnown::Unknown`] variant.
// #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
// #[serde(untagged)]
// pub enum MaybeKnown<T> {
//     Known(T),
//     Unknown(String),
// }
// implementation moved to shared_vcx::maybe_known::MaybeKnown

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use serde::Deserialize;
    use serde_json::json;
    use shared_vcx::maybe_known::MaybeKnown;

    use super::*;
    use crate::msg_types::Protocol;

    #[test]
    fn test_maybe_known_protocol() {
        // Note that deserializing from a [`serde_json::Value`] must consume the data, while
        // deserializing from a reference from it allows borrowing.
        //
        // See: <https://github.com/serde-rs/serde/issues/1009>
        let protocol_json = json!("https://didcomm.org/connections/1.0");
        let maybe_known = MaybeKnown::<Protocol>::deserialize(&protocol_json).unwrap();

        assert!(matches!(maybe_known, MaybeKnown::Known(_)));

        let protocol_string = "https://didcomm.org/dummy_protocol/1.0".to_owned();
        let protocol_json = json!(protocol_string);
        let maybe_known: MaybeKnown<Protocol> = serde_json::from_value(protocol_json).unwrap();

        assert_eq!(maybe_known, MaybeKnown::Unknown(protocol_string));
    }
}
