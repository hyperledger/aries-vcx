use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// Struct representing the `~please_ack` decorators from its [RFC](<https://github.com/hyperledger/aries-rfcs/blob/main/features/0317-please-ack/README.md>).
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, TypedBuilder)]
pub struct PleaseAck {
    // This is wrong, but necessary for backwards compatibility.
    // Per the [RFC](<https://github.com/hyperledger/aries-rfcs/blob/main/features/0317-please-ack/README.md#on>)
    // this is a *required* array.
    //
    // However, the entire field was previously NOT serialized if it was empty,
    // resulting in something like '"~please_ack": null' instead of '"~please_ack": {"on": []}'.
    //
    // One could argue that the field could be treated even better, so that an empty array would be
    // incorrect. Perhaps using an enum altogether (if the values become somewhat stable) or
    // ordering the array might be of interest, so that processing happens in a specific order.
    #[serde(default)]
    pub on: Vec<AckOn>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AckOn {
    Receipt,
    Outcome,
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
pub mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    pub fn make_minimal_please_ack() -> PleaseAck {
        let on = vec![AckOn::Receipt, AckOn::Outcome];
        PleaseAck::builder().on(on).build()
    }

    #[test]
    fn test_minimal_please_ack() {
        let please_ack = make_minimal_please_ack();
        let expected = json!({ "on": please_ack.on });

        test_utils::test_serde(please_ack, expected);
    }
}
