use aries_vcx_core::{
    anoncreds::credx_anoncreds::RevocationRegistryInfo,
    wallet::base_wallet::record_category::RecordCategory,
};
use vdrtools::{types::domain::wallet::IndyRecord, IndyError};

use crate::error::MigrationResult;

// The deltas in libvdrtools are prefixed. For absolutely no reason.
const REV_REG_DELTA_ID_PREFIX: &str = "rev_reg_delta:";

pub fn convert_master_secret(mut record: IndyRecord) -> MigrationResult<IndyRecord> {
    let master_secret: vdrtools::MasterSecret = serde_json::from_str(&record.value)?;

    record.value = master_secret
        .value
        .value()
        .map_err(IndyError::from)?
        .to_dec()
        .map_err(IndyError::from)?;

    record.type_ = RecordCategory::LinkSecret.to_string();

    Ok(record)
}

pub fn convert_cred(mut record: IndyRecord) -> MigrationResult<IndyRecord> {
    record.type_ = RecordCategory::Cred.to_string();
    let _: credx::types::Credential = serde_json::from_str(&record.value)?;
    Ok(record)
}

pub fn convert_cred_def(mut record: IndyRecord) -> MigrationResult<IndyRecord> {
    record.type_ = RecordCategory::CredDef.to_string();
    let _: credx::types::CredentialDefinition = serde_json::from_str(&record.value)?;
    Ok(record)
}

pub fn convert_cred_def_priv_key(mut record: IndyRecord) -> MigrationResult<IndyRecord> {
    record.type_ = RecordCategory::CredDefPriv.to_string();
    let _: credx::types::CredentialDefinitionPrivate = serde_json::from_str(&record.value)?;
    Ok(record)
}

pub fn convert_cred_def_correctness_proof(mut record: IndyRecord) -> MigrationResult<IndyRecord> {
    record.type_ = RecordCategory::CredKeyCorrectnessProof.to_string();
    let old: vdrtools::CredentialDefinitionCorrectnessProof = serde_json::from_str(&record.value)?;
    let old_value = serde_json::to_string(&old.value)?;
    let new_value = serde_json::from_str(&old_value)?;
    let new = credx::types::CredentialKeyCorrectnessProof { value: new_value };
    record.value = serde_json::to_string(&new)?;
    Ok(record)
}

pub fn convert_schema(mut record: IndyRecord) -> MigrationResult<IndyRecord> {
    record.type_ = RecordCategory::CredSchema.to_string();
    let _: credx::types::Schema = serde_json::from_str(&record.value)?;
    Ok(record)
}

pub fn convert_schema_id(mut record: IndyRecord) -> MigrationResult<IndyRecord> {
    record.type_ = RecordCategory::CredMapSchemaId.to_string();
    // The plain ID is stored as a String,
    // so not that much to check.
    let _ = credx::types::SchemaId(record.value.clone());
    Ok(record)
}

pub fn convert_rev_reg(mut record: IndyRecord) -> MigrationResult<IndyRecord> {
    record.type_ = RecordCategory::RevReg.to_string();
    let _: credx::types::RevocationRegistry = serde_json::from_str(&record.value)?;
    Ok(record)
}

pub fn convert_rev_reg_delta(mut record: IndyRecord) -> MigrationResult<IndyRecord> {
    record.type_ = RecordCategory::RevRegDelta.to_string();

    // Shave off the useless prefix, if found.
    record.id = record
        .id
        .strip_prefix(REV_REG_DELTA_ID_PREFIX)
        .map(ToOwned::to_owned)
        .unwrap_or(record.id);

    // Them indy devs serializing a String to JSON...
    record.value = serde_json::from_str(&record.value)?;
    let _: credx::types::RevocationRegistryDelta = serde_json::from_str(&record.value)?;
    Ok(record)
}

pub fn convert_rev_reg_info(mut record: IndyRecord) -> MigrationResult<IndyRecord> {
    record.type_ = RecordCategory::RevRegInfo.to_string();
    let _: RevocationRegistryInfo = serde_json::from_str(&record.value)?;
    Ok(record)
}

pub fn convert_rev_reg_def(mut record: IndyRecord) -> MigrationResult<IndyRecord> {
    record.type_ = RecordCategory::RevRegDef.to_string();
    let _: credx::types::RevocationRegistryDefinition = serde_json::from_str(&record.value)?;
    Ok(record)
}

pub fn convert_rev_reg_def_priv(mut record: IndyRecord) -> MigrationResult<IndyRecord> {
    record.type_ = RecordCategory::RevRegDefPriv.to_string();
    let _: credx::types::RevocationRegistryDefinitionPrivate = serde_json::from_str(&record.value)?;
    Ok(record)
}
