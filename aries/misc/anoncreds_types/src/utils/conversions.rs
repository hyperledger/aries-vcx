use bitvec::bitvec;

use crate::data_types::{
    identifiers::{issuer_id::IssuerId, rev_reg_def_id::RevocationRegistryDefinitionId},
    ledger::{rev_reg_delta::RevocationRegistryDeltaValue, rev_status_list::RevocationStatusList},
};

/// Converts from a [RevocationRegistryDeltaValue] into a completed [RevocationStatusList]
/// (newer format).
///
/// NOTE: this conversion only works if the delta was calculated from START (timestamp 0/None)
/// to `timestamp`.
pub fn from_revocation_registry_delta_to_revocation_status_list(
    delta: &RevocationRegistryDeltaValue,
    timestamp: u64,
    rev_reg_id: &RevocationRegistryDefinitionId,
    max_cred_num: usize,
    issuer_id: IssuerId,
) -> Result<RevocationStatusList, crate::Error> {
    // no way to derive this value here. So we assume true, as false (ISSAUNCE_ON_DEAMAND) is not
    // recomended by anoncreds: https://hyperledger.github.io/anoncreds-spec/#anoncreds-issuer-setup-with-revocation
    let issuance_by_default = true;
    let default_state = if issuance_by_default { 0 } else { 1 };
    let mut revocation_list = bitvec![default_state; max_cred_num];
    let revocation_len = revocation_list.len();

    for issued in &delta.issued {
        if revocation_len <= *issued as usize {
            return Err(crate::Error::from_msg(
                crate::ErrorKind::ConversionError,
                format!(
                    "Error whilst constructing a revocation status list from the ledger's delta. \
                     Ledger delta reported an issuance for cred_rev_id '{issued}', but the \
                     revocation_list max size is {revocation_len}"
                ),
            ));
        }
        revocation_list.insert(*issued as usize, false);
    }

    for revoked in &delta.revoked {
        if revocation_len <= *revoked as usize {
            return Err(crate::Error::from_msg(
                crate::ErrorKind::ConversionError,
                format!(
                    "Error whilst constructing a revocation status list from the ledger's delta. \
                     Ledger delta reported an revocation for cred_rev_id '{revoked}', but the \
                     revocation_list max size is {revocation_len}"
                ),
            ));
        }
        revocation_list.insert(*revoked as usize, true);
    }

    let accum = delta.accum.into();

    RevocationStatusList::new(
        Some(&rev_reg_id.to_string()),
        issuer_id,
        revocation_list,
        Some(accum),
        Some(timestamp),
    )
    .map_err(Into::into)
}
