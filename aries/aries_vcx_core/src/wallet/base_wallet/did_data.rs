use public_key::Key;
use serde::{Deserialize, Serialize};

use crate::wallet::utils::did_from_key;

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

    pub fn did_from_verkey(&self) -> String {
        did_from_key(self.verkey.clone())
    }
}
