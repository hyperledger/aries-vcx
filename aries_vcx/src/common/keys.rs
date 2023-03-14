use std::sync::Arc;

use serde_json::Value;

use crate::{core::profile::profile::Profile, errors::error::prelude::*};

pub async fn rotate_verkey_apply(profile: &Arc<dyn Profile>, did: &str, temp_vk: &str) -> VcxResult<()> {
    let ledger = Arc::clone(profile).inject_ledger();

    let nym_result = ledger.publish_nym(did, did, Some(temp_vk), None, None).await?;

    let nym_result_json: Value = serde_json::from_str(&nym_result).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!("Cannot deserialize {:?} into Value, err: {:?}", nym_result, err),
        )
    })?;
    let response_type: String = nym_result_json["op"]
        .as_str()
        .ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!("Cannot failed to convert {:?} into str", nym_result_json["op"]),
        ))?
        .to_string();
    if response_type != "REPLY" {
        return Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidLedgerResponse,
            format!("Obained non-success ledger response: {}", nym_result_json),
        ));
    }

    let wallet = profile.inject_wallet();
    wallet.replace_did_keys_apply(did).await
}

pub async fn rotate_verkey(profile: &Arc<dyn Profile>, did: &str) -> VcxResult<()> {
    let wallet = profile.inject_wallet();
    let trustee_temp_verkey = wallet.replace_did_keys_start(did).await?;
    rotate_verkey_apply(profile, did, &trustee_temp_verkey).await
}

pub async fn get_verkey_from_ledger(profile: &Arc<dyn Profile>, did: &str) -> VcxResult<String> {
    let ledger = Arc::clone(profile).inject_ledger();

    let nym_response: String = ledger.get_nym(did).await?;
    let nym_json: Value = serde_json::from_str(&nym_response).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!("Cannot deserialize {:?} into Value, err: {:?}", nym_response, err),
        )
    })?;
    let nym_data: String = nym_json["result"]["data"]
        .as_str()
        .ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!("Cannot deserialize {:?} into String", nym_json["result"]["data"]),
        ))?
        .to_string();
    let nym_data: Value = serde_json::from_str(&nym_data).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!("Cannot deserialize {:?} into Value, err: {:?}", nym_data, err),
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

#[cfg(test)]
#[cfg(feature = "pool_tests")]
mod test {
    use super::*;
    use crate::{
        indy::utils::mocks::pool_mocks::{enable_pool_mocks, PoolMocks},
        utils::{devsetup::*, mockdata::mockdata_pool},
    };

    #[tokio::test]
    async fn test_rotate_verkey_fails() {
        SetupProfile::run_indy(|setup| async move {
            enable_pool_mocks();

            PoolMocks::set_next_pool_response(mockdata_pool::RESPONSE_REQNACK);
            PoolMocks::set_next_pool_response(mockdata_pool::NYM_REQUEST_VALID);

            let local_verkey_1 = setup
                .profile
                .inject_wallet()
                .key_for_local_did(&setup.institution_did)
                .await
                .unwrap();
            assert_eq!(
                rotate_verkey(&setup.profile, &setup.institution_did)
                    .await
                    .unwrap_err()
                    .kind(),
                AriesVcxErrorKind::InvalidLedgerResponse
            );
            let local_verkey_2 = setup
                .profile
                .inject_wallet()
                .key_for_local_did(&setup.institution_did)
                .await
                .unwrap();
            assert_eq!(local_verkey_1, local_verkey_2);
        })
        .await;
    }
}
