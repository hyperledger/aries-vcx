use serde::{Deserialize, Serialize};

use super::base58_string::Base58String;

#[derive(Debug, Deserialize, Serialize)]
pub struct KeyValue {
    pub verkey: Base58String,
    pub signkey: Base58String,
}

impl KeyValue {
    pub fn new(signkey: Base58String, verkey: Base58String) -> Self {
        Self { signkey, verkey }
    }

    pub fn signkey(&self) -> &Base58String {
        &self.signkey
    }

    pub fn verkey(&self) -> &Base58String {
        &self.verkey
    }
}
