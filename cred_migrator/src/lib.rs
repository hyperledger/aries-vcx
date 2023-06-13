pub mod credx2anoncreds;
pub mod error;
pub mod vdrtools2credx;

use std::fmt::Display;

use error::MigrationResult;
pub use vdrtools::{
    types::domain::wallet::{Config, Credentials, KeyDerivationMethod, Record},
    Locator, WalletHandle,
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

/// Retrieves all records from a wallet and migrates them
/// by applying the `migrate_fn` argument.
///
/// The migrated records are inserted into a newly created
/// wallet, based on the provided `config` and `credentials`.
pub async fn migrate_wallet<E>(
    wallet_handle: WalletHandle,
    config: Config,
    credentials: Credentials,
    migrate_fn: impl FnMut(Record) -> Result<Option<Record>, E>,
) -> MigrationResult<()>
where
    E: Display,
{
    // LOG: migrating wallet
    let locator = Locator::instance();

    locator
        .wallet_controller
        .create(config.clone(), credentials.clone())
        .await?;

    let new_wh = locator
        .wallet_controller
        .open(config.clone(), credentials.clone())
        .await?;

    let res = locator
        .wallet_controller
        .migrate_records(wallet_handle, new_wh, migrate_fn)
        .await;

    locator.wallet_controller.close(new_wh).await?;

    if res.is_err() {
        // LOG: error encountered -> deleting newly created wallet.
        locator.wallet_controller.delete(config, credentials).await.ok();
    }

    res?;

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
            vdrtools2credx::migrate_any_record,
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
