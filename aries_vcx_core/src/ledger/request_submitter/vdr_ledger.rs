use std::{
    collections::{hash_map::RandomState, HashMap},
    fmt::{Debug, Formatter},
    sync::Arc,
};

use async_trait::async_trait;
use indy_vdr::{
    common::error::{VdrError, VdrResult},
    config::PoolConfig,
    pool::{
        PoolBuilder, PoolRunner, PoolTransactions, PreparedRequest, RequestResult, TimingResult,
    },
    resolver::types::Callback,
};
use tokio::sync::oneshot;

use super::RequestSubmitter;
use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};

#[derive(Clone)]
pub struct IndyVdrLedgerPool {
    runner: Arc<PoolRunner>,
}

impl IndyVdrLedgerPool {
    pub fn new_from_runner(runner: PoolRunner) -> Self {
        IndyVdrLedgerPool {
            runner: Arc::new(runner),
        }
    }

    fn generate_exclusion_weights(exclude_nodes: Vec<String>) -> HashMap<String, f32> {
        exclude_nodes
            .into_iter()
            .map(|node| (node, 0.0f32))
            .collect()
    }

    pub fn new(
        genesis_file_path: String,
        indy_vdr_config: PoolConfig,
        exclude_nodes: Vec<String>,
    ) -> VcxCoreResult<Self> {
        info!(
            "IndyVdrLedgerPool::new >> genesis_file_path: {genesis_file_path}, indy_vdr_config: \
             {indy_vdr_config:?}"
        );
        let txns = PoolTransactions::from_json_file(genesis_file_path)?;
        let runner = PoolBuilder::new(
            indy_vdr_config,
            None,
            Some(Self::generate_exclusion_weights(exclude_nodes)),
        )
        .transactions(txns)?
        .into_runner()?;

        Ok(IndyVdrLedgerPool {
            runner: Arc::new(runner),
        })
    }
}

impl Debug for IndyVdrLedgerPool {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndyVdrLedgerPool")
            .field("runner", &"PoolRunner")
            .finish()
    }
}

#[derive(Clone)]
pub struct IndyVdrSubmitter {
    pool: IndyVdrLedgerPool,
}

impl IndyVdrSubmitter {
    pub fn new(pool: IndyVdrLedgerPool) -> Self {
        Self { pool }
    }

    pub(crate) fn send_request(
        &self,
        request: PreparedRequest,
        callback: Callback<VdrResult<(RequestResult<String>, Option<TimingResult>)>>,
    ) -> VcxCoreResult<()> {
        self.pool
            .runner
            .send_request(request, callback)
            .map_err(From::from)
    }
}

#[async_trait]
impl RequestSubmitter for IndyVdrSubmitter {
    async fn submit(&self, request: PreparedRequest) -> VcxCoreResult<String> {
        // indyvdr send_request is Async via a callback.
        // Use oneshot channel to send result from callback, converting the fn to future.
        type VdrSendRequestResult = Result<
            (
                RequestResult<String>,
                Option<HashMap<String, f32, RandomState>>,
            ),
            VdrError,
        >;
        let (sender, recv) = oneshot::channel::<VdrSendRequestResult>();
        self.send_request(
            request,
            Box::new(move |result| {
                // unable to handle a failure from `send` here
                sender.send(result).ok();
            }),
        )?;

        let send_req_result: VdrSendRequestResult = recv
            .await
            .map_err(|e| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidState, e))?;
        let (result, _) = send_req_result?;

        let reply = match result {
            RequestResult::Reply(reply) => Ok(reply),
            RequestResult::Failed(failed) => Err(failed),
        };

        Ok(reply?)
    }
}
