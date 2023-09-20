use std::{
    collections::{hash_map::RandomState, HashMap},
    fmt::{Debug, Formatter},
    sync::Arc,
};

use async_trait::async_trait;
use indy_vdr::{
    common::error::VdrError,
    config::PoolConfig,
    pool::{PoolBuilder, PoolRunner, PoolTransactions, PreparedRequest, RequestResult},
};
use tokio::sync::oneshot;

use super::RequestSubmitter;
use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};

pub struct IndyVdrLedgerPool {
    pub(self) runner: Option<PoolRunner>,
}

impl IndyVdrLedgerPool {
    pub fn new_from_runner(runner: PoolRunner) -> Self {
        IndyVdrLedgerPool {
            runner: Some(runner),
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
            runner: Some(runner),
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

pub struct IndyVdrSubmitter {
    pool: Arc<IndyVdrLedgerPool>,
}

impl IndyVdrSubmitter {
    pub fn new(pool: Arc<IndyVdrLedgerPool>) -> Self {
        Self { pool }
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
        self.pool
            .runner
            .as_ref()
            .ok_or(
                // should not happen - strictly for unit testing
                AriesVcxCoreError::from_msg(
                    AriesVcxCoreErrorKind::NoPoolOpen,
                    "IndyVdrLedgerPool runner was not provided",
                ),
            )?
            .send_request(
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
