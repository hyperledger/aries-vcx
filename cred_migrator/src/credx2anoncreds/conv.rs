use vdrtools::types::domain::wallet::Record;

use crate::error::MigrationResult;

pub fn convert_master_secret(record: Record) -> MigrationResult<Record> {
    Ok(record)
}

pub fn convert_cred(record: Record) -> MigrationResult<Record> {
    Ok(record)
}

pub fn convert_cred_def(record: Record) -> MigrationResult<Record> {
    Ok(record)
}

pub fn convert_cred_def_priv_key(record: Record) -> MigrationResult<Record> {
    Ok(record)
}

pub fn convert_cred_def_correctness_proof(record: Record) -> MigrationResult<Record> {
    Ok(record)
}

pub fn convert_schema(record: Record) -> MigrationResult<Record> {
    Ok(record)
}

pub fn convert_rev_reg(record: Record) -> MigrationResult<Record> {
    Ok(record)
}

pub fn convert_rev_reg_delta(record: Record) -> MigrationResult<Record> {
    Ok(record)
}

pub fn convert_rev_reg_def(record: Record) -> MigrationResult<Record> {
    Ok(record)
}

pub fn convert_rev_reg_def_priv(record: Record) -> MigrationResult<Record> {
    Ok(record)
}
