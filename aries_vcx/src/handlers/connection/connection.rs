use std::clone::Clone;

use messages::a2a::A2AMessage;
use serde::{Deserialize, Serialize};
use vdrtools_sys::WalletHandle;

use crate::error::prelude::*;
use crate::protocols::{SendClosure, SendClosureConnection};
use crate::protocols::connection::invitee::state_machine::{InviteeFullState, InviteeState, SmConnectionInvitee};
use crate::protocols::connection::inviter::state_machine::{InviterFullState, InviterState, SmConnectionInviter};
use crate::protocols::connection::pairwise_info::PairwiseInfo;
use crate::utils::send_message;
use messages::connection::invite::Invitation;
use messages::connection::request::Request;
use messages::did_doc::DidDoc;

#[derive(Clone, PartialEq)]
pub struct Connection {
    connection_sm: SmConnection,
}

#[derive(Clone, PartialEq)]
pub enum SmConnection {
    Inviter(SmConnectionInviter),
    Invitee(SmConnectionInvitee),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmConnectionState {
    Inviter(InviterFullState),
    Invitee(InviteeFullState),
}

#[derive(Debug, PartialEq)]
pub enum ConnectionState {
    Inviter(InviterState),
    Invitee(InviteeState),
}

impl Connection {
    // ----------------------------- CONSTRUCTORS ------------------------------------
    pub async fn create_inviter(wallet_handle: WalletHandle) -> VcxResult<Self> {
        trace!("Connection::create >>>");
        let pairwise_info = PairwiseInfo::create(wallet_handle).await?;
        Ok(Self {
            connection_sm: SmConnection::Inviter(SmConnectionInviter::new("", pairwise_info)),
        })
    }

    pub async fn create_invitee(wallet_handle: WalletHandle, did_doc: DidDoc) -> VcxResult<Self> {
        trace!("Connection::create_with_invite >>>");
        Ok(Self {
            connection_sm: SmConnection::Invitee(SmConnectionInvitee::new(
                "",
                PairwiseInfo::create(wallet_handle).await?,
                did_doc,
            )),
        })
    }

    // ----------------------------- GETTERS ------------------------------------
    // TODO: Do clones ALWAYS make sense?
    pub fn get_thread_id(&self) -> String {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.get_thread_id(),
            SmConnection::Invitee(sm_invitee) => sm_invitee.get_thread_id(),
        }
    }

    pub fn get_state(&self) -> ConnectionState {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => ConnectionState::Inviter(sm_inviter.get_state()),
            SmConnection::Invitee(sm_invitee) => ConnectionState::Invitee(sm_invitee.get_state()),
        }
    }

    pub fn pairwise_info(&self) -> &PairwiseInfo {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.pairwise_info(),
            SmConnection::Invitee(sm_invitee) => sm_invitee.pairwise_info(),
        }
    }

    pub async fn remote_did(&self) -> VcxResult<String> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.remote_did(),
            SmConnection::Invitee(sm_invitee) => sm_invitee.remote_did().await,
        }
    }

    pub async fn remote_vk(&self) -> VcxResult<String> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.remote_vk(),
            SmConnection::Invitee(sm_invitee) => sm_invitee.remote_vk().await,
        }
    }

    pub fn state_object(&self) -> SmConnectionState {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => SmConnectionState::Inviter(sm_inviter.state_object().clone()),
            SmConnection::Invitee(sm_invitee) => SmConnectionState::Invitee(sm_invitee.state_object().clone()),
        }
    }

    pub async fn their_did_doc(&self) -> Option<DidDoc> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.their_did_doc(),
            SmConnection::Invitee(sm_invitee) => sm_invitee.their_did_doc().await,
        }
    }

    pub fn get_invite_details(&self) -> Option<&Invitation> {
        trace!("Connection::get_invite_details >>>");
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.get_invitation(),
            SmConnection::Invitee(_sm_invitee) => None,
        }
    }

    // ----------------------------- MSG PROCESSING ------------------------------------
    pub fn process_invite(&mut self, invitation: Invitation) -> VcxResult<()> {
        trace!("Connection::process_invite >>> invitation: {:?}", invitation);
        self.connection_sm = match &self.connection_sm {
            SmConnection::Inviter(_sm_inviter) => {
                return Err(VcxError::from_msg(VcxErrorKind::NotReady, "Invalid action"));
            }
            SmConnection::Invitee(sm_invitee) => {
                SmConnection::Invitee(sm_invitee.clone().handle_invitation(invitation)?)
            }
        };
        Ok(())
    }

    pub async fn process_request(
        &mut self,
        wallet_handle: WalletHandle,
        request: Request,
        routing_keys: Vec<String>,
        service_endpoint: String,
    ) -> VcxResult<()> {
        trace!("Connection::process_request >>> request: {:?}", request);
        let connection_sm = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                let send_message = self.send_message_closure_connection(wallet_handle);
                let new_pairwise_info = PairwiseInfo::create(wallet_handle).await?;
                let send_message = self.send_message_closure(wallet_handle).await?;
                SmConnection::Inviter(
                    sm_inviter
                        .clone()
                        .handle_connection_request(
                            wallet_handle,
                            request,
                            &new_pairwise_info,
                            routing_keys,
                            service_endpoint,
                            send_message,
                        )
                        .await?,
                )
            }
            SmConnection::Invitee(_) => {
                return Err(VcxError::from_msg(VcxErrorKind::NotReady, "Invalid action"));
            }
        };
        self.connection_sm = connection_sm;
        Ok(())
    }

    // TODO: Does Query, Disclose, Ping, OOB processing REALLY belong here?

    // ----------------------------- MSG SENDING ------------------------------------
    pub async fn send_response(self, wallet_handle: WalletHandle) -> VcxResult<Self> {
        trace!("Connection::send_response >>>");
        let connection_sm = match self.connection_sm.clone() {
            SmConnection::Inviter(sm_inviter) => {
                if let InviterFullState::Requested(_) = sm_inviter.state_object() {
                    let send_message = self.send_message_closure_connection(wallet_handle);
                    SmConnection::Inviter(sm_inviter.handle_send_response(send_message).await?)
                } else {
                    return Err(VcxError::from_msg(VcxErrorKind::NotReady, "Invalid action"));
                }
            }
            SmConnection::Invitee(_) => {
                return Err(VcxError::from_msg(VcxErrorKind::NotReady, "Invalid action"));
            }
        };
        Ok(Self {
            connection_sm,
            ..self
        })
    }

    pub async fn send_request(
        self,
        wallet_handle: WalletHandle,
        service_endpoint: String,
        routing_keys: Vec<String>,
    ) -> VcxResult<Self> {
        trace!("Connection::send_request");
        let connection_sm = match &self.connection_sm {
            SmConnection::Inviter(_) => {
                return Err(VcxError::from_msg(VcxErrorKind::NotReady, "Inviter cannot send connection request"));
            }
            SmConnection::Invitee(sm_invitee) => {
                SmConnection::Invitee(
                    sm_invitee
                        .clone()
                        .send_connection_request(
                            routing_keys,
                            service_endpoint,
                            self.send_message_closure_connection(wallet_handle)
                        )
                        .await?
                )
            }
        };
        Ok(Self {
            connection_sm,
            ..self
        })
    }

    pub async fn create_invite(
        self,
        service_endpoint: String,
        routing_keys: Vec<String>,
    ) -> VcxResult<Self> {
        trace!("Connection::create_invite >>>");
        let connection_sm = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                SmConnection::Inviter(sm_inviter.clone().create_invitation(routing_keys, service_endpoint)?)
            }
            SmConnection::Invitee(_) => {
                return Err(VcxError::from_msg(VcxErrorKind::NotReady, "Invitee cannot create invite"));
            }
        };
        Ok(Self {
            connection_sm,
            ..self
        })
    }

    pub async fn send_message_closure(&self, wallet_handle: WalletHandle) -> VcxResult<SendClosure> {
        trace!("send_message_closure >>>");
        let did_doc = self.their_did_doc().await.ok_or(VcxError::from_msg(
            VcxErrorKind::NotReady,
            "Cannot send message: Remote Connection information is not set",
        ))?;
        let sender_vk = self.pairwise_info().pw_vk.clone();
        Ok(Box::new(move |message: A2AMessage| {
            Box::pin(send_message(wallet_handle, sender_vk.clone(), did_doc.clone(), message))
        }))
    }

    fn send_message_closure_connection(&self, wallet_handle: WalletHandle) -> SendClosureConnection {
        trace!("send_message_closure_connection >>>");
        Box::new(move |message: A2AMessage, sender_vk: String, did_doc: DidDoc| {
            Box::pin(send_message(wallet_handle, sender_vk, did_doc, message))
        })
    }
}
