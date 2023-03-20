use serde::{Deserialize, Serialize};

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
mod tests {
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
        let json = json!({ "on": please_ack.on });

        test_utils::test_serde(please_ack, json);
    }
}
