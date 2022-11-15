use std::sync::Arc;

use crate::{error::prelude::*, core::profile::profile::Profile};

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
            VcxError::from_msg(
                VcxErrorKind::SerializationError,
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
    use crate::{indy::test_utils::create_and_store_credential_def, utils::devsetup::SetupIndyWalletPool};

    #[tokio::test]
    async fn test_create_rev_reg_delta_from_ledger() {
        let setup = SetupWalletPool::init().await;

        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (_, _, _, _, rev_reg_id, _, _) =
            create_and_store_credential_def(setup.wallet_handle, setup.pool_handle, &setup.institution_did, attrs)
                .await;

        assert!(
            RevocationRegistryDelta::create_from_ledger(setup.pool_handle, &rev_reg_id, None, None)
                .await
                .is_ok()
        );
    }
}
