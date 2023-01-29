mod common;
mod initiation_type;
pub mod invitee;
pub mod inviter;
pub mod pairwise_info;
mod serde;
mod traits;

use messages::{
    a2a::{protocol_registry::ProtocolRegistry, A2AMessage},
    diddoc::aries::diddoc::AriesDidDoc,
    protocols::discovery::disclose::{Disclose, ProtocolDescriptor},
};
use std::sync::Arc;

use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    plugins::wallet::base_wallet::BaseWallet,
    utils::encryption_envelope::EncryptionEnvelope,
};

use self::{
    common::states::complete::CompleteState,
    pairwise_info::PairwiseInfo,
    serde::de::VagueState,
    traits::{TheirDidDoc, ThreadId},
};

pub use self::serde::de::VagueConnection;
pub use self::traits::Transport;

#[derive(Clone, Deserialize)]
#[serde(try_from = "VagueConnection")]
#[serde(bound = "(I, S): TryFrom<VagueState, Error = AriesVcxError>")]
pub struct Connection<I, S> {
    source_id: String,
    pairwise_info: PairwiseInfo,
    initiation_type: I,
    state: S,
}

impl<I, S> Connection<I, S> {
    pub(crate) fn from_parts(source_id: String, pairwise_info: PairwiseInfo, initiation_type: I, state: S) -> Self {
        Self {
            source_id,
            pairwise_info,
            initiation_type,
            state,
        }
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

    pub(crate) async fn basic_send_message<T>(
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
        Self::basic_send_message(wallet, message, sender_verkey, did_doc, transport).await
    }
}

impl<I> Connection<I, CompleteState> {
    pub fn remote_protocols(&self) -> Option<&[ProtocolDescriptor]> {
        self.state.remote_protocols()
    }

    pub fn handle_disclose(&mut self, disclose: Disclose) {
        self.state.handle_disclose(disclose)
    }
}
