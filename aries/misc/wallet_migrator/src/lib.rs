pub mod credx2anoncreds;
pub mod error;
pub mod vdrtools2credx;

use std::{fmt::Display, sync::Arc};

use aries_vcx_core::wallet::base_wallet::{migrate::migrate_records, record::Record, BaseWallet};
use error::MigrationResult;
use log::{error, info};
pub use vdrtools::types::domain::wallet::IndyRecord;
use vdrtools::{Locator, WalletHandle};

use crate::error::MigrationError;

/// Retrieves all records from the source wallet and migrates them
/// by applying the `migrate_fn` argument. The records are then
/// placed in the destination wallet.
pub async fn migrate_wallet<E>(
    src_wallet: impl BaseWallet,
    dest_wallet: impl BaseWallet,
    migrate_fn: impl FnMut(Record) -> Result<Option<Record>, E>,
) -> MigrationResult<()>
where
    E: Display,
{
    info!("Starting wallet migration");

    info!(
        "Migrating records from wallet with handle {src_wallet:?} to wallet with handle \
         {dest_wallet:?}"
    );

    if src_wallet == dest_wallet {
        Err(MigrationError::EqualWallets)
    }

    migrate_records(src_wallet, dest_wallet, migrate_fn).await?;

    info!("Migration completed");

    Ok(())
}

#[cfg(test)]
mod tests {
    use vdrtools::{
        types::domain::wallet::{Config, Credentials, KeyDerivationMethod},
        Locator,
    };

    #[tokio::test]
    #[should_panic]
    async fn test_cant_open_wallet_twice() {
        let wallet_key = "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY".to_owned();
        let wallet_name = "wallet_with_some_name".to_owned();

        let credentials = Credentials {
            key: wallet_key,
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

        let _first_wh = Locator::instance()
            .wallet_controller
            .open(config.clone(), credentials.clone())
            .await
            .unwrap();

        let _second_wh = Locator::instance()
            .wallet_controller
            .open(config, credentials)
            .await
            .unwrap();
    }
}
