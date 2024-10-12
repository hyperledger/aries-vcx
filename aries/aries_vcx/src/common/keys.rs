use aries_vcx_ledger::ledger::base_ledger::{IndyLedgerRead, IndyLedgerWrite};
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;
use did_parser_nom::Did;
use public_key::{Key, KeyType};
use serde_json::Value;

use crate::errors::error::prelude::*;

pub async fn rotate_verkey_apply(
    wallet: &impl BaseWallet,
    indy_ledger_write: &impl IndyLedgerWrite,
    did: &Did,
    temp_vk: &str,
) -> VcxResult<()> {
    let nym_result = indy_ledger_write
        .publish_nym(
            wallet,
            did,
            did,
            Some(&Key::from_base58(temp_vk, KeyType::Ed25519)?),
            None,
            None,
        )
        .await?;

    let nym_result_json: Value = serde_json::from_str(&nym_result).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!(
                "Cannot deserialize {:?} into Value, err: {:?}",
                nym_result, err
            ),
        )
    })?;
    let response_type: String = nym_result_json["op"]
        .as_str()
        .ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!(
                "Cannot failed to convert {:?} into str",
                nym_result_json["op"]
            ),
        ))?
        .to_string();
    if response_type != "REPLY" {
        return Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidLedgerResponse,
            format!("Obained non-success ledger response: {}", nym_result_json),
        ));
    }

    Ok(wallet.replace_did_key_apply(&did.to_string()).await?)
}

pub async fn rotate_verkey(
    wallet: &impl BaseWallet,
    indy_ledger_write: &impl IndyLedgerWrite,
    did: &Did,
) -> VcxResult<()> {
    let trustee_verkey = wallet.replace_did_key_start(&did.to_string(), None).await?;
    rotate_verkey_apply(wallet, indy_ledger_write, did, &trustee_verkey.base58()).await
}

pub async fn get_verkey_from_ledger(
    indy_ledger: &impl IndyLedgerRead,
    did: &Did,
) -> VcxResult<String> {
    let nym_response: String = indy_ledger.get_nym(did).await?;
    let nym_json: Value = serde_json::from_str(&nym_response).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!(
                "Cannot deserialize {:?} into Value, err: {:?}",
                nym_response, err
            ),
        )
    })?;
    let nym_data: String = nym_json["result"]["data"]
        .as_str()
        .ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!(
                "Cannot deserialize {:?} into String",
                nym_json["result"]["data"]
            ),
        ))?
        .to_string();
    let nym_data: Value = serde_json::from_str(&nym_data).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!(
                "Cannot deserialize {:?} into Value, err: {:?}",
                nym_data, err
            ),
        )
    })?;
    let unparsed_verkey = nym_data["verkey"]
        .as_str()
        .ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!("Cannot deserialize {:?} into String", nym_data["verkey"]),
        ))?
        .to_string();

    expand_abbreviated_verkey(did.id(), &unparsed_verkey)
}

/// Indy ledgers may return abbreviated verkeys, where the abbreviation only makes sense
/// with the context of the NYM, this function expands them to full verkeys
fn expand_abbreviated_verkey(nym: &str, verkey: &str) -> VcxResult<String> {
    let Some(stripped_verkey) = verkey.strip_prefix('~') else {
        // expansion not needed
        return Ok(verkey.to_string());
    };
    let mut decoded_nym = bs58::decode(nym).into_vec().map_err(|e| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidLedgerResponse,
            format!("Failed to decode did from base58: {} (error: {})", nym, e),
        )
    })?;
    let decoded_stripped_verkey = bs58::decode(stripped_verkey).into_vec().map_err(|e| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidLedgerResponse,
            format!(
                "Failed to decode verkey from base58: {} (error: {})",
                stripped_verkey, e
            ),
        )
    })?;
    decoded_nym.extend(&decoded_stripped_verkey);

    Ok(bs58::encode(decoded_nym).into_string())
}

// todo: was originally written for vdrtool ledger implementation, ideally we should moc out
//       ledger client completely
// #[cfg(test)]
// mod test {
//     use aries_vcx_core::ledger::indy::pool_mocks::{enable_pool_mocks, PoolMocks};
//
//     #[tokio::test]
//     #[ignore]
//     #[cfg(all(not(feature = "vdr_proxy_ledger")))]
//     async fn test_pool_rotate_verkey_fails() {
//         use super::*;
//
//         use crate::utils::devsetup::*;
//         use crate::utils::mockdata::mockdata_pool;
//
//         SetupProfile::run(|setup| async move {
//             enable_pool_mocks();
//
//             PoolMocks::set_next_pool_response(mockdata_pool::RESPONSE_REQNACK);
//             PoolMocks::set_next_pool_response(mockdata_pool::NYM_REQUEST_VALID);
//
//             let local_verkey_1 = setup
//                 .profile
//                 .inject_wallet()
//                 .key_for_local_did(&setup.institution_did)
//                 .await
//                 .unwrap();
//             assert_eq!(
//                 rotate_verkey(
//                     &setup.profile.inject_wallet(),
//                     &setup.profile.inject_indy_ledger_write(),
//                     &setup.institution_did
//                 )
//                 .await
//                 .unwrap_err()
//                 .kind(),
//                 AriesVcxErrorKind::InvalidLedgerResponse
//             );
//             let local_verkey_2 = setup
//                 .profile
//                 .inject_wallet()
//                 .key_for_local_did(&setup.institution_did)
//                 .await
//                 .unwrap();
//             assert_eq!(local_verkey_1, local_verkey_2);
//         })
//         .await;
//     }
// }
