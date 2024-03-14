use public_key::Key;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DidValue {
    verkey: Key,
}

impl DidValue {
    pub fn new(verkey: &Key) -> Self {
        Self {
            verkey: verkey.clone(),
        }
    }

    pub fn verkey(&self) -> &Key {
        &self.verkey
    }
}
