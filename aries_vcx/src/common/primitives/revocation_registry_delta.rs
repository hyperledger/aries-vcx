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
    pub async fn create_from_ledger(rev_reg_delta_json: &str) -> VcxResult<Self> {
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
