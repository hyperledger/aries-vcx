mod indy_ledger;
#[cfg(feature = "cheqd")]
mod cheqd_ledger;
mod ledger;
mod read;
mod write;
mod endorsement;

use indy_ledger::IndyLedger;
#[cfg(feature = "cheqd")]
use cheqd_ledger::CheqdLedger;

use futures::future::join_all;
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
};
use indy_api_types::{errors::*};
use async_std::sync::{Arc, RwLock, Mutex, RwLockReadGuard};

use crate::services::{
    PoolService as IndyPoolService,
    LedgerService as IndyLedgerService,
    WalletService,
    CryptoService,
};
#[cfg(feature = "cheqd")]
use crate::services::{
    CheqdPoolService,
    CheqdLedgerService,
    CheqdKeysService,
};

use crate::domain::{
    vdr::{
        taa_config::TAAConfig,
        ping_status::PingStatus,
        namespaces::Namespaces,
    },
    id::FullyQualifiedId,
};

use ledger::Ledger;

pub(crate) struct VDRBuilder {
    pub(crate) namespaces: HashMap<String, Arc<RwLock<dyn Ledger + 'static>>>,
}

impl VDRBuilder {
    pub fn create() -> VDRBuilder {
        VDRBuilder {
            namespaces: HashMap::new(),
        }
    }

    fn add_ledger(&mut self,
                  namespace_list: Namespaces,
                  ledger: Arc<RwLock<dyn Ledger>>) {
        namespace_list
            .into_iter()
            .for_each(|namespace| {
                self.namespaces.insert(namespace, ledger.clone());
            });
    }

    fn validate_unique_namespaces(&self,
                                  namespace_list: &Namespaces) -> IndyResult<()> {
        match namespace_list.0.iter().find(|key| self.namespaces.contains_key(key.as_str())) {
            Some(namespace) => {
                Err(err_msg(
                    IndyErrorKind::InvalidVDRNamespace,
                    format!("Unable to register namespace \"{}\" as it was already registered.", namespace),
                ))
            }
            None => Ok(())
        }
    }

    pub(crate) fn finalize(&self) -> VDR {
        VDR {
            namespaces: self.namespaces.to_owned()
        }
    }
}


pub(crate) struct VDR {
    namespaces: HashMap<String, Arc<RwLock<dyn Ledger + 'static>>>,
}

impl VDR {
    async fn resolve_ledger_for_namespace<'a>(&'a self,
                                              namespace: &str) -> IndyResult<RwLockReadGuard<'a, dyn Ledger>> {
        trace!(
            "resolve_ledger_for_namespace > namespace {:?}, all registred namespaces {:?}",
            namespace, self.namespaces.keys()
        );
        let ledger = self.namespaces
            .get(namespace)
            .ok_or(err_msg(
                IndyErrorKind::InvalidVDRNamespace,
                format!("Unable to get Ledger with namespace \"{}\" in VDR.", namespace),
            ))?;

        let ledger = ledger.read().await;
        Ok(ledger)
    }

    async fn resolve_ledger_for_id<'a>(&'a self,
                                       id: &str) -> IndyResult<(RwLockReadGuard<'a, dyn Ledger>, FullyQualifiedId)> {
        trace!(
            "resolve_ledger_for_id > id {:?}",
            id
        );
        let parsed_id: FullyQualifiedId = FullyQualifiedId::try_from(id)
            .map_err(|err| err_msg(IndyErrorKind::InvalidStructure, err))?;

        let ledger = self.resolve_ledger_for_namespace(&parsed_id.namespace()).await?;

        if parsed_id.did_method != ledger.ledger_type() {
            return Err(err_msg(
                IndyErrorKind::InvalidVDRHandle,
                format!("Registered Ledger type \"{:?}\" does not match to the network of id \"{:?}\"",
                        ledger.ledger_type(), parsed_id.did_method),
            ));
        }

        Ok((ledger, parsed_id))
    }
}

#[cfg(feature = "cheqd")]
pub(crate) struct VDRController {
    wallet_service: Arc<WalletService>,
    indy_ledger_service: Arc<IndyLedgerService>,
    cheqd_ledger_service: Arc<CheqdLedgerService>,
    indy_pool_service: Arc<IndyPoolService>,
    cheqd_pool_service: Arc<CheqdPoolService>,
    crypto_service: Arc<CryptoService>,
    cheqd_crypto_service: Arc<CheqdKeysService>,
}

#[cfg(not(feature = "cheqd"))]
pub(crate) struct VDRController {
    wallet_service: Arc<WalletService>,
    indy_ledger_service: Arc<IndyLedgerService>,
    indy_pool_service: Arc<IndyPoolService>,
    crypto_service: Arc<CryptoService>,
}

impl VDRController {
    #[cfg(feature = "cheqd")]
    pub(crate) fn new(
        wallet_service: Arc<WalletService>,
        indy_ledger_service: Arc<IndyLedgerService>,
        cheqd_ledger_service: Arc<CheqdLedgerService>,
        indy_pool_service: Arc<IndyPoolService>,
        cheqd_pool_service: Arc<CheqdPoolService>,
        crypto_service: Arc<CryptoService>,
        cheqd_crypto_service: Arc<CheqdKeysService>, ) -> VDRController {
        VDRController {
            wallet_service,
            indy_ledger_service,
            cheqd_ledger_service,
            indy_pool_service,
            cheqd_pool_service,
            crypto_service,
            cheqd_crypto_service,
        }
    }

    #[cfg(not(feature = "cheqd"))]
    pub(crate) fn new(
        wallet_service: Arc<WalletService>,
        indy_ledger_service: Arc<IndyLedgerService>,
        indy_pool_service: Arc<IndyPoolService>,
        crypto_service: Arc<CryptoService>) -> VDRController {
        VDRController {
            wallet_service,
            indy_ledger_service,
            indy_pool_service,
            crypto_service,
        }
    }

    pub(crate) async fn register_indy_ledger(&self,
                                             vdr_builder: Arc<Mutex<VDRBuilder>>,
                                             namespace_list: Namespaces,
                                             genesis_txn: String,
                                             taa_config: Option<TAAConfig>) -> IndyResult<()> {
        let mut vdr_builder = vdr_builder.lock().await;
        vdr_builder.validate_unique_namespaces(&namespace_list)?;

        let ledger = IndyLedger::create(genesis_txn,
                                        taa_config,
                                        self.indy_ledger_service.clone(),
                                        self.indy_pool_service.clone())?;
        let ledger = Arc::new(RwLock::new(ledger));

        vdr_builder.add_ledger(namespace_list, ledger);
        Ok(())
    }

    #[cfg(feature = "cheqd")]
    pub(crate) async fn register_cheqd_ledger(&self,
                                              vdr_builder: Arc<Mutex<VDRBuilder>>,
                                              namespace_list: Namespaces,
                                              chain_id: String,
                                              rpc_address: String) -> IndyResult<()> {
        let mut vdr_builder = vdr_builder.lock().await;
        vdr_builder.validate_unique_namespaces(&namespace_list)?;

        let ledger = CheqdLedger::create(&chain_id,
                                         &rpc_address,
                                         self.cheqd_ledger_service.clone(),
                                         self.cheqd_pool_service.clone()).await?;
        let ledger = Arc::new(RwLock::new(ledger));

        vdr_builder.add_ledger(namespace_list, ledger);
        Ok(())
    }

    pub(crate) async fn ping(&self,
                             vdr: &VDR,
                             namespace_list: Namespaces) -> IndyResult<String> {
        // Group namespaces by Ledger name
        let mut ledgers: HashMap<String, Vec<String>> = HashMap::new();
        for namespace in namespace_list {
            let ledger = vdr.namespaces.get(&namespace)
                .ok_or(err_msg(
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
            futures.push(
                self.query_ledger_status(vdr, namespaces)
            );
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

    async fn query_ledger_status(&self,
                                 vdr: &VDR,
                                 namespaces: Vec<String>) -> IndyResult<(Vec<String>, PingStatus)> {
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

    pub(crate) async fn submit_txn(&self,
                                   vdr: &VDR,
                                   namespace: String,
                                   signature_spec: String,
                                   txn_bytes: Vec<u8>,
                                   signature: Vec<u8>,
                                   endorsement: Option<String>) -> IndyResult<String> {
        trace!(
            "submit_txn > namespace {:?} signature_spec {:?} txn_bytes {:?} signature {:?} endorsement {:?}",
            namespace, signature_spec, txn_bytes, signature, endorsement
        );
        let ledger = vdr.resolve_ledger_for_namespace(&namespace).await?;
        ledger.submit_txn(&txn_bytes, &signature, endorsement.as_deref()).await
    }

    pub(crate) async fn submit_raw_txn(&self,
                                       vdr: &VDR,
                                       namespace: String,
                                       txn_bytes: Vec<u8>) -> IndyResult<String> {
        trace!(
            "submit_raw_txn > namespace {:?} txn_bytes {:?} ",
            namespace, txn_bytes
        );
        let ledger = vdr.resolve_ledger_for_namespace(&namespace).await?;
        ledger.submit_raw_txn(&txn_bytes).await
    }

    pub(crate) async fn submit_query(&self,
                                     vdr: &VDR,
                                     namespace: String,
                                     query: String) -> IndyResult<String> {
        trace!(
            "submit_query > namespace {:?} query {:?} ",
            namespace, query
        );
        let ledger = vdr.resolve_ledger_for_namespace(&namespace).await?;
        ledger.submit_query(&query).await
    }

    pub(crate) async fn cleanup(&self,
                                vdr: &mut VDR) -> IndyResult<()> {
        trace!(
            "cleanup > ",
        );
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
