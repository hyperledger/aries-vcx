use aries_vcx_core::ledger::base_ledger::AnoncredsLedgerRead;
use std::sync::Arc;

use crate::errors::error::prelude::*;

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
        ledger: &Arc<dyn AnoncredsLedgerRead>,
        rev_reg_id: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxResult<Self> {
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
#[allow(clippy::unwrap_used)]
pub mod integration_tests {
    use super::*;
    use crate::{common::test_utils::create_and_store_credential_def_and_rev_reg, utils::devsetup::SetupProfile};

    #[tokio::test]
    #[ignore]
    async fn test_pool_create_rev_reg_delta_from_ledger() {
        SetupProfile::run(|setup| async move {
            let attrs = r#"["address1","address2","city","state","zip"]"#;
            let (_, _, _, _, rev_reg_id, _, _, _) = create_and_store_credential_def_and_rev_reg(
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds_ledger_read(),
                &setup.profile.inject_anoncreds_ledger_write(),
                &setup.institution_did,
                attrs,
            )
            .await;

            assert!(RevocationRegistryDelta::create_from_ledger(
                &setup.profile.inject_anoncreds_ledger_read(),
                &rev_reg_id,
                None,
                None
            )
            .await
            .is_ok());
        })
        .await;
    }
}
