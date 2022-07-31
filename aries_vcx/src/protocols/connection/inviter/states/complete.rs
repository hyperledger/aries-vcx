use std::clone::Clone;
use std::future::Future;

use indy_sys::WalletHandle;

use crate::error::prelude::*;
use crate::messages::a2a::A2AMessage;
use crate::messages::a2a::protocol_registry::ProtocolRegistry;
use crate::did_doc::DidDoc;
use crate::messages::discovery::disclose::{Disclose, ProtocolDescriptor};
use crate::messages::discovery::query::Query;
use crate::messages::out_of_band::handshake_reuse::OutOfBandHandshakeReuse;
use crate::messages::out_of_band::handshake_reuse_accepted::OutOfBandHandshakeReuseAccepted;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompleteState {
    pub did_doc: DidDoc,
    pub protocols: Option<Vec<ProtocolDescriptor>>,
    pub thread_id: Option<String>,
}

impl From<(CompleteState, Vec<ProtocolDescriptor>)> for CompleteState {
    fn from((state, protocols): (CompleteState, Vec<ProtocolDescriptor>)) -> CompleteState {
        trace!("ConnectionInviter: transit state from CompleteState to CompleteState");
        CompleteState { did_doc: state.did_doc, thread_id: state.thread_id, protocols: Some(protocols) }
    }
}

impl CompleteState {
    pub async fn handle_send_handshake_reuse<F, T>(&self,
                                                   wallet_handle: WalletHandle,
                                                   oob_id: &str,
                                                   pw_vk: &str,
                                                   send_message: F,
    ) -> VcxResult<()>
        where
            F: Fn(WalletHandle, String, DidDoc, A2AMessage) -> T,
            T: Future<Output=VcxResult<()>>
    {
        let reuse_msg = OutOfBandHandshakeReuse::default()
            .set_thread_id_matching_id()
            .set_parent_thread_id(oob_id);
        send_message(wallet_handle, pw_vk.to_string(), self.did_doc.clone(), reuse_msg.to_a2a_message()).await.ok();
        Ok(())
    }

    pub async fn handle_send_handshake_reuse_accepted<F, T>(&self,
                                                            wallet_handle: WalletHandle,
                                                            reuse_msg: OutOfBandHandshakeReuse,
                                                            pw_vk: &str,
                                                            send_message: F,
    ) -> VcxResult<()>
        where
            F: Fn(WalletHandle, String, DidDoc, A2AMessage) -> T,
            T: Future<Output=VcxResult<()>>
    {
        let ack_msg = OutOfBandHandshakeReuseAccepted::default()
            .set_thread_id(&reuse_msg.get_thread_id())
            .set_parent_thread_id(&reuse_msg.thread.pthid.ok_or(VcxError::from_msg(VcxErrorKind::InvalidOption, "Parent thread missing"))?);
        send_message(wallet_handle, pw_vk.to_string(), self.did_doc.clone(), ack_msg.to_a2a_message()).await.ok();
        Ok(())
    }


    pub async fn handle_discover_features<F, T>(&self,
                                                wallet_handle: WalletHandle,
                                                query: Option<String>,
                                                comment: Option<String>,
                                                pw_vk: &str,
                                                send_message: F,
    ) -> VcxResult<()>
        where
            F: Fn(WalletHandle, String, DidDoc, A2AMessage) -> T,
            T: Future<Output=VcxResult<()>>
    {
        let query_ =
            Query::create()
                .set_query(query)
                .set_comment(comment);

        send_message(wallet_handle, pw_vk.to_string(), self.did_doc.clone(), query_.to_a2a_message()).await
    }

    pub async fn handle_discovery_query<F, T>(&self,
                                              wallet_handle: WalletHandle,
                                              query: Query,
                                              pw_vk: &str,
                                              send_message: F,
    ) -> VcxResult<()>
        where
            F: Fn(WalletHandle, String, DidDoc, A2AMessage) -> T,
            T: Future<Output=VcxResult<()>>
    {
        let protocols = ProtocolRegistry::init().get_protocols_for_query(query.query.as_deref());

        let disclose = Disclose::create()
            .set_protocols(protocols)
            .set_thread_id(&query.id.0.clone());

        send_message(wallet_handle, pw_vk.to_string(), self.did_doc.clone(), disclose.to_a2a_message()).await
    }
}
