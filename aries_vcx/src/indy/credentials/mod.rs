pub mod encoding;
pub mod holder;
pub mod issuer;

use std::collections::HashMap;

use vdrtools_sys::WalletHandle;

use crate::error::prelude::*;

use self::holder::libindy_prover_get_credential;

#[derive(Serialize, Deserialize)]
struct ProverCredential {
    referent: String,
    attrs: HashMap<String, String>,
    schema_id: String,
    cred_def_id: String,
    rev_reg_id: Option<String>,
    cred_rev_id: Option<String>
}

pub async fn get_cred_rev_id(wallet_handle: WalletHandle, cred_id: &str) -> VcxResult<String> {
    let cred_json = libindy_prover_get_credential(wallet_handle, cred_id).await?;
    let prover_cred = serde_json::from_str::<ProverCredential>(&cred_json)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to deserialize anoncreds credential: {}", err)))?;
    prover_cred.cred_rev_id.ok_or(VcxError::from_msg(VcxErrorKind::InvalidRevocationDetails, "Credenial revocation id missing on credential - is this credential revokable?"))
}

#[cfg(test)]
#[cfg(feature = "pool_tests")]
mod integration_tests {
    use super::*;

    use crate::indy::test_utils::create_and_store_credential;
    use crate::utils::constants::{DEFAULT_SCHEMA_ATTRS};
    use crate::utils::devsetup::SetupWalletPool;

    #[tokio::test]
    async fn test_prover_get_credential() {
        let setup = SetupWalletPool::init().await;

        let res = create_and_store_credential(setup.wallet_handle, setup.pool_handle, &setup.institution_did, DEFAULT_SCHEMA_ATTRS).await;
        let schema_id = res.0;
        let cred_def_id = res.2;
        let cred_id = res.7;
        let rev_reg_id = res.8;

        let cred_json = libindy_prover_get_credential(setup.wallet_handle, &cred_id).await.unwrap();
        let prover_cred = serde_json::from_str::<ProverCredential>(&cred_json).unwrap();

        assert_eq!(prover_cred.schema_id, schema_id);
        assert_eq!(prover_cred.cred_def_id, cred_def_id);
        assert_eq!(prover_cred.rev_reg_id.unwrap(), rev_reg_id);
    }
}
