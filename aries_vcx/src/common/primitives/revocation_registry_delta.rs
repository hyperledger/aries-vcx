use std::sync::Arc;

use crate::{core::profile::profile::Profile, errors::error::prelude::*};

#[derive(Clone, Deserialize, Debug, Serialize, Default)]
pub struct RevocationRegistryDelta {
    value: RevocationRegistryDeltaValue,
    #[serde(rename = "ver")]
    version: String,
}

#[derive(Clone, Deserialize, Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDeltaValue {
    prev_accum: Option<String>,
    accum: String,
    #[serde(default)]
    issued: Vec<u32>,
    #[serde(default)]
    revoked: Vec<u32>,
}

impl RevocationRegistryDeltaValue {
    pub fn issued(&self) -> &[u32] {
        self.issued.as_ref()
    }

    pub fn revoked(&self) -> &[u32] {
        self.revoked.as_ref()
    }
}

impl RevocationRegistryDelta {
    pub async fn create_from_ledger(
        profile: &Arc<dyn Profile>,
        rev_reg_id: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxResult<Self> {
        let ledger = Arc::clone(profile).inject_ledger();
        let (_, rev_reg_delta_json, _) = ledger.get_rev_reg_delta_json(rev_reg_id, from, to).await?;
        serde_json::from_str(&rev_reg_delta_json).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::SerializationError,
                format!("Failed to deserialize rev_reg_delta_json from ledger, err: {}", err),
            )
        })
    }

    pub fn issued(&self) -> &[u32] {
        self.value.issued()
    }

    pub fn revoked(&self) -> &[u32] {
        self.value.revoked()
    }
}

#[cfg(test)]
#[cfg(feature = "pool_tests")]
pub mod integration_tests {
    use super::*;
    use crate::{common::test_utils::create_and_store_credential_def, utils::devsetup::SetupProfile};

    #[tokio::test]
    async fn test_create_rev_reg_delta_from_ledger() {
        SetupProfile::run_indy(|setup| async move {
            let attrs = r#"["address1","address2","city","state","zip"]"#;
            let (_, _, _, _, rev_reg_id, _, _) =
                create_and_store_credential_def(&setup.profile, &setup.institution_did, attrs).await;

            assert!(
                RevocationRegistryDelta::create_from_ledger(&setup.profile, &rev_reg_id, None, None)
                    .await
                    .is_ok()
            );
        })
        .await;
    }
}
