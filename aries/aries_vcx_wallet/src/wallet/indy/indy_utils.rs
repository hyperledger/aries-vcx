use indy_api_types::domain::wallet::KeyDerivationMethod;

use crate::errors::error::{VcxWalletError, VcxWalletResult};

pub fn parse_key_derivation_method(method: &str) -> VcxWalletResult<KeyDerivationMethod> {
    match method {
        "RAW" => Ok(KeyDerivationMethod::RAW),
        "ARGON2I_MOD" => Ok(KeyDerivationMethod::ARGON2I_MOD),
        "ARGON2I_INT" => Ok(KeyDerivationMethod::ARGON2I_INT),
        _ => Err(VcxWalletError::InvalidInput(format!(
            "Unknown derivation method {method}"
        ))),
    }
}
