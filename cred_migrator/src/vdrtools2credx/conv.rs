use aries_vcx_core::anoncreds::credx_anoncreds::CATEGORY_LINK_SECRET;
use vdrtools::{types::domain::wallet::Record, Credential, IndyError};

use crate::error::MigrationResult;

pub fn convert_master_secret(mut record: Record) -> MigrationResult<Record> {
    let master_secret: vdrtools::MasterSecret = serde_json::from_str(&record.value)?;

    record.type_ = CATEGORY_LINK_SECRET.to_owned();
    record.value = master_secret
        .value
        .value()
        .map_err(IndyError::from)?
        .to_dec()
        .map_err(IndyError::from)?;
    Ok(record)
}

pub fn convert_cred(mut record: Record) -> MigrationResult<Record> {
    let cred: Credential = serde_json::from_str(&record.value)?;
    Ok(record)
}

pub fn convert_cred_def(mut record: Record) -> MigrationResult<Record> {
    Ok(record)
}

pub fn convert_cred_def_priv_key(mut record: Record) -> MigrationResult<Record> {
    Ok(record)
}

pub fn convert_cred_def_correctness_proof(mut record: Record) -> MigrationResult<Record> {
    Ok(record)
}

pub fn convert_schema(mut record: Record) -> MigrationResult<Record> {
    Ok(record)
}

pub fn convert_schema_id(mut record: Record) -> MigrationResult<Record> {
    Ok(record)
}

pub fn convert_rev_reg(mut record: Record) -> MigrationResult<Record> {
    Ok(record)
}

pub fn convert_rev_reg_delta(mut record: Record) -> MigrationResult<Record> {
    Ok(record)
}

pub fn convert_rev_reg_info(mut record: Record) -> MigrationResult<Record> {
    Ok(record)
}

pub fn convert_rev_reg_def(mut record: Record) -> MigrationResult<Record> {
    Ok(record)
}

pub fn convert_rev_reg_def_priv(mut record: Record) -> MigrationResult<Record> {
    Ok(record)
}
