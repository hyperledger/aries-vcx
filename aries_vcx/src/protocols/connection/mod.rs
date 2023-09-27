mod generic;
pub mod initiation_type;
pub mod invitee;
pub mod inviter;
pub mod pairwise_info;
mod serializable;
mod trait_bounds;

use std::error::Error;

use aries_vcx_core::wallet::base_wallet::BaseWallet;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use messages::{
    msg_fields::protocols::discover_features::{
        disclose::Disclose, query::QueryContent, ProtocolDescriptor,
    },
    AriesMessage,
};

pub use self::generic::{GenericConnection, State, ThinState};
use self::{
    generic::GenericState,
    pairwise_info::PairwiseInfo,
    trait_bounds::{CompletedState, TheirDidDoc, ThreadId},
};
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
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
    pub fn from_parts(
        source_id: String,
        pairwise_info: PairwiseInfo,
        initiation_type: I,
        state: S,
    ) -> Self {
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
        let query = QueryContent::builder().query("*".to_owned()).build();
        query.lookup()
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

    pub async fn encrypt_message(
        &self,
        wallet: &Arc<dyn BaseWallet>,
        message: &AriesMessage,
    ) -> VcxResult<EncryptionEnvelope> {
        let sender_verkey = &self.pairwise_info().pw_vk;
        EncryptionEnvelope::create(wallet, message, Some(sender_verkey), self.their_did_doc()).await
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
        wallet: &impl BaseWallet,
        message: &AriesMessage,
        transport: &T,
    ) -> VcxResult<()>
    where
        T: Transport,
    {
        let msg = self.encrypt_message(wallet, message).await?.0;
        let service_endpoint = self.their_did_doc().get_endpoint().ok_or_else(|| {
            AriesVcxError::from_msg(AriesVcxErrorKind::InvalidUrl, "No URL in DID Doc")
        })?;
        transport.send_message(msg, service_endpoint).await
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
