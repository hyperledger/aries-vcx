use serde::{Deserialize, Serialize};

use super::EmptyDecorator;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct PleaseAck {
    on: Vec<AckOn>,
}

impl EmptyDecorator for PleaseAck {
    fn is_empty(&self) -> bool {
        self.on.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AckOn {
    Receipt,
    Outcome,
}
