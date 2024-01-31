use vdrtools::types::domain::wallet::IndyRecord;

use crate::error::MigrationResult;

pub fn convert_master_secret(record: IndyRecord) -> MigrationResult<IndyRecord> {
    Ok(record)
}

pub fn convert_cred(record: IndyRecord) -> MigrationResult<IndyRecord> {
    Ok(record)
}

pub fn convert_cred_def(record: IndyRecord) -> MigrationResult<IndyRecord> {
    Ok(record)
}

pub fn convert_cred_def_priv_key(record: IndyRecord) -> MigrationResult<IndyRecord> {
    Ok(record)
}

pub fn convert_cred_def_correctness_proof(record: IndyRecord) -> MigrationResult<IndyRecord> {
    Ok(record)
}

pub fn convert_schema(record: IndyRecord) -> MigrationResult<IndyRecord> {
    Ok(record)
}

pub fn convert_rev_reg(record: IndyRecord) -> MigrationResult<IndyRecord> {
    Ok(record)
}

pub fn convert_rev_reg_delta(record: IndyRecord) -> MigrationResult<IndyRecord> {
    Ok(record)
}

pub fn convert_rev_reg_def(record: IndyRecord) -> MigrationResult<IndyRecord> {
    Ok(record)
}

pub fn convert_rev_reg_def_priv(record: IndyRecord) -> MigrationResult<IndyRecord> {
    Ok(record)
}
