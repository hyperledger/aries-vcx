use crate::cl::RevocationRegistry as CryptoRevocationRegistry;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RevocationRegistry {
    pub value: CryptoRevocationRegistry,
}
