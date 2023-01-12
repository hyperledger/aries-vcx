pub mod did;
pub mod key;
pub mod pack;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum CryptoTypes {
    #[serde(rename = "ed25519")]
    Ed25519,
    #[serde(rename = "secp256k1")]
    Secp256k1,
}
