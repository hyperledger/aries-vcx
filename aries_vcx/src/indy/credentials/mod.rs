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
