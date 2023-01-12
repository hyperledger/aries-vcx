use indy_api_types::{domain::wallet::Tags, errors::*, WalletHandle};

#[cfg(feature = "ffi_api")]
use crate::controllers::vdr::{VDRController, VDR};
use crate::controllers::{cache, cache::get_seconds_since_epoch};
use crate::domain::cache::GetCacheOptions;

const DID_CACHE: &str = "vdr_did_cache";
const CREDDEF_CACHE: &str = "vdr_cred_def_cache";
const SCHEMA_CACHE: &str = "vdr_schema_cache";

#[cfg(feature = "ffi_api")]
impl VDRController {
    pub(crate) async fn resolve_did(&self, vdr: &VDR, id: &str) -> IndyResult<String> {
        trace!("resolve_did >id {:?}", id,);
        let (ledger, _) = vdr.resolve_ledger_for_id(id).await?;

        let request = ledger.build_resolve_did_request(&id).await?;
        let response = ledger.submit_query(&request).await?;
        let response = ledger.parse_resolve_did_response(&response).await?;
        Ok(response)
    }

    pub(crate) async fn resolve_did_with_cache(
        &self,
        vdr: &VDR,
        wallet_handle: WalletHandle,
        id: &str,
        options: &GetCacheOptions,
    ) -> IndyResult<String> {
        trace!(
            "resolve_did_with_cache > wallet_handle {:?} id {:?} options {:?}",
            wallet_handle,
            id,
            options,
        );
        let cache = cache::get_record_from_cache(
            &self.wallet_service,
            wallet_handle,
            &id,
            &options,
            DID_CACHE,
        )
        .await?;

        check_cache!(cache, options);

        let response = self.resolve_did(vdr, &id).await?;

        cache::delete_and_add_record(
            &self.wallet_service,
            wallet_handle,
            options,
            &id,
            &response,
            DID_CACHE,
        )
        .await?;

        Ok(response)
    }

    pub(crate) async fn resolve_schema(&self, vdr: &VDR, id: &str) -> IndyResult<String> {
        trace!("resolve_schema > id {:?}", id,);
        let (ledger, _) = vdr.resolve_ledger_for_id(id).await?;

        let request = ledger.build_resolve_schema_request(&id).await?;
        let response = ledger.submit_query(&request).await?;
        let response = ledger.parse_resolve_schema_response(&response).await?;

        Ok(response)
    }

    pub(crate) async fn resolve_schema_with_cache(
        &self,
        vdr: &VDR,
        wallet_handle: WalletHandle,
        id: &str,
        options: &GetCacheOptions,
    ) -> IndyResult<String> {
        trace!(
            "resolve_schema_with_cache > wallet_handle {:?} id {:?} options {:?}",
            wallet_handle,
            id,
            options,
        );
        let cache = cache::get_record_from_cache(
            &self.wallet_service,
            wallet_handle,
            &id,
            &options,
            SCHEMA_CACHE,
        )
        .await?;

        check_cache!(cache, options);

        let response = self.resolve_schema(vdr, id).await?;

        cache::delete_and_add_record(
            &self.wallet_service,
            wallet_handle,
            options,
            id,
            &response,
            SCHEMA_CACHE,
        )
        .await?;

        Ok(response)
    }

    pub(crate) async fn resolve_creddef(&self, vdr: &VDR, id: &str) -> IndyResult<String> {
        trace!("resolve_creddef > id {:?}", id,);
        let (ledger, _) = vdr.resolve_ledger_for_id(id).await?;

        let request = ledger.build_resolve_cred_def_request(&id).await?;
        let response = ledger.submit_query(&request).await?;
        let response = ledger.parse_resolve_cred_def_response(&response).await?;

        Ok(response)
    }

    pub(crate) async fn resolve_creddef_with_cache(
        &self,
        vdr: &VDR,
        wallet_handle: WalletHandle,
        id: &str,
        options: &GetCacheOptions,
    ) -> IndyResult<String> {
        trace!(
            "resolve_creddef_with_cache > wallet_handle {:?} id {:?} options {:?}",
            wallet_handle,
            id,
            options,
        );
        let cache = cache::get_record_from_cache(
            &self.wallet_service,
            wallet_handle,
            id,
            options,
            CREDDEF_CACHE,
        )
        .await?;

        check_cache!(cache, options);

        let response = self.resolve_creddef(vdr, id).await?;

        cache::delete_and_add_record(
            &self.wallet_service,
            wallet_handle,
            options,
            id,
            &response,
            CREDDEF_CACHE,
        )
        .await?;

        Ok(response)
    }
}
