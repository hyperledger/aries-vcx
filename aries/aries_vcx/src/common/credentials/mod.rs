use aries_vcx_anoncreds::anoncreds::base_anoncreds::{BaseAnonCreds, CredentialId};
use aries_vcx_ledger::ledger::base_ledger::AnoncredsLedgerRead;
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;
use time::OffsetDateTime;

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

pub mod encoding;

pub async fn get_cred_rev_id(
    wallet: &impl BaseWallet,
    anoncreds: &impl BaseAnonCreds,
    cred_id: &CredentialId,
) -> VcxResult<u32> {
    let cred_json = anoncreds.prover_get_credential(wallet, cred_id).await?;
    cred_json.cred_rev_id.ok_or(AriesVcxError::from_msg(
        AriesVcxErrorKind::InvalidRevocationDetails,
        "Credenial revocation id missing on credential - is this credential revokable?",
    ))
}

pub async fn is_cred_revoked(
    ledger: &impl AnoncredsLedgerRead,
    rev_reg_id: &str,
    rev_id: u32,
) -> VcxResult<bool> {
    let to = Some(OffsetDateTime::now_utc().unix_timestamp() as u64 + 100);
    #[allow(deprecated)] // TODO - https://github.com/hyperledger/aries-vcx/issues/1309
    let (rev_reg_delta, _) = ledger
        .get_rev_reg_delta_json(&rev_reg_id.try_into()?, None, to)
        .await?;
    Ok(rev_reg_delta.value.revoked.iter().any(|s| s.eq(&rev_id)))
}
