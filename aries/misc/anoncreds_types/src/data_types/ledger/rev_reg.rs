use crate::cl::RevocationRegistry as CryptoRevocationRegistry;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RevocationRegistry {
    pub value: CryptoRevocationRegistry,
}
