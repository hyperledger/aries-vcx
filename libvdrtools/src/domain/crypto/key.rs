extern crate zeroize;

use self::zeroize::Zeroize;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Key {
    pub verkey: String,
    pub signkey: String,
}

impl Key {
    pub fn new(verkey: String, signkey: String) -> Key {
        Key {
            verkey,
            signkey,
        }
    }
}

impl Zeroize for Key {
    fn zeroize(&mut self) {
        self.signkey.zeroize();
    }
}

impl Drop for Key {
    fn drop(&mut self) {
        self.signkey.zeroize();
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct KeyInfo {
    pub seed: Option<String>,
    pub crypto_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyMetadata {
    pub value: String
}
