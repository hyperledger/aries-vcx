pub mod conv;

use vdrtools::types::domain::wallet::Record;

use crate::error::MigrationResult;

/// Contains the logic for record mapping and migration.
pub fn migrate_any_record(record: Record) -> MigrationResult<Option<Record>> {
    match record.type_.as_str() {
        // Are these needed?
        // "Indy::Did" => Ok(Some(record)),
        // "Indy::Key" => Ok(Some(record)),
        // Master secret
        "Indy::MasterSecret" => Some(conv::convert_master_secret(record)).transpose(),
        // Credential
        "Indy::Credential" => Ok(Some(record)),
        "Indy::CredentialDefinition" => Ok(Some(record)),
        "Indy::CredentialDefinitionPrivateKey" => Ok(Some(record)),
        "Indy::CredentialDefinitionCorrectnessProof" => Ok(Some(record)),
        // Schema
        "Indy::Schema" => Ok(Some(record)),
        "Indy::SchemaId" => Ok(Some(record)),
        // Revocation registry
        "Indy::RevocationRegistry" => Ok(Some(record)),
        "Indy::RevocationRegistryDelta" => Ok(Some(record)),
        "Indy::RevocationRegistryInfo" => Ok(Some(record)),
        "Indy::RevocationRegistryDefinition" => Ok(Some(record)),
        "Indy::RevocationRegistryDefinitionPrivate" => Ok(Some(record)),
        _ => Ok(None), // Ignore unknown/uninteresting records
    }
}
