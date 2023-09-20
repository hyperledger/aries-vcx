use std::sync::Arc;

use aries_vcx_core::{
    ledger::base_ledger::{IndyLedgerRead, IndyLedgerWrite},
    wallet::base_wallet::BaseWallet,
};
use serde_json::Value;

use crate::errors::error::prelude::*;

pub async fn rotate_verkey_apply(
    wallet: &Arc<dyn BaseWallet>,
    indy_ledger_write: &Arc<dyn IndyLedgerWrite>,
    did: &str,
    temp_vk: &str,
) -> VcxResult<()> {
    let nym_result = indy_ledger_write
        .publish_nym(did, did, Some(temp_vk), None, None)
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

    wallet
        .replace_did_keys_apply(did)
        .await
        .map_err(|err| err.into())
}

pub async fn rotate_verkey(
    wallet: &Arc<dyn BaseWallet>,
    indy_ledger_write: &Arc<dyn IndyLedgerWrite>,
    did: &str,
) -> VcxResult<()> {
    let trustee_temp_verkey = wallet.replace_did_keys_start(did).await?;
    rotate_verkey_apply(wallet, indy_ledger_write, did, &trustee_temp_verkey).await
}

pub async fn get_verkey_from_ledger(
    indy_ledger: &Arc<dyn IndyLedgerRead>,
    did: &str,
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
    Ok(nym_data["verkey"]
        .as_str()
        .ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!("Cannot deserialize {:?} into String", nym_data["verkey"]),
        ))?
        .to_string())
}

// todo: was originally written for vdrtool ledger implementation, ideally we should moc out
//       ledger client completely
// #[cfg(test)]
// #[allow(clippy::unwrap_used)]
// mod test {
//     use aries_vcx_core::ledger::indy::pool_mocks::{enable_pool_mocks, PoolMocks};
//
//     #[tokio::test]
//     #[ignore]
//     #[cfg(all(not(feature = "vdr_proxy_ledger"), not(feature = "modular_libs"),))]
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
