pub mod key;
pub mod did;
pub mod combo_box;
pub mod pack;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum CryptoTypes {
    #[serde(rename="ed25519")]
    Ed25519,
    #[serde(rename="secp256k1")]
    Secp256k1,
}

pub const ED25519: &str = "ed25519";
pub const SECP256K1: &str = "Secp256k1";