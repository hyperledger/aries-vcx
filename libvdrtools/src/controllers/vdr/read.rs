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
    /// Resolve DID information for specified fully-qualified DID.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// vdr: pointer to VDR object
    /// fqdid: fully-qualified DID of the target DID on the Ledger
    ///
    /// #Returns
    /// diddoc: Resolved DID information.
    ///         Note that the format of the value depends on the Ledger type:
    ///   Indy:    {
    ///           "did": string
    ///           "verkey": string
    ///       }
    pub(crate) async fn resolve_did(&self, vdr: &VDR, id: &str) -> IndyResult<String> {
        trace!("resolve_did >id {:?}", id,);
        let (ledger, _) = vdr.resolve_ledger_for_id(id).await?;

        let request = ledger.build_resolve_did_request(&id).await?;
        let response = ledger.submit_query(&request).await?;
        let response = ledger.parse_resolve_did_response(&response).await?;
        Ok(response)
    }

    /// Resolve DID information for specified fully-qualified DID with using of wallet cache.
    ///
    /// If data is present inside of wallet cache, cached data is returned.
    /// Otherwise data is fetched from the associated Ledger and stored inside of cache for future use.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// vdr: pointer to VDR object
    /// wallet_handle: handle pointing to an opened wallet (returned by indy_open_wallet)
    /// fqdid: fully-qualified DID of the target DID on the Ledger
    /// cache_options: caching options
    ///     {
    ///         forceUpdate: (optional, false by default) Force update of record in cache from the ledger,
    ///     }
    ///
    /// #Returns
    /// Error Code
    /// diddoc: Resolved DID information.
    ///         Note that the format of the value depends on the Ledger type:
    ///   Indy:    {
    ///           "did": string
    ///           "verkey": string
    ///       }
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

    /// Resolve Schema for specified fully-qualified ID.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// vdr: pointer to VDR object
    /// fqschema: fully-qualified Schema ID of the target Schema on the Ledger
    ///
    /// #Returns
    /// schema: Resolved Schema
    ///   {
    ///       id: identifier of schema
    ///       attrNames: array of attribute name strings
    ///       name: Schema's name string
    ///       version: Schema's version string
    ///       ver: Version of the Schema json
    ///   }
    pub(crate) async fn resolve_schema(&self, vdr: &VDR, id: &str) -> IndyResult<String> {
        trace!("resolve_schema > id {:?}", id,);
        let (ledger, _) = vdr.resolve_ledger_for_id(id).await?;

        let request = ledger.build_resolve_schema_request(&id).await?;
        let response = ledger.submit_query(&request).await?;
        let response = ledger.parse_resolve_schema_response(&response).await?;

        Ok(response)
    }

    /// Resolve Schema for specified fully-qualified ID with using of wallet cache.
    ///
    /// If data is present inside of wallet cache, cached data is returned.
    /// Otherwise data is fetched from the associated Ledger and stored inside of cache for future use.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// vdr: pointer to VDR object
    /// wallet_handle: handle pointing to an opened wallet (returned by indy_open_wallet)
    /// fqschema: fully-qualified Schema ID of the target Schema on the Ledger
    /// cache_options: caching options
    ///     {
    ///         forceUpdate: (optional, false by default) Force update of record in cache from the ledger,
    ///     }
    ///
    /// #Returns
    /// schema: Resolved Schema
    ///   {
    ///       id: identifier of schema
    ///       attrNames: array of attribute name strings
    ///       name: Schema's name string
    ///       version: Schema's version string
    ///       ver: Version of the Schema json
    ///   }
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

    /// Resolve Credential Definition for specified fully-qualified ID.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// vdr: pointer to VDR object
    /// fqcreddef: fully-qualified CredDef ID of the target CredentialDefinition on the Ledger
    ///
    /// #Returns
    /// credential_definition: Resolved Credential Definition
    ///   {
    ///       id: string - identifier of credential definition
    ///       schemaId: string - identifier of stored in ledger schema
    ///       type: string - type of the credential definition. CL is the only supported type now.
    ///       tag: string - allows to distinct between credential definitions for the same issuer and schema
    ///       value: Dictionary with Credential Definition's data: {
    ///           primary: primary credential public key,
    ///           Optional<revocation>: revocation credential public key
    ///       },
    ///       ver: Version of the Credential Definition json
    ///   }
    pub(crate) async fn resolve_creddef(&self, vdr: &VDR, id: &str) -> IndyResult<String> {
        trace!("resolve_creddef > id {:?}", id,);
        let (ledger, _) = vdr.resolve_ledger_for_id(id).await?;

        let request = ledger.build_resolve_cred_def_request(&id).await?;
        let response = ledger.submit_query(&request).await?;
        let response = ledger.parse_resolve_cred_def_response(&response).await?;

        Ok(response)
    }

    /// Resolve Credential Definition for specified fully-qualified ID with using of wallet cache.
    ///
    /// If data is present inside of wallet cache, cached data is returned.
    /// Otherwise data is fetched from the associated Ledger and stored inside of cache for future use.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// vdr: pointer to VDR object
    /// wallet_handle: handle pointing to an opened wallet (returned by indy_open_wallet)
    /// fqcreddef: fully-qualified CredDef ID of the target CredentialDefinition on the Ledger
    /// cache_options: caching options
    ///     {
    ///         forceUpdate: (optional, false by default) Force update of record in cache from the ledger,
    ///     }
    ///
    /// #Returns
    /// credential_definition: Resolved Credential Definition
    ///   {
    ///       id: string - identifier of credential definition
    ///       schemaId: string - identifier of stored in ledger schema
    ///       type: string - type of the credential definition. CL is the only supported type now.
    ///       tag: string - allows to distinct between credential definitions for the same issuer and schema
    ///       value: Dictionary with Credential Definition's data: {
    ///           primary: primary credential public key,
    ///           Optional<revocation>: revocation credential public key
    ///       },
    ///       ver: Version of the Credential Definition json
    ///   }
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
