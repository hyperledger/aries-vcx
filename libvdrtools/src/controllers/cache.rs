use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use indy_api_types::{domain::wallet::Tags, errors::prelude::*, PoolHandle, WalletHandle};
use indy_wallet::{WalletRecord, WalletService};

use crate::{
    domain::{
        anoncreds::{credential_definition::CredentialDefinitionId, schema::SchemaId},
        cache::{GetCacheOptions, PurgeOptions},
        crypto::did::DidValue,
    },
    services::{CryptoService, LedgerService, PoolService},
};

const CRED_DEF_CACHE: &str = "cred_def_cache";
const SCHEMA_CACHE: &str = "schema_cache";

pub struct CacheController {
    crypto_service: Arc<CryptoService>,
    ledger_service: Arc<LedgerService>,
    pool_service: Arc<PoolService>,
    wallet_service: Arc<WalletService>,
}

macro_rules! check_cache {
    ($cache:ident, $options:ident) => {
        if let Some(cache) = $cache {
            let min_fresh = $options.min_fresh.unwrap_or(-1);
            if min_fresh >= 0 {
                let ts = match get_seconds_since_epoch() {
                    Ok(ts) => ts,
                    Err(err) => return Err(err),
                };
                if ts - min_fresh
                    <= cache
                        .get_tags()
                        .unwrap_or(&Tags::new())
                        .get("timestamp")
                        .unwrap_or(&"-1".to_string())
                        .parse()
                        .unwrap_or(-1)
                {
                    return Ok(cache.get_value().unwrap_or("").to_string());
                }
            } else {
                return Ok(cache.get_value().unwrap_or("").to_string());
            }
        };

        if $options.no_update.unwrap_or(false) {
            return Err(IndyError::from(IndyErrorKind::LedgerItemNotFound));
        }
    };
}

impl CacheController {
    pub(crate) fn new(
        crypto_service: Arc<CryptoService>,
        ledger_service: Arc<LedgerService>,
        pool_service: Arc<PoolService>,
        wallet_service: Arc<WalletService>,
    ) -> CacheController {
        CacheController {
            crypto_service,
            ledger_service,
            pool_service,
            wallet_service,
        }
    }

    /// Gets schema json data for specified schema id.
    /// If data is present inside of cache, cached data is returned.
    /// Otherwise data is fetched from the ledger and stored inside of cache for future use.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params

    /// pool_handle: pool handle (created by open_pool_ledger).
    /// wallet_handle: wallet handle (created by open_wallet).
    /// submitter_did: DID of the submitter stored in secured Wallet.
    /// id: identifier of schema.
    /// options_json:
    ///  {
    ///    noCache: (bool, optional, false by default) Skip usage of cache,
    ///    noUpdate: (bool, optional, false by default) Use only cached data, do not try to update.
    ///    noStore: (bool, optional, false by default) Skip storing fresh data if updated,
    ///    minFresh: (int, optional, -1 by default) Return cached data if not older than this many
    /// seconds. -1 means do not check age.  }
    pub async fn get_schema(
        &self,
        pool_handle: PoolHandle,
        wallet_handle: WalletHandle,
        submitter_did: DidValue,
        id: SchemaId,
        options: GetCacheOptions,
    ) -> IndyResult<String> {
        trace!(
            "get_schema > pool_handle {:?} wallet_handle {:?} submitter_did {:?} id {:?} options {:?}",
            pool_handle,
            wallet_handle,
            submitter_did,
            id,
            options
        );

        let cache = get_record_from_cache(&self.wallet_service, wallet_handle, &id.0, &options, SCHEMA_CACHE).await?;

        check_cache!(cache, options);

        let ledger_response = {
            let request_json = {
                self.crypto_service.validate_opt_did(Some(&submitter_did))?;

                self.ledger_service
                    .build_get_schema_request(Some(&submitter_did), &id)?
            };

            let pool_response = self.pool_service.send_tx(pool_handle, &request_json).await?;

            self.ledger_service
                .parse_get_schema_response(&pool_response, id.get_method().as_deref())
        };

        let (schema_id, schema_json) = ledger_response?;

        delete_and_add_record(
            &self.wallet_service,
            wallet_handle,
            &options,
            &schema_id,
            &schema_json,
            SCHEMA_CACHE,
        )
        .await
        .to_indy(IndyErrorKind::InvalidState, "Can't update cache.")?;

        let res = Ok(schema_json);
        trace!("get_schema < {:?}", res);
        res
    }

    /// Gets credential definition json data for specified credential definition id.
    /// If data is present inside of cache, cached data is returned.
    /// Otherwise data is fetched from the ledger and stored inside of cache for future use.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params

    /// pool_handle: pool handle (created by open_pool_ledger).
    /// wallet_handle: wallet handle (created by open_wallet).
    /// submitter_did: DID of the submitter stored in secured Wallet.
    /// id: identifier of credential definition.
    /// options_json:
    ///  {
    ///    forceUpdate: (optional, false by default) Force update of record in cache from the
    /// ledger,  }
    pub async fn get_cred_def(
        &self,
        pool_handle: PoolHandle,
        wallet_handle: WalletHandle,
        submitter_did: DidValue,
        id: CredentialDefinitionId,
        options: GetCacheOptions,
    ) -> IndyResult<String> {
        trace!(
            "get_cred_def > pool_handle {:?} wallet_handle {:?} submitter_did {:?} id {:?} options {:?}",
            pool_handle,
            wallet_handle,
            submitter_did,
            id,
            options
        );

        let cache = get_record_from_cache(&self.wallet_service, wallet_handle, &id.0, &options, CRED_DEF_CACHE).await?;

        check_cache!(cache, options);

        let (cred_def_id, cred_def_json) = self
            ._ledger_get_cred_def_and_parse(pool_handle, Some(&submitter_did), &id)
            .await?;

        delete_and_add_record(
            &self.wallet_service,
            wallet_handle,
            &options,
            &cred_def_id,
            &cred_def_json,
            CRED_DEF_CACHE,
        )
        .await
        .to_indy(IndyErrorKind::InvalidState, "Can't update cache.")?;

        let res = Ok(cred_def_json);
        trace!("get_cred_def < {:?}", res);
        return res;
    }

    /// Purge schema cache.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params

    /// wallet_handle: wallet handle (created by open_wallet).
    /// options_json:
    ///  {
    ///    minFresh: (int, optional, -1 by default) Purge cached data if older than this many
    /// seconds. -1 means purge all.  }
    pub async fn purge_schema_cache(&self, wallet_handle: WalletHandle, options: PurgeOptions) -> IndyResult<()> {
        trace!(
            "purge_schema_cache > wallet_handle {:?} options {:?}",
            wallet_handle,
            options
        );

        let query_json = build_query_json(options.max_age.unwrap_or(-1))?;

        let mut search = self
            .wallet_service
            .search_records(
                wallet_handle,
                SCHEMA_CACHE,
                &query_json,
                &json!({
                    "retrieveType": false,
                    "retrieveValue": false,
                    "retrieveTags": false,
                })
                .to_string(),
            )
            .await?;

        while let Some(record) = search.fetch_next_record().await? {
            self.wallet_service
                .delete_record(wallet_handle, SCHEMA_CACHE, record.get_id())
                .await?;
        }

        let res = Ok(());
        trace!("purge_schema_cache < {:?}", res);
        res
    }

    /// Purge credential definition cache.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params

    /// wallet_handle: wallet handle (created by open_wallet).
    /// options_json:
    ///  {
    ///    minFresh: (int, optional, -1 by default) Purge cached data if older than this many
    /// seconds. -1 means purge all.  }
    pub async fn purge_cred_def_cache(&self, wallet_handle: WalletHandle, options: PurgeOptions) -> IndyResult<()> {
        trace!(
            "purge_cred_def_cache > wallet_handle {:?} options {:?}",
            wallet_handle,
            options
        );

        let query_json = build_query_json(options.max_age.unwrap_or(-1))?;

        let mut search = self
            .wallet_service
            .search_records(
                wallet_handle,
                CRED_DEF_CACHE,
                &query_json,
                &json!({
                    "retrieveType": false,
                    "retrieveValue": false,
                    "retrieveTags": false,
                })
                .to_string(),
            )
            .await?;

        while let Some(record) = search.fetch_next_record().await? {
            self.wallet_service
                .delete_record(wallet_handle, CRED_DEF_CACHE, record.get_id())
                .await?;
        }

        let res = Ok(());
        trace!("purge_cred_def_cache <<< res: ()");
        res
    }

    async fn _ledger_get_cred_def_and_parse(
        &self,
        pool_handle: i32,
        submitter_did: Option<&DidValue>,
        id: &CredentialDefinitionId,
    ) -> IndyResult<(String, String)> {
        self.crypto_service.validate_opt_did(submitter_did)?;

        let request_json = self.ledger_service.build_get_cred_def_request(submitter_did, id)?;

        let pool_response = self.pool_service.send_tx(pool_handle, &request_json).await?;

        let res = self
            .ledger_service
            .parse_get_cred_def_response(&pool_response, id.get_method().as_ref().map(String::as_str))?;

        Ok(res)
    }
}

pub(crate) async fn get_record_from_cache(
    wallet_service: &WalletService,
    wallet_handle: WalletHandle,
    id: &str,
    options: &GetCacheOptions,
    which_cache: &str,
) -> Result<Option<WalletRecord>, IndyError> {
    if options.no_cache.unwrap_or(false) {
        return Ok(None);
    }

    let options = json!({
        "retrieveType": false,
        "retrieveValue": true,
        "retrieveTags": true,
    })
    .to_string();

    match wallet_service
        .get_record(wallet_handle, which_cache, &id, &options)
        .await
    {
        Ok(record) => Ok(Some(record)),
        Err(err) if err.kind() == IndyErrorKind::WalletItemNotFound => Ok(None),
        Err(err) => Err(err),
    }
}

pub(crate) async fn delete_and_add_record(
    wallet_service: &WalletService,
    wallet_handle: WalletHandle,
    options: &GetCacheOptions,
    schema_id: &str,
    schema_json: &str,
    which_cache: &str,
) -> IndyResult<()> {
    if !options.no_store.unwrap_or(false) {
        let mut tags = Tags::new();
        let ts = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(ts) => ts.as_secs() as i32,
            Err(err) => {
                warn!("Cannot get time {:?}", err);
                0
            }
        };
        tags.insert("timestamp".to_string(), ts.to_string());
        let _ignore = wallet_service
            .delete_record(wallet_handle, which_cache, &schema_id)
            .await;
        wallet_service
            .add_record(wallet_handle, which_cache, &schema_id, &schema_json, &tags)
            .await
            .to_indy(IndyErrorKind::InvalidState, "Can't update cache.")?;
    }
    Ok(())
}

pub(crate) fn get_seconds_since_epoch() -> Result<i32, IndyError> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .to_indy(IndyErrorKind::InvalidState, "Can't get system time")?
        .as_secs() as i32;

    Ok(ts)
}

pub(crate) fn build_query_json(max_age: i32) -> Result<String, IndyError> {
    if max_age >= 0 {
        let ts = get_seconds_since_epoch()?;
        Ok(json!({"timestamp": {"$lt": ts - max_age}}).to_string())
    } else {
        Ok("{}".to_string())
    }
}
