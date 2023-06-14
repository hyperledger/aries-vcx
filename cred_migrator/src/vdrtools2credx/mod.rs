pub mod conv;

use vdrtools::types::domain::wallet::Record;

use crate::error::MigrationResult;

const INDY_MASTER_SECRET: &str = "Indy::MasterSecret";
const INDY_CRED: &str = "Indy::Credential";
const INDY_CRED_DEF: &str = "Indy::CredentialDefinition";
const INDY_CRED_DEF_PRIV: &str = "Indy::CredentialDefinitionPrivateKey";
const INDY_CRED_DEF_CR_PROOF: &str = "Indy::CredentialDefinitionCorrectnessProof";
const INDY_SCHEMA: &str = "Indy::Schema";
const INDY_SCHEMA_ID: &str = "Indy::SchemaId";
const INDY_REV_REG: &str = "Indy::RevocationRegistry";
const INDY_REV_REG_DELTA: &str = "Indy::RevocationRegistryDelta";
const INDY_REV_REG_INFO: &str = "Indy::RevocationRegistryInfo";
const INDY_REV_REG_DEF: &str = "Indy::RevocationRegistryDefinition";
const INDY_REV_REG_DEF_PRIV: &str = "Indy::RevocationRegistryDefinitionPrivate";

/// Contains the logic for record mapping and migration.
pub fn migrate_any_record(record: Record) -> MigrationResult<Option<Record>> {
    match record.type_.as_str() {
        // Master secret
        INDY_MASTER_SECRET => Some(conv::convert_master_secret(record)).transpose(),
        // Credential
        INDY_CRED => Some(conv::convert_cred(record)).transpose(),
        INDY_CRED_DEF => Some(conv::convert_cred_def(record)).transpose(),
        INDY_CRED_DEF_PRIV => Some(conv::convert_cred_def_priv_key(record)).transpose(),
        INDY_CRED_DEF_CR_PROOF => Some(conv::convert_cred_def_correctness_proof(record)).transpose(),
        // Schema
        INDY_SCHEMA => Some(conv::convert_schema(record)).transpose(),
        INDY_SCHEMA_ID => Some(conv::convert_schema_id(record)).transpose(),
        // Revocation registry
        INDY_REV_REG => Some(conv::convert_rev_reg(record)).transpose(),
        INDY_REV_REG_DELTA => Some(conv::convert_rev_reg_delta(record)).transpose(),
        INDY_REV_REG_INFO => Some(conv::convert_rev_reg_info(record)).transpose(),
        INDY_REV_REG_DEF => Some(conv::convert_rev_reg_def(record)).transpose(),
        INDY_REV_REG_DEF_PRIV => Some(conv::convert_rev_reg_def_priv(record)).transpose(),
        _ => Ok(None), // Ignore unknown/uninteresting records
    }
}
