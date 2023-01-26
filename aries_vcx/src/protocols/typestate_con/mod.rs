mod common;
mod initiation_type;
mod invitee;
mod inviter;
mod pairwise_info;
pub mod serde;
mod trait_bounds;

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
    serde::{SerdeCon, SerdeState},
    trait_bounds::{TheirDidDoc, Transport},
};

// The serialization will first convert into the serializable connection type
// but will do that by cloning, which is fairly unfortunate and unnecessary.
//
// We can theoretically serialize this type as well directly, through a reference,
// but we must align the deserialization to be to the serializable connection
// and ensure the format matches.
//
// Can definitely be done, but just requires a bit of work put into it.
#[derive(Clone, Serialize)]
#[serde(into = "SerdeCon")]
#[serde(bound(serialize = "SerdeState: From<(I, S)>, I: Clone, S: Clone"))]
pub struct Connection<I, S> {
    source_id: String,
    pairwise_info: PairwiseInfo,
    initiation_type: I,
    state: S,
}

impl<I, S> Connection<I, S> {
    pub fn pairwise_info(&self) -> &PairwiseInfo {
        &self.pairwise_info
    }

    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    pub fn protocols(&self) -> Vec<ProtocolDescriptor> {
        ProtocolRegistry::init().protocols()
    }

    pub async fn send_message<T>(
        wallet: &Arc<dyn BaseWallet>,
        did_doc: &AriesDidDoc,
        message: &A2AMessage,
        sender_verkey: &str,
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
}

impl<I> Connection<I, CompleteState> {
    pub fn remote_protocols(&self) -> Option<&[ProtocolDescriptor]> {
        self.state.remote_protocols()
    }

    pub fn handle_disclose(&mut self, disclose: Disclose) {
        self.state.handle_disclose(disclose)
    }
}
