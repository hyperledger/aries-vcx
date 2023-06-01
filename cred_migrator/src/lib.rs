use aries_vcx_core::indy::{self, wallet::RestoreWalletConfigs};
use vdrtools::{types::domain::wallet::Record, IndyError, WalletHandle};

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

fn migrate_record(record: Record) -> Result<Record, IndyError> {
    println!("{record:?}");
    Ok(record)
}

pub async fn migrate_wallet(
    wallet_handle: WalletHandle,
    path: &str,
    backup_key: &str,
    wallet_config: &RestoreWalletConfigs,
) -> Result<(), String> {
    indy::wallet::export_wallet(wallet_handle, path, backup_key)
        .await
        .as_ref()
        .map_err(ToString::to_string)?;

    indy::wallet::import_and_migrate(wallet_config, migrate_record)
        .await
        .as_ref()
        .map_err(ToString::to_string)?;

    Ok(())
}