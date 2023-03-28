use serde::{Deserialize, Serialize};

/// Struct representing the `~please_ack` decorators from its [RFC](<https://github.com/hyperledger/aries-rfcs/blob/main/features/0317-please-ack/README.md>).
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct PleaseAck {
    pub on: Vec<AckOn>,
}

impl PleaseAck {
    pub fn new(on: Vec<AckOn>) -> Self {
        Self { on }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AckOn {
    Receipt,
    Outcome,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
pub mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    pub fn make_minimal_please_ack() -> PleaseAck {
        let on = vec![AckOn::Receipt, AckOn::Outcome];
        PleaseAck::new(on)
    }

    #[test]
    fn test_minimal_please_ack() {
        let please_ack = make_minimal_please_ack();
        let expected = json!({ "on": please_ack.on });

        test_utils::test_serde(please_ack, expected);
    }
}
