pub mod conv;

use vdrtools::types::domain::wallet::Record;

use crate::error::MigrationResult;

/// Contains the logic for record mapping and migration.
pub fn migrate_any_record(record: Record) -> MigrationResult<Option<Record>> {
    match record.type_.as_str() {
        // Master secret
        "Indy::MasterSecret" => Some(conv::convert_master_secret(record)).transpose(),
        // Credential
        "Indy::Credential" => Some(conv::convert_cred(record)).transpose(),
        "Indy::CredentialDefinition" => Some(conv::convert_cred_def(record)).transpose(),
        "Indy::CredentialDefinitionPrivateKey" => Some(conv::convert_cred_def_priv_key(record)).transpose(),
        "Indy::CredentialDefinitionCorrectnessProof" => {
            Some(conv::convert_cred_def_correctness_proof(record)).transpose()
        }
        // Schema
        "Indy::Schema" => Some(conv::convert_schema(record)).transpose(),
        "Indy::SchemaId" => Some(conv::convert_schema_id(record)).transpose(),
        // Revocation registry
        "Indy::RevocationRegistry" => Some(conv::convert_rev_reg(record)).transpose(),
        "Indy::RevocationRegistryDelta" => Some(conv::convert_rev_reg_delta(record)).transpose(),
        "Indy::RevocationRegistryInfo" => Some(conv::convert_rev_reg_info(record)).transpose(),
        "Indy::RevocationRegistryDefinition" => Some(conv::convert_rev_reg_def(record)).transpose(),
        "Indy::RevocationRegistryDefinitionPrivate" => Some(conv::convert_rev_reg_def_priv(record)).transpose(),
        _ => Ok(None), // Ignore unknown/uninteresting records
    }
}
