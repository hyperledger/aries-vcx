use serde::{Deserialize, Serialize};

use crate::{errors::error::VcxCoreResult, wallet::base_wallet::record::Record};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndyWalletRecord {
    id: Option<String>,
    #[serde(rename = "type")]
    record_type: Option<String>,
    pub value: Option<String>,
    tags: Option<String>,
}

impl IndyWalletRecord {
    pub fn from_record(record: Record) -> VcxCoreResult<Self> {
        let tags = if record.tags().is_empty() {
            None
        } else {
            Some(serde_json::to_string(&record.tags())?)
        };

        Ok(Self {
            id: Some(record.name().into()),
            record_type: Some(record.category().to_string()),
            value: Some(record.value().into()),
            tags,
        })
    }
}
