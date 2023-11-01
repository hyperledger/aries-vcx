use std::collections::HashMap;

use async_trait::async_trait;
use futures::channel::oneshot;
use indy_vdr::{
    common::error::VdrError,
    pool::{PreparedRequest, RequestResult},
};

use super::{Ledger, LedgerRequest};
use crate::{
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult},
    ledger::request_submitter::vdr_ledger::IndyVdrSubmitter,
};

#[async_trait]
impl Ledger for IndyVdrSubmitter {
    type Request = PreparedRequest;

    type Response = RequestResult<String>;

    async fn submit<R>(&self, request: Self::Request) -> VcxCoreResult<R>
    where
        R: LedgerRequest<Self>,
    {
        // indyvdr send_request is Async via a callback.
        // Use oneshot channel to send result from callback, converting the fn to future.
        type VdrSendRequestResult =
            Result<(RequestResult<String>, Option<HashMap<String, f32>>), VdrError>;
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

        R::from_ledger_response(result)
    }
}
