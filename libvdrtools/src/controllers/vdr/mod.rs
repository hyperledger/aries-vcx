mod endorsement;
mod indy_ledger;
mod ledger;
mod read;
mod write;

use indy_ledger::IndyLedger;

use async_std::sync::{Arc, Mutex, RwLock, RwLockReadGuard};
use futures::future::join_all;
use indy_api_types::errors::*;
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
};

use crate::services::{
    CryptoService, LedgerService as IndyLedgerService, PoolService as IndyPoolService,
    WalletService,
};

use crate::domain::{
    id::FullyQualifiedId,
    vdr::{namespaces::Namespaces, ping_status::PingStatus, taa_config::TAAConfig},
};

use ledger::Ledger;

#[cfg(feature = "ffi_api")]
pub(crate) struct VDRBuilder {
    pub(crate) namespaces: HashMap<String, Arc<RwLock<dyn Ledger + 'static>>>,
}

#[cfg(feature = "ffi_api")]
impl VDRBuilder {
    /// Create a Builder object for Verifiable Data Registry which provides a unified interface for interactions with supported Ledgers.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// vdr_builder_p: pointer to store VDRBuilder object
    ///
    /// #Returns
    /// Error Code
    pub fn create() -> VDRBuilder {
        VDRBuilder {
            namespaces: HashMap::new(),
        }
    }

    fn add_ledger(&mut self, namespace_list: Namespaces, ledger: Arc<RwLock<dyn Ledger>>) {
        namespace_list.into_iter().for_each(|namespace| {
            self.namespaces.insert(namespace, ledger.clone());
        });
    }

    fn validate_unique_namespaces(&self, namespace_list: &Namespaces) -> IndyResult<()> {
        match namespace_list
            .0
            .iter()
            .find(|key| self.namespaces.contains_key(key.as_str()))
        {
            Some(namespace) => Err(err_msg(
                IndyErrorKind::InvalidVDRNamespace,
                format!(
                    "Unable to register namespace \"{}\" as it was already registered.",
                    namespace
                ),
            )),
            None => Ok(()),
        }
    }

    /// Finalize building of VDR object and receive a pointer to VDR providing a unified interface for interactions with supported Ledgers.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// vdr_builder: pointer to VDRBuilder object
    /// vdr_p: pointer to store VDR object
    ///
    /// #Returns
    /// Error Code
    pub(crate) fn finalize(&self) -> VDR {
        VDR {
            namespaces: self.namespaces.to_owned(),
        }
    }
}

#[cfg(feature = "ffi_api")]
pub(crate) struct VDR {
    namespaces: HashMap<String, Arc<RwLock<dyn Ledger + 'static>>>,
}

#[cfg(feature = "ffi_api")]
impl VDR {
    async fn resolve_ledger_for_namespace<'a>(
        &'a self,
        namespace: &str,
    ) -> IndyResult<RwLockReadGuard<'a, dyn Ledger>> {
        trace!(
            "resolve_ledger_for_namespace > namespace {:?}, all registred namespaces {:?}",
            namespace,
            self.namespaces.keys()
        );
        let ledger = self.namespaces.get(namespace).ok_or(err_msg(
            IndyErrorKind::InvalidVDRNamespace,
            format!(
                "Unable to get Ledger with namespace \"{}\" in VDR.",
                namespace
            ),
        ))?;

        let ledger = ledger.read().await;
        Ok(ledger)
    }

    async fn resolve_ledger_for_id<'a>(
        &'a self,
        id: &str,
    ) -> IndyResult<(RwLockReadGuard<'a, dyn Ledger>, FullyQualifiedId)> {
        trace!("resolve_ledger_for_id > id {:?}", id);
        let parsed_id: FullyQualifiedId = FullyQualifiedId::try_from(id)
            .map_err(|err| err_msg(IndyErrorKind::InvalidStructure, err))?;

        let ledger = self
            .resolve_ledger_for_namespace(&parsed_id.namespace())
            .await?;

        if parsed_id.did_method != ledger.ledger_type() {
            return Err(err_msg(
                IndyErrorKind::InvalidVDRHandle,
                format!(
                    "Registered Ledger type \"{:?}\" does not match to the network of id \"{:?}\"",
                    ledger.ledger_type(),
                    parsed_id.did_method
                ),
            ));
        }

        Ok((ledger, parsed_id))
    }
}

#[cfg(feature = "ffi_api")]
pub struct VDRController {
    wallet_service: Arc<WalletService>,
    indy_ledger_service: Arc<IndyLedgerService>,
    indy_pool_service: Arc<IndyPoolService>,
    crypto_service: Arc<CryptoService>,
}

#[cfg(feature = "ffi_api")]
impl VDRController {
    pub(crate) fn new(
        wallet_service: Arc<WalletService>,
        indy_ledger_service: Arc<IndyLedgerService>,
        indy_pool_service: Arc<IndyPoolService>,
        crypto_service: Arc<CryptoService>,
    ) -> VDRController {
        VDRController {
            wallet_service,
            indy_ledger_service,
            indy_pool_service,
            crypto_service,
        }
    }

    /// Register Indy Ledger in the VDR object.
    /// Associate registered Indy Ledger with the list of specified namespaces that will be used for
    /// the resolution of public entities referencing by fully qualified identifiers.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// vdr_builder: pointer to VDRBuilder object
    /// namespace_list: list of namespaces to associated with Ledger ('["namespace_1", "namespace_2"]')
    /// genesis_txn_data: genesis transactions for Indy Ledger (Note that node transactions must be located in separate lines)
    /// taa_config: accepted transaction author agreement data:
    ///     {
    ///         text and version - (optional) raw data about TAA from ledger.
    ///                             These parameters should be passed together.
    ///                             These parameters are required if taa_digest parameter is omitted.
    ///         taa_digest - (optional) digest on text and version.
    ///                             Digest is sha256 hash calculated on concatenated strings: version || text.
    ///                             This parameter is required if text and version parameters are omitted.
    ///         acc_mech_type - mechanism how user has accepted the TAA
    ///         time - UTC timestamp when user has accepted the TAA. Note that the time portion will be discarded to avoid a privacy risk.
    ///     }
    ///
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Error Code
    /// cb:
    /// - command_handle_: com
    pub(crate) async fn register_indy_ledger(
        &self,
        vdr_builder: Arc<Mutex<VDRBuilder>>,
        namespace_list: Namespaces,
        genesis_txn: String,
        taa_config: Option<TAAConfig>,
    ) -> IndyResult<()> {
        let mut vdr_builder = vdr_builder.lock().await;
        vdr_builder.validate_unique_namespaces(&namespace_list)?;

        let ledger = IndyLedger::create(
            genesis_txn,
            taa_config,
            self.indy_ledger_service.clone(),
            self.indy_pool_service.clone(),
        )?;
        let ledger = Arc::new(RwLock::new(ledger));

        vdr_builder.add_ledger(namespace_list, ledger);
        Ok(())
    }

    /// Ping Ledgers registered in the VDR.
    ///
    /// NOTE: This function MUST be called for Indy Ledgers before sending any request.
    ///
    /// Indy Ledger: The function performs sync with the ledger and returns the most recent nodes state.
    /// Cheqd Ledger: The function query network information.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// vdr: pointer to VDR object
    /// namespace_list: list of namespaces to ping
    /// cb: Callback that takes command result as parameter
    ///
    /// #Returns
    /// Error Code
    /// cb:
    /// - command_handle_: command handle to map callback to caller context.
    /// - err: Error code.
    pub(crate) async fn ping(&self, vdr: &VDR, namespace_list: Namespaces) -> IndyResult<String> {
        // Group namespaces by Ledger name
        let mut ledgers: HashMap<String, Vec<String>> = HashMap::new();
        for namespace in namespace_list {
            let ledger = vdr.namespaces.get(&namespace).ok_or(err_msg(
                IndyErrorKind::InvalidVDRNamespace,
                format!("Unable to get Ledger with namespace \"{}\".", namespace),
            ))?;

            let ledger = ledger.read().await;
            let networks = ledgers.entry(ledger.name()).or_insert(Vec::new());
            networks.push(namespace.to_string());
        }

        // Ping Ledgers
        let mut futures = Vec::new();
        for (_, namespaces) in ledgers.into_iter() {
            futures.push(self.query_ledger_status(vdr, namespaces));
        }
        let statuses = join_all(futures).await;

        let mut status_list: HashMap<String, PingStatus> = HashMap::new();
        for result in statuses.into_iter() {
            let (namespaces, status) = result?;
            for namespace in namespaces.into_iter() {
                status_list.insert(namespace.to_string(), status.clone());
            }
        }

        json_string_result!(status_list)
    }

    async fn query_ledger_status(
        &self,
        vdr: &VDR,
        namespaces: Vec<String>,
    ) -> IndyResult<(Vec<String>, PingStatus)> {
        let ledger = namespaces
            .get(0)
            .and_then(|namespace| vdr.namespaces.get(namespace.as_str()))
            .ok_or(err_msg(
                IndyErrorKind::InvalidVDRNamespace,
                format!("Unable to get Ledger with namespace \"{:?}\".", namespaces),
            ))?;

        let ledger = ledger.read().await;
        let status = ledger.ping().await?;
        Ok((namespaces, status))
    }

    /// Submit transaction to the Ledger.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// vdr: pointer to VDR object
    /// namespace of the registered Ledger to submit transaction
    /// txn_bytes_raw: a pointer to first byte of transaction
    /// txn_bytes_len: a transaction length
    /// signature_spec: type of the signature used for transaction signing
    /// signature_raw: a pointer to first byte of the transaction signature
    /// signatures_len: a transaction signature length
    /// endorsement: (Optional) transaction endorsement data (depends on the ledger type)
    ///     Indy:
    ///         {
    ///             "signature" - endorser signature as base58 string
    ///         }
    ///     Cheqd: TODO
    /// cb: Callback that takes command result as parameter
    ///
    /// #Returns
    /// Error Code
    /// cb:
    /// - command_handle_: command handle to map callback to caller context.
    /// - err: Error code.
    /// - response: received response
    pub(crate) async fn submit_txn(
        &self,
        vdr: &VDR,
        namespace: String,
        signature_spec: String,
        txn_bytes: Vec<u8>,
        signature: Vec<u8>,
        endorsement: Option<String>,
    ) -> IndyResult<String> {
        trace!(
            "submit_txn > namespace {:?} signature_spec {:?} txn_bytes {:?} signature {:?} endorsement {:?}",
            namespace, signature_spec, txn_bytes, signature, endorsement
        );
        let ledger = vdr.resolve_ledger_for_namespace(&namespace).await?;
        ledger
            .submit_txn(&txn_bytes, &signature, endorsement.as_deref())
            .await
    }

    /// Submit raw transaction to the Ledger.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// vdr: pointer to VDR object
    /// namespace of the registered Ledger to submit transaction
    /// txn_bytes_raw: a pointer to first byte of transaction
    /// txn_bytes_len: a transaction length
    /// cb: Callback that takes command result as parameter
    ///
    /// #Returns
    /// Error Code
    /// cb:
    /// - command_handle_: command handle to map callback to caller context.
    /// - err: Error code.
    /// - response: received response

    pub(crate) async fn submit_raw_txn(
        &self,
        vdr: &VDR,
        namespace: String,
        txn_bytes: Vec<u8>,
    ) -> IndyResult<String> {
        trace!(
            "submit_raw_txn > namespace {:?} txn_bytes {:?} ",
            namespace,
            txn_bytes
        );
        let ledger = vdr.resolve_ledger_for_namespace(&namespace).await?;
        ledger.submit_raw_txn(&txn_bytes).await
    }

    /// Submit query to the Ledger.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// vdr: pointer to VDR object
    /// namespace of the registered Ledger to submit transaction
    /// query: query message to submit on the Ledger
    /// cb: Callback that takes command result as parameter
    ///
    /// #Returns
    /// Error Code
    /// cb:
    /// - command_handle_: command handle to map callback to caller context.
    /// - err: Error code.
    /// - response: received response
    pub(crate) async fn submit_query(
        &self,
        vdr: &VDR,
        namespace: String,
        query: String,
    ) -> IndyResult<String> {
        trace!(
            "submit_query > namespace {:?} query {:?} ",
            namespace,
            query
        );
        let ledger = vdr.resolve_ledger_for_namespace(&namespace).await?;
        ledger.submit_query(&query).await
    }

    /// Drop VDR object and associated Ledger connections.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// vdr: pointer to VDR object
    /// cb: Callback that takes command result as parameter
    ///
    /// #Returns
    /// Error Code
    /// cb:
    /// - command_handle_: command handle to map callback to caller context.
    /// - err: Error code.

    pub(crate) async fn cleanup(&self, vdr: &mut VDR) -> IndyResult<()> {
        trace!("cleanup > ",);
        let mut visited_ledgers: HashSet<String> = HashSet::new();

        for (_, ledger) in vdr.namespaces.iter() {
            let ledger = ledger.read().await;
            let name = ledger.name();

            if !visited_ledgers.contains(&name) {
                visited_ledgers.insert(name);
                ledger.cleanup().await?;
            }
        }

        Ok(())
    }
}
