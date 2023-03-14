mod generic;
pub mod initiation_type;
pub mod invitee;
pub mod inviter;
pub mod pairwise_info;
mod serializable;
mod trait_bounds;

use std::{error::Error, sync::Arc};

use messages::{
    a2a::{protocol_registry::ProtocolRegistry, A2AMessage},
    diddoc::aries::diddoc::AriesDidDoc,
    protocols::{
        connection::problem_report::{ProblemCode, ProblemReport},
        discovery::disclose::{Disclose, ProtocolDescriptor},
    },
};

pub use self::generic::{GenericConnection, State, ThinState};
use self::{
    generic::GenericState,
    pairwise_info::PairwiseInfo,
    trait_bounds::{CompletedState, HandleProblem, TheirDidDoc, ThreadId},
};
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    plugins::wallet::base_wallet::BaseWallet,
    transport::Transport,
    utils::encryption_envelope::EncryptionEnvelope,
};

/// A state machine for progressing through the [connection protocol](https://github.com/hyperledger/aries-rfcs/blob/main/features/0160-connection-protocol/README.md).
#[derive(Clone, Deserialize)]
#[serde(try_from = "GenericConnection")]
#[serde(bound = "(I, S): TryFrom<GenericState, Error = AriesVcxError>")]
pub struct Connection<I, S> {
    source_id: String,
    pairwise_info: PairwiseInfo,
    initiation_type: I,
    state: S,
}

impl<I, S> Connection<I, S> {
    pub fn from_parts(source_id: String, pairwise_info: PairwiseInfo, initiation_type: I, state: S) -> Self {
        Self {
            source_id,
            pairwise_info,
            initiation_type,
            state,
        }
    }

    pub fn into_parts(self) -> (String, PairwiseInfo, I, S) {
        let Self {
            source_id,
            pairwise_info,
            initiation_type,
            state,
        } = self;
        (source_id, pairwise_info, initiation_type, state)
    }

    pub fn pairwise_info(&self) -> &PairwiseInfo {
        &self.pairwise_info
    }

    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    pub fn protocols(&self) -> Vec<ProtocolDescriptor> {
        ProtocolRegistry::init().protocols()
    }
}

impl<I, S> Connection<I, S>
where
    S: ThreadId,
{
    pub fn thread_id(&self) -> &str {
        self.state.thread_id()
    }
}

impl<I, S> Connection<I, S>
where
    S: TheirDidDoc,
{
    pub fn their_did_doc(&self) -> &AriesDidDoc {
        self.state.their_did_doc()
    }

    pub fn remote_did(&self) -> &str {
        &self.their_did_doc().id
    }

    pub fn remote_vk(&self) -> VcxResult<String> {
        self.their_did_doc()
            .recipient_keys()?
            .first()
            .map(ToOwned::to_owned)
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Can't resolve recipient key from the counterparty diddoc.",
            ))
    }

    pub async fn send_message<T>(
        &self,
        wallet: &Arc<dyn BaseWallet>,
        message: &A2AMessage,
        transport: &T,
    ) -> VcxResult<()>
    where
        T: Transport,
    {
        let sender_verkey = &self.pairwise_info().pw_vk;
        let did_doc = self.their_did_doc();
        wrap_and_send_msg(wallet, message, sender_verkey, did_doc, transport).await
    }
}

impl<I, S> Connection<I, S>
where
    S: HandleProblem,
{
    fn create_problem_report<E>(&self, err: &E, thread_id: &str) -> ProblemReport
    where
        E: Error,
    {
        ProblemReport::create()
            .set_problem_code(ProblemCode::RequestProcessingError)
            .set_explain(err.to_string())
            .set_thread_id(thread_id)
            .set_out_time()
    }

    async fn send_problem_report<E, T>(
        &self,
        wallet: &Arc<dyn BaseWallet>,
        err: &E,
        thread_id: &str,
        did_doc: &AriesDidDoc,
        transport: &T,
    ) where
        E: Error,
        T: Transport,
    {
        let sender_verkey = &self.pairwise_info().pw_vk;
        let problem_report = self.create_problem_report(err, thread_id);
        let res = wrap_and_send_msg(
            wallet,
            &problem_report.to_a2a_message(),
            sender_verkey,
            did_doc,
            transport,
        )
        .await;

        if let Err(e) = res {
            trace!("Error encountered when sending ProblemReport: {}", e);
        } else {
            info!("Error report sent!");
        }
    }
}

impl<I, S> Connection<I, S>
where
    S: CompletedState,
{
    pub fn remote_protocols(&self) -> Option<&[ProtocolDescriptor]> {
        self.state.remote_protocols()
    }

    pub fn handle_disclose(&mut self, disclose: Disclose) {
        self.state.handle_disclose(disclose)
    }
}

pub(crate) async fn wrap_and_send_msg<T>(
    wallet: &Arc<dyn BaseWallet>,
    message: &A2AMessage,
    sender_verkey: &str,
    did_doc: &AriesDidDoc,
    transport: &T,
) -> VcxResult<()>
where
    T: Transport,
{
    let env = EncryptionEnvelope::create(wallet, message, Some(sender_verkey), did_doc).await?;
    let msg = env.0;
    let service_endpoint = did_doc.get_endpoint(); // This, like many other things, shouldn't clone...

    transport.send_message(msg, &service_endpoint).await
}
