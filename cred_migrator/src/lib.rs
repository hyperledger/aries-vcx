pub mod credx2anoncreds;
pub mod error;
pub mod vdrtools2credx;

use std::fmt::Display;

use error::MigrationResult;
pub use vdrtools::{
    types::domain::wallet::{Config, Credentials, KeyDerivationMethod, Record},
    Locator, WalletHandle,
};

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
    use std::collections::HashMap;

    use aries_vcx_core::anoncreds::credx_anoncreds::{CATEGORY_CREDENTIAL, CATEGORY_LINK_SECRET};
    use credx::ursa::bn::BigNumber;
    use serde_json::json;
    use vdrtools::WalletRecord;

    use crate::vdrtools2credx::{INDY_CRED, INDY_MASTER_SECRET};

    use super::*;

    macro_rules! add_wallet_item {
        ($wh:expr, $category:expr, $val:expr) => {
            Locator::instance()
                .non_secret_controller
                .add_record(
                    $wh,
                    $category.to_owned(),
                    "test_id".to_owned(),
                    serde_json::to_string(&$val).unwrap(),
                    None,
                )
                .await
                .unwrap();
        };
    }

    async fn get_wallet_item_raw(wallet_handle: WalletHandle, category: &str) -> String {
        let options = r#"{
            "retrieve_type": false,
            "retrieve_value": true,
            "retrieve_tags": false
        }"#;

        let record_str = Locator::instance()
            .non_secret_controller
            .get_record(
                wallet_handle,
                category.to_owned(),
                "test_id".to_owned(),
                options.to_owned(),
            )
            .await
            .unwrap();

        println!("{record_str}");
        let record: WalletRecord = serde_json::from_str(&record_str).unwrap();
        record.get_value().unwrap().to_owned()
    }

    macro_rules! get_wallet_item {
        ($wh:expr, $category:expr, $res:ty) => {{
            let val = get_wallet_item_raw($wh, $category).await;
            serde_json::from_str::<$res>(&val).unwrap()
        }};
    }

    fn make_wallet_reqs(wallet_key: String, wallet_name: String) -> (Credentials, Config) {
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

        (credentials, config)
    }

    fn make_dummy_master_secret() -> vdrtools::MasterSecret {
        let ms_str = json!({
            "value": {
                "ms": "1234567890"
            }
        })
        .to_string();

        serde_json::from_str(&ms_str).unwrap()
    }

    fn make_dummy_cred() -> vdrtools::Credential {
        let cred_sig_str = json!({
            "p_credential": {
                "m_2": "1234567890",
                "a": "1234567890",
                "e": "1234567890",
                "v": "1234567890"
            },
            "r_credential": null
        })
        .to_string();

        let sig_cor_proof_str = json!({
            "se": "1234567890",
            "c": "1234567890"
        })
        .to_string();

        vdrtools::Credential {
            schema_id: vdrtools::SchemaId("test_schema_id".to_owned()),
            cred_def_id: vdrtools::CredentialDefinitionId("test_cred_def_id".to_owned()),
            rev_reg_id: Some(vdrtools::RevocationRegistryId("test_rev_reg_id".to_owned())),
            values: vdrtools::CredentialValues(HashMap::new()),
            signature: serde_json::from_str(&cred_sig_str).unwrap(),
            signature_correctness_proof: serde_json::from_str(&sig_cor_proof_str).unwrap(),
            rev_reg: None,
            witness: None,
        }
    }

    #[tokio::test]
    async fn test_migration() {
        let wallet_key = "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY".to_owned();
        let (credentials, config) = make_wallet_reqs(wallet_key.clone(), "wallet_test_migration".to_owned());

        Locator::instance()
            .wallet_controller
            .delete(config.clone(), credentials.clone())
            .await
            .ok();

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

        add_wallet_item!(wallet_handle, INDY_MASTER_SECRET, make_dummy_master_secret());
        add_wallet_item!(wallet_handle, INDY_CRED, make_dummy_cred());

        let (new_credentials, new_config) = make_wallet_reqs(wallet_key, "new_better_wallet".to_owned());

        Locator::instance()
            .wallet_controller
            .delete(new_config.clone(), new_credentials.clone())
            .await
            .ok();

        migrate_wallet(
            wallet_handle,
            new_config.clone(),
            new_credentials.clone(),
            vdrtools2credx::migrate_any_record,
        )
        .await
        .unwrap();

        Locator::instance()
            .wallet_controller
            .close(wallet_handle)
            .await
            .unwrap();

        Locator::instance()
            .wallet_controller
            .delete(config, credentials)
            .await
            .unwrap();

        let new_wallet_handle = Locator::instance()
            .wallet_controller
            .open(new_config.clone(), new_credentials.clone())
            .await
            .unwrap();

        let ms_decimal = get_wallet_item_raw(new_wallet_handle, CATEGORY_LINK_SECRET).await;
        let ms_bn = BigNumber::from_dec(&ms_decimal).unwrap();

        let ursa_ms: credx::ursa::cl::MasterSecret = serde_json::from_value(json!({ "ms": ms_bn })).unwrap();
        let _ = credx::types::MasterSecret { value: ursa_ms };

        get_wallet_item!(new_wallet_handle, CATEGORY_CREDENTIAL, credx::types::Credential);

        Locator::instance()
            .wallet_controller
            .close(new_wallet_handle)
            .await
            .unwrap();

        Locator::instance()
            .wallet_controller
            .delete(new_config, new_credentials)
            .await
            .unwrap();
    }
}
