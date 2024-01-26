use public_key::Key;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DidData {
    did: String,
    verkey: Key,
}

impl DidData {
    pub fn new(did: &str, verkey: Key) -> Self {
        Self {
            did: did.into(),
            verkey,
        }
    }

    pub fn did(&self) -> &str {
        &self.did
    }

    pub fn verkey(&self) -> &Key {
        &self.verkey
    }
}
