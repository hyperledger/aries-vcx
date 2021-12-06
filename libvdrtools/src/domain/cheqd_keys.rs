extern crate zeroize;

use self::zeroize::Zeroize;
use cosmrs::bip32::PrivateKeyBytes;

#[derive(Derivative)]
#[derivative(Debug)]
#[derive(Serialize, Deserialize, Clone)]
pub struct Key {
    pub alias: String,
    // SEC1-encoded secp256k1 ECDSA priv key
    #[cfg(not(test))]
    #[derivative(Debug = "ignore")]
    pub priv_key: PrivateKeyBytes,
    #[cfg(test)]
    pub priv_key: PrivateKeyBytes,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub mnemonic: Option<String>,
}

impl Key {
    pub fn new(alias: String, priv_key: PrivateKeyBytes, mnemonic: Option<String>) -> Self {
        Key {
            alias,
            priv_key,
            mnemonic
        }
    }

    pub fn without_mnemonic(self) -> Self {
        Self {
            alias: self.alias.clone(),
            priv_key: self.priv_key.clone(),
            mnemonic: None
        }
    }
}


impl Zeroize for Key {
    fn zeroize(&mut self) {
        self.priv_key.zeroize();
    }
}

impl Drop for Key {
    fn drop(&mut self) {
        self.zeroize();
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyInfo {
    pub alias: String,
    pub account_id: String,
    // Base58-encoded SEC1-encoded secp256k1 ECDSA key
    pub pub_key: String,
    pub mnemonic: Option<String>,
}

impl KeyInfo {
    pub fn new(alias: String, account_id: String, pub_key: String, mnemonic: Option<String>) -> Self {
        KeyInfo {
            alias,
            account_id,
            pub_key,
            mnemonic
        }
    }
}
