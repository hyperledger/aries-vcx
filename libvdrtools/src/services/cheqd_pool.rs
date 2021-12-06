//! Pool service for Tendermint back-end

use std::fs;
use std::io::Write;

use std::collections::HashMap;
use async_std::sync::RwLock;

use http_client::HttpClient;
use http_client::http_types::{Method,
                              Request as HttpRequest,
                              Response as HttpResponse,
                              Body};
use http_client::h1::H1Client;

use cosmrs::rpc;
use cosmrs::rpc::{Request, Response};
use cosmrs::rpc::endpoint::broadcast;
use cosmrs::tendermint::abci;
use cosmrs::tx::Raw;
use indy_api_types::errors::{IndyErrorKind, IndyResult, IndyResultExt};
use indy_api_types::errors::*;
use indy_api_types::IndyError;

use crate::domain::cheqd_pool::{PoolConfig, AddPoolConfig, PoolMode};
use crate::utils::environment;

pub(crate) struct CheqdPoolService {
    // store in-memory pools
    configs: RwLock<HashMap<String, PoolConfig>>,
}

// ToDo: it can be used only in "unstable-config".
// const CLIENT_TIMEOUT: u64 = 10;

impl CheqdPoolService {
    pub(crate) fn new() -> Self {
        Self { configs: RwLock::new(HashMap::new()) }
    }

    pub(crate) async fn add(
        &self,
        alias: &str,
        config: &AddPoolConfig,
    ) -> IndyResult<PoolConfig> {
        if self.get_config(alias).await.is_ok() {
            return Err(err_msg(
                IndyErrorKind::PoolConfigAlreadyExists,
                format!("Cheqd pool ledger config with alias \"{}\" already exists", alias),
            ));
        }

        let pool_config = PoolConfig::new(
            alias.to_string(),
            config.rpc_address.to_string(),
            config.chain_id.to_string(),
        );

        match config.pool_mode {
            PoolMode::InMemory => self.store_config_in_memory(alias, &pool_config).await?,
            PoolMode::Persistent => self.store_config_on_disc(alias, &pool_config)?
        }

        Ok(pool_config)
    }

    async fn store_config_in_memory(&self, alias: &str, config: &PoolConfig) -> IndyResult<()> {
        let mut configs = self.configs.write().await;
        configs.insert(alias.to_string(), config.clone());
        Ok(())
    }

    fn store_config_on_disc(&self, alias: &str, config: &PoolConfig) -> IndyResult<()> {
        let config_json = json_string!(config);

        let mut path = environment::cheqd_pool_path(alias);

        fs::create_dir_all(path.as_path())
            .to_indy(IndyErrorKind::IOError, "Can't create cheqd pool config directory")?;

        path.push("config");
        path.set_extension("json");

        let mut f: fs::File = fs::File::create(path.as_path())
            .to_indy(IndyErrorKind::IOError, "Can't create cheqd pool config file")?;

        f.write_all(config_json.as_bytes())
            .to_indy(IndyErrorKind::IOError, "Can't write to cheqd pool config file")?;

        f.flush()
            .to_indy(IndyErrorKind::IOError, "Can't write to cheqd pool config file")?;

        Ok(())
    }

    pub(crate) async fn get_config(&self, alias: &str) -> IndyResult<PoolConfig> {
        // check in-memory configs
        let configs = self.configs.read().await;
        if let Some(config) = configs.get(alias) {
            return Ok(config.clone())
        }

        // check disc configs
        let mut path = environment::cheqd_pool_path(alias);

        if !path.exists() {
            return Err(IndyError::from_msg(
                IndyErrorKind::IOError,
                format!("Can't find cheqd pool config file: {}", alias))
            );
        }

        path.push("config");
        path.set_extension("json");

        let config = fs::read_to_string(path)
            .to_indy(IndyErrorKind::IOError, format!("Can't open cheqd pool config file: {}", alias))?;

        let result: PoolConfig = serde_json::from_str(&config)
            .to_indy(IndyErrorKind::IOError, "Invalid data of cheqd pool config file")?;

        Ok(result)
    }

    pub(crate) async fn get_all_config(&self) -> IndyResult<Vec<PoolConfig>> {
        let mut result = Vec::<PoolConfig>::new();

        // add in-memory pools
        let config = self.configs.read().await;
        for (_, cfg) in config.iter() {
            result.push(cfg.clone());
        }

        // add persistent pools
        let path = environment::cheqd_pool_home_path();
        let path_name = path.to_str();

        if !path.exists() {
            let error_msg = "Can't find cheqd pool config files";
            warn!("{}", error_msg);
            return Err(IndyError::from_msg(IndyErrorKind::IOError, error_msg));
        }

        let paths = fs::read_dir(path.clone())?;

        let mut result = Vec::<PoolConfig>::new();
        for dir in paths {
            let mut path = match dir {
                Ok(t) => t.path(),
                Err(error) => {
                    warn!("While iterating over {:?} directory the next error was caught:", path_name);
                    warn!("{}", error);

                    continue;
                },
            };

            path.push("config");
            path.set_extension("json");

            let config = match fs::read_to_string(path.clone()){
                Ok(conf) => conf,
                Err(error) => {
                    warn!("Can't find cheqd pool config file in directory: {:?}", path.clone());
                    warn!("{}", error);

                    continue;
                },
            };

            let pool_config : PoolConfig = match serde_json::from_str(&config){
                Ok(pool_conf) => pool_conf,
                Err(error) => {
                    warn!("Invalid cheqd pool config file in directory: {:?}", path);
                    warn!("{}", error);

                    continue;
                },
            };
            result.push(pool_config);
        }

        Ok(result)
    }

    // Send and wait for commit
    pub(crate) async fn broadcast_tx_commit(
        &self,
        pool_alias: &str,
        tx: &[u8],
    ) -> IndyResult<rpc::endpoint::broadcast::tx_commit::Response> {
        let pool = self.get_config(pool_alias).await?;

        let tx = Raw::from_bytes(tx)?;
        let tx_bytes = tx.to_bytes()?;
        let req = broadcast::tx_commit::Request::new(tx_bytes.into());
        let resp = self.send_req(req, &pool.rpc_address).await?;

        if let abci::Code::Err(_) = resp.check_tx.code {
            return Err(IndyError::from(resp.check_tx));
        }

        if let abci::Code::Err(_) = resp.deliver_tx.code {
            return Err(IndyError::from(resp.deliver_tx));
        }

        Ok(resp)
    }

    pub(crate) async fn abci_query(
        &self,
        pool_alias: &str,
        req: &str,
    ) -> IndyResult<String> {
        let pool = self.get_config(pool_alias).await?;

        let req: rpc::endpoint::abci_query::Request = serde_json::from_str(req).to_indy(
            IndyErrorKind::InvalidStructure,
            "Cannot deserialize string of ABCI Response object",
        )?;

        let resp = self.send_req(req, pool.rpc_address.as_str()).await?;
        Ok(json!(resp).to_string())
    }

    pub(crate) async fn abci_info(
        &self,
        pool_alias: &str,
    ) -> IndyResult<String> {
        let pool = self.get_config(pool_alias).await?;
        let req = rpc::endpoint::abci_info::Request {};
        let resp = self.send_req(req, pool.rpc_address.as_str()).await?;
        let resp = json!(resp).to_string();
        Ok(resp)
    }

    async fn send_req<R>(&self, req: R, rpc_address: &str) -> IndyResult<R::Response>
        where
            R: Request,
    {
        let req_json = req.into_json();
        let mut req = HttpRequest::new(Method::Post,
                                       rpc_address);
        req.append_header("Content-Type", "application/json");
        req.append_header("User-Agent", format!("indy-sdk/{}", env!("CARGO_PKG_VERSION")));
        req.set_body(Body::from_string(req_json));

        let client = H1Client::new();
        // ToDo: it can be changed only in "unstable-config".
        // let config = client.config();
        // config.timeout(std::time::Duration::from_secs(CLIENT_TIMEOUT));
        // client.set_config(config);

        let mut resp: HttpResponse = client.send(req).await?;
        let resp_str = resp.body_string().await?;
        let resp = R::Response::from_string(resp_str).to_indy(
            IndyErrorKind::InvalidStructure,
            "Error was raised while converting tendermint_rpc::request::Request into string",
        )?;

        Ok(resp)
    }
}

#[cfg(test)]
mod send_req {
    use crate::CheqdPoolService;
    use cosmrs::rpc::endpoint::abci_info::Request;

    #[async_std::test]
    async fn client_close_if_connection_refused() {
        let pool_service = CheqdPoolService::new();
        let req = Request {};
        pool_service.send_req(req, "http://127.0.0.1:12345").await.map_err(|err| {
            assert!(err.to_string().contains("Connection refused"))
        }).unwrap_err();
    }
}
