use aries_vcx_core::anoncreds::credx_anoncreds::CATEGORY_LINK_SECRET;
use vdrtools::types::errors::IndyErrorKind;
pub use vdrtools::{
    types::domain::wallet::{Config, Credentials, KeyDerivationMethod, Record},
    IndyError, Locator, WalletHandle,
};

// pub const CATEGORY_LINK_SECRET: &str = "VCX_LINK_SECRET";

// pub const CATEGORY_CREDENTIAL: &str = "VCX_CREDENTIAL";
// pub const CATEGORY_CRED_DEF: &str = "VCX_CRED_DEF";
// pub const CATEGORY_CRED_KEY_CORRECTNESS_PROOF: &str = "VCX_CRED_KEY_CORRECTNESS_PROOF";
// pub const CATEGORY_CRED_DEF_PRIV: &str = "VCX_CRED_DEF_PRIV";
// pub const CATEGORY_CRED_SCHEMA: &str = "VCX_CRED_SCHEMA";

// // Category used for mapping a cred_def_id to a schema_id
// pub const CATEGORY_CRED_MAP_SCHEMA_ID: &str = "VCX_CRED_MAP_SCHEMA_ID";

// pub const CATEGORY_REV_REG: &str = "VCX_REV_REG";
// pub const CATEGORY_REV_REG_DELTA: &str = "VCX_REV_REG_DELTA";
// pub const CATEGORY_REV_REG_INFO: &str = "VCX_REV_REG_INFO";
// pub const CATEGORY_REV_REG_DEF: &str = "VCX_REV_REG_DEF";
// pub const CATEGORY_REV_REG_DEF_PRIV: &str = "VCX_REV_REG_DEF_PRIV";

/// Contains the logic for record mapping and migration.
pub fn migrate_any_record(record: Record) -> Result<Option<Record>, IndyError> {
    match record.type_.as_str() {
        "Indy::MasterSecret" => Some(convert_master_secret(record)).transpose(),
        // "Indy::Did" => Ok(Some(record)),
        // "Indy::Key" => Ok(Some(record)),

        "Indy::Credential" => Ok(Some(record)),
        "Indy::CredentialDefinition" => Ok(Some(record)),
        "Indy::CredentialDefinitionPrivateKey" => Ok(Some(record)),
        "Indy::CredentialDefinitionCorrectnessProof" => Ok(Some(record)),
        "Indy::Schema" => Ok(Some(record)),
        "Indy::SchemaId" => Ok(Some(record)),
        _ => Ok(None), // Ignore unknown/uninteresting records
    }
}

pub fn convert_master_secret(mut record: Record) -> Result<Record, IndyError> {
    let master_secret: vdrtools::MasterSecret =
        serde_json::from_str(&record.value).map_err(|e| IndyError::from_msg(IndyErrorKind::WalletItemNotFound, e))?;

    record.type_ = CATEGORY_LINK_SECRET.to_owned();
    record.value = master_secret.value.value()?.to_dec()?;
    Ok(record)
}

/// Retrieves all records from a wallet and migrates them
/// by applying the `migrate_fn` argument.
///
/// The migrated records are inserted into a newly created
/// wallet, based on the provided `config` and `credentials`.
pub async fn migrate_wallet(
    wallet_handle: WalletHandle,
    config: Config,
    credentials: Credentials,
    migrate_fn: impl Fn(Record) -> Result<Option<Record>, IndyError>,
) -> Result<(), IndyError> {
    println!("Migrating wallet");

    let locator = Locator::instance();

    locator
        .wallet_controller
        .create(config.clone(), credentials.clone())
        .await?;

    let new_wh = locator.wallet_controller.open(config, credentials).await?;

    locator
        .wallet_controller
        .migrate_records(wallet_handle, new_wh, migrate_fn)
        .await?;

    locator.wallet_controller.close(new_wh).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_migration() {
        let wallet_name = "wallet_test_migration".to_owned();
        let wallet_key = "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY".to_owned();

        let credentials = Credentials {
            key: wallet_key.clone(),
            key_derivation_method: KeyDerivationMethod::RAW,
            rekey: None,
            rekey_derivation_method: KeyDerivationMethod::ARGON2I_MOD,
            storage_credentials: None,
        };

        let config = Config {
            id: wallet_name,
            storage_type: None,
            storage_config: None,
            cache: None,
        };

        Locator::instance()
            .wallet_controller
            .create(config.clone(), credentials.clone())
            .await
            .unwrap();
        let wallet_handle = Locator::instance()
            .wallet_controller
            .open(config.clone(), credentials.clone())
            .await
            .unwrap();

        let new_wallet_name = "new_better_wallet".to_owned();

        let new_credentials = Credentials {
            key: wallet_key,
            key_derivation_method: KeyDerivationMethod::RAW,
            rekey: None,
            rekey_derivation_method: KeyDerivationMethod::ARGON2I_MOD,
            storage_credentials: None,
        };

        let new_config = Config {
            id: new_wallet_name,
            storage_type: None,
            storage_config: None,
            cache: None,
        };

        if let Ok(()) = migrate_wallet(
            wallet_handle,
            new_config.clone(),
            new_credentials.clone(),
            migrate_any_record,
        )
        .await
        {
            Locator::instance()
                .wallet_controller
                .delete(config, credentials)
                .await
                .ok();
            Locator::instance()
                .wallet_controller
                .delete(new_config, new_credentials)
                .await
                .ok();
        }
    }
}
