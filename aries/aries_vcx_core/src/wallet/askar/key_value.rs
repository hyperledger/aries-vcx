use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct KeyValue {
    verkey: String,
    signkey: String,
}

impl KeyValue {
    pub fn new(signkey: String, verkey: String) -> Self {
        Self { signkey, verkey }
    }
}
