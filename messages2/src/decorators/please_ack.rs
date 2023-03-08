use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PleaseAck {
    pub on: Vec<AckOn>,
}

impl PleaseAck {
    pub fn new(on: Vec<AckOn>) -> Self {
        Self { on }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AckOn {
    Receipt,
    Outcome,
}
