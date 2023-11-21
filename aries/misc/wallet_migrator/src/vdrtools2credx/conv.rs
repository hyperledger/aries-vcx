use aries_vcx_core::anoncreds::credx_anoncreds::{
    RevocationRegistryInfo, CATEGORY_CREDENTIAL, CATEGORY_CRED_DEF, CATEGORY_CRED_DEF_PRIV,
    CATEGORY_CRED_KEY_CORRECTNESS_PROOF, CATEGORY_CRED_MAP_SCHEMA_ID, CATEGORY_CRED_SCHEMA,
    CATEGORY_LINK_SECRET, CATEGORY_REV_REG, CATEGORY_REV_REG_DEF, CATEGORY_REV_REG_DEF_PRIV,
    CATEGORY_REV_REG_DELTA, CATEGORY_REV_REG_INFO,
};
use vdrtools::{types::domain::wallet::Record, IndyError};

use crate::error::MigrationResult;

// The deltas in libvdrtools are prefixed. For absolutely no reason.
const REV_REG_DELTA_ID_PREFIX: &str = "rev_reg_delta:";

pub fn convert_master_secret(mut record: Record) -> MigrationResult<Record> {
    let master_secret: vdrtools::MasterSecret = serde_json::from_str(&record.value)?;

    record.value = master_secret
        .value
        .value()
        .map_err(IndyError::from)?
        .to_dec()
        .map_err(IndyError::from)?;

    record.type_ = CATEGORY_LINK_SECRET.to_owned();

    Ok(record)
}

pub fn convert_cred(mut record: Record) -> MigrationResult<Record> {
    record.type_ = CATEGORY_CREDENTIAL.to_owned();
    let _: credx::types::Credential = serde_json::from_str(&record.value)?;
    Ok(record)
}

pub fn convert_cred_def(mut record: Record) -> MigrationResult<Record> {
    record.type_ = CATEGORY_CRED_DEF.to_owned();
    let _: credx::types::CredentialDefinition = serde_json::from_str(&record.value)?;
    Ok(record)
}

pub fn convert_cred_def_priv_key(mut record: Record) -> MigrationResult<Record> {
    record.type_ = CATEGORY_CRED_DEF_PRIV.to_owned();
    let _: credx::types::CredentialDefinitionPrivate = serde_json::from_str(&record.value)?;
    Ok(record)
}

pub fn convert_cred_def_correctness_proof(mut record: Record) -> MigrationResult<Record> {
    record.type_ = CATEGORY_CRED_KEY_CORRECTNESS_PROOF.to_owned();
    let old: vdrtools::CredentialDefinitionCorrectnessProof = serde_json::from_str(&record.value)?;
    let old_value = serde_json::to_string(&old.value)?;
    let new_value = serde_json::from_str(&old_value)?;
    let new = credx::types::CredentialKeyCorrectnessProof { value: new_value };
    record.value = serde_json::to_string(&new)?;
    Ok(record)
}

pub fn convert_schema(mut record: Record) -> MigrationResult<Record> {
    record.type_ = CATEGORY_CRED_SCHEMA.to_owned();
    let _: credx::types::Schema = serde_json::from_str(&record.value)?;
    Ok(record)
}

pub fn convert_schema_id(mut record: Record) -> MigrationResult<Record> {
    record.type_ = CATEGORY_CRED_MAP_SCHEMA_ID.to_owned();
    // The plain ID is stored as a String,
    // so not that much to check.
    let _ = credx::types::SchemaId(record.value.clone());
    Ok(record)
}

pub fn convert_rev_reg(mut record: Record) -> MigrationResult<Record> {
    record.type_ = CATEGORY_REV_REG.to_owned();
    let _: credx::types::RevocationRegistry = serde_json::from_str(&record.value)?;
    Ok(record)
}

pub fn convert_rev_reg_delta(mut record: Record) -> MigrationResult<Record> {
    record.type_ = CATEGORY_REV_REG_DELTA.to_owned();

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

pub fn convert_rev_reg_info(mut record: Record) -> MigrationResult<Record> {
    record.type_ = CATEGORY_REV_REG_INFO.to_owned();
    let _: RevocationRegistryInfo = serde_json::from_str(&record.value)?;
    Ok(record)
}

pub fn convert_rev_reg_def(mut record: Record) -> MigrationResult<Record> {
    record.type_ = CATEGORY_REV_REG_DEF.to_owned();
    let _: credx::types::RevocationRegistryDefinition = serde_json::from_str(&record.value)?;
    Ok(record)
}

pub fn convert_rev_reg_def_priv(mut record: Record) -> MigrationResult<Record> {
    record.type_ = CATEGORY_REV_REG_DEF_PRIV.to_owned();
    let _: credx::types::RevocationRegistryDefinitionPrivate = serde_json::from_str(&record.value)?;
    Ok(record)
}
