use anoncreds_clsignatures::{RevocationRegistry as CryptoRevocationRegistry, Witness};

use crate::{invalid, utils::validation::Validatable, Error};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialRevocationState {
    pub witness: Witness,
    pub rev_reg: CryptoRevocationRegistry,
    pub timestamp: u64,
}

impl Validatable for CredentialRevocationState {
    fn validate(&self) -> std::result::Result<(), Error> {
        if self.timestamp == 0 {
            return Err(invalid!(
                "Credential Revocation State validation failed: `timestamp` must be greater than 0",
            ));
        }
        Ok(())
    }
}
