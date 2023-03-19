use std::sync::Arc;

use indy_api_types::{errors::prelude::*, PoolHandle};

use crate::{
    domain::{
        ledger::request::ProtocolVersion,
        pool::{PoolConfig, PoolOpenConfig},
    },
    services::PoolService,
};

pub struct PoolController {
    pool_service: Arc<PoolService>,
}

impl PoolController {
    pub fn new(pool_service: Arc<PoolService>) -> PoolController {
        PoolController { pool_service }
    }

    /// Creates a new local pool ledger configuration that can be used later to connect pool nodes.
    ///
    /// #Params
    /// config_name: Name of the pool ledger configuration.
    /// config (optional): Pool configuration json. if NULL, then default config will be used.
    /// Example: {
    ///     "genesis_txn": string (optional), A path to genesis transaction file. If NULL, then a
    /// default one will be used.                    If file doesn't exists default one will be
    /// created. }
    ///
    /// #Returns
    /// Error code
    ///
    /// #Errors
    /// Common*
    /// Ledger*
    pub fn create(&self, name: String, config: Option<PoolConfig>) -> IndyResult<()> {
        trace!("create > name {:?} config {:?}", name, config);

        self.pool_service.create(&name, config)?;

        let res = Ok(());
        trace!("create < {:?}", res);
        res
    }

    /// Deletes created pool ledger configuration.
    ///
    /// #Params
    /// config_name: Name of the pool ledger configuration to delete.
    ///
    /// #Returns
    /// Error code
    ///
    /// #Errors
    /// Common*
    /// Ledger*
    pub async fn delete(&self, name: String) -> IndyResult<()> {
        trace!("delete > name {:?}", name);

        self.pool_service.delete(&name).await?;

        let res = Ok(());
        trace!("delete < {:?}", res);
        res
    }

    /// Opens pool ledger and performs connecting to pool nodes.
    ///
    /// Pool ledger configuration with corresponded name must be previously created
    /// with indy_create_pool_ledger_config method.
    /// It is impossible to open pool with the same name more than once.
    ///
    /// config_name: Name of the pool ledger configuration.
    /// config (optional): Runtime pool configuration json.
    ///                         if NULL, then default config will be used. Example:
    /// {
    ///     "timeout": int (optional), timeout for network request (in sec).
    ///     "extended_timeout": int (optional), extended timeout for network request (in sec).
    ///     "preordered_nodes": array<string> -  (optional), names of nodes which will have a
    /// priority during request sending:         ["name_of_1st_prior_node",
    /// "name_of_2nd_prior_node", .... ]         This can be useful if a user prefers querying
    /// specific nodes.         Assume that `Node1` and `Node2` nodes reply faster.
    ///         If you pass them Libindy always sends a read request to these nodes first and only
    /// then (if not enough) to others.         Note: Nodes not specified will be placed
    /// randomly.     "number_read_nodes": int (optional) - the number of nodes to send read
    /// requests (2 by default)         By default Libindy sends a read requests to 2 nodes in
    /// the pool.         If response isn't received or `state proof` is invalid Libindy sends
    /// the request again but to 2 (`number_read_nodes`) * 2 = 4 nodes and so far until completion.
    ///     "pool_mode": mode of pool to be used:
    ///         InMemory - pool will be stored in-memory
    ///         Persistent - pool will be persisted in file (default mode)
    ///     "transactions": string (optional) - string with genesis transactions. Must be present if
    /// 'InMemory' pool mode is selected.
    ///
    /// }
    ///
    /// #Returns
    /// Handle to opened pool to use in methods that require pool connection.
    ///
    /// #Errors
    /// Common*
    /// Ledger*
    pub async fn open(&self, name: String, config: Option<PoolOpenConfig>) -> IndyResult<PoolHandle> {
        trace!("open > name {:?} config {:?}", name, config);

        let (handle, _) = self.pool_service.open(name, config, None).await?;

        let res = Ok(handle);
        trace!("open < {:?}", res);
        res
    }

    /// Lists names of created pool ledgers
    ///
    /// #Params
    ///
    /// #Returns
    /// Error code
    ///
    /// #Errors
    pub fn list(&self) -> IndyResult<String> {
        trace!("list > ");

        let pools = self.pool_service.list()?;

        let pools = serde_json::to_string(&pools).to_indy(IndyErrorKind::InvalidState, "Can't serialize pools list")?;

        let res = Ok(pools);
        trace!("list < {:?}", res);
        res
    }

    /// Closes opened pool ledger, opened nodes connections and frees allocated resources.
    ///
    /// #Params
    /// handle: pool handle returned by indy_open_pool_ledger.
    ///
    /// #Returns
    /// Error code
    ///
    /// #Errors
    /// Common*
    /// Ledger*
    pub async fn close(&self, pool_handle: PoolHandle) -> IndyResult<()> {
        trace!("close > handle {:?}", pool_handle);

        self.pool_service.close(pool_handle).await?;

        let res = Ok(());
        trace!("close < {:?}", res);
        res
    }

    /// Refreshes a local copy of a pool ledger and updates pool nodes connections.
    ///
    /// #Params
    /// handle: pool handle returned by indy_open_pool_ledger
    ///
    /// #Returns
    /// Error code
    ///
    /// #Errors
    /// Common*
    /// Ledger*
    pub async fn refresh(&self, handle: PoolHandle) -> IndyResult<()> {
        trace!("refresh > handle {:?}", handle);

        self.pool_service.refresh(handle).await?;

        let res = Ok(());
        trace!("refresh < {:?}", res);
        res
    }

    /// Set PROTOCOL_VERSION to specific version.
    ///
    /// There is a global property PROTOCOL_VERSION that used in every request to the pool and
    /// specified version of Indy Node which Libindy works.
    ///
    /// By default PROTOCOL_VERSION=1.
    ///
    /// #Params
    /// protocol_version: Protocol version will be used:
    ///     1 - for Indy Node 1.3
    ///     2 - for Indy Node 1.4 and greater
    ///
    /// #Returns
    /// Error code
    ///
    /// #Errors
    /// Common*
    pub fn set_protocol_version(&self, version: usize) -> IndyResult<()> {
        trace!("set_protocol_version > version {:?}", version);

        if version != 1 && version != 2 {
            Err(err_msg(
                IndyErrorKind::PoolIncompatibleProtocolVersion,
                format!("Unsupported Protocol version: {}", version),
            ))?;
        }

        ProtocolVersion::set(version);

        let res = Ok(());
        trace!("set_protocol_version < {:?}", res);
        res
    }
}
