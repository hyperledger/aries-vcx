use core::fmt;
use std::clone::Clone;
use std::collections::HashMap;

use futures::future::BoxFuture;
use serde::de::{Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use vdrtools_sys::WalletHandle;

use crate::error::prelude::*;
use crate::handlers::connection::legacy_agent_info::LegacyAgentInfo;
use crate::handlers::discovery::{respond_discovery_query, send_discovery_query};
use crate::handlers::trust_ping::TrustPingSender;
use crate::protocols::connection::invitee::state_machine::{InviteeFullState, InviteeState, SmConnectionInvitee};
use crate::protocols::connection::inviter::state_machine::{InviterFullState, InviterState, SmConnectionInviter};
use crate::protocols::connection::pairwise_info::PairwiseInfo;
use crate::protocols::oob::{build_handshake_reuse_accepted_msg, build_handshake_reuse_msg};
use crate::protocols::trustping::build_ping_response;
use crate::protocols::{SendClosure, SendClosureConnection};
use crate::utils::send_message;
use crate::utils::serialization::SerializableObjectWithState;
use messages::a2a::protocol_registry::ProtocolRegistry;
use messages::a2a::A2AMessage;
use messages::basic_message::message::BasicMessage;
use messages::connection::invite::Invitation;
use messages::connection::request::Request;
use messages::did_doc::DidDoc;
use messages::discovery::disclose::{Disclose, ProtocolDescriptor};

#[derive(Clone, PartialEq)]
pub struct Connection {
    connection_sm: SmConnection,
    autohop_enabled: bool,
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

#[derive(Debug, Serialize)]
struct ConnectionInfo {
    my: SideConnectionInfo,
    their: Option<SideConnectionInfo>,
}

#[derive(Debug, PartialEq)]
pub enum ConnectionState {
    Inviter(InviterState),
    Invitee(InviteeState),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SideConnectionInfo {
    did: String,
    recipient_keys: Vec<String>,
    routing_keys: Vec<String>,
    service_endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    protocols: Option<Vec<ProtocolDescriptor>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Actor {
    Inviter,
    Invitee,
}

impl Connection {
    // ----------------------------- CONSTRUCTORS ------------------------------------
    pub async fn create(
        source_id: &str,
        wallet_handle: WalletHandle,
        autohop_enabled: bool,
    ) -> VcxResult<Self> {
        trace!("Connection::create >>> source_id: {}", source_id);
        let pairwise_info = PairwiseInfo::create(wallet_handle).await?;
        Ok(Self {
            connection_sm: SmConnection::Inviter(SmConnectionInviter::new(source_id, pairwise_info)),
            autohop_enabled,
        })
    }

    pub async fn create_with_invite(
        source_id: &str,
        wallet_handle: WalletHandle,
        invitation: Invitation,
        did_doc: DidDoc,
        autohop_enabled: bool,
    ) -> VcxResult<Self> {
        trace!(
            "Connection::create_with_invite >>> source_id: {}, invitation: {:?}",
            source_id,
            invitation
        );
        let pairwise_info = PairwiseInfo::create(wallet_handle).await?;
        let mut connection = Self {
            connection_sm: SmConnection::Invitee(SmConnectionInvitee::new(source_id, pairwise_info, did_doc)),
            autohop_enabled,
        };
        connection.process_invite(invitation)?;
        Ok(connection)
    }

    pub async fn create_with_request(
        wallet_handle: WalletHandle,
        request: Request,
        pairwise_info: PairwiseInfo,
    ) -> VcxResult<Self> {
        trace!(
            "Connection::create_with_request >>> request: {:?}, pairwise_info: {:?}",
            request,
            pairwise_info
        );
        let mut connection = Self {
            connection_sm: SmConnection::Inviter(SmConnectionInviter::new(&request.id.0, pairwise_info)),
            autohop_enabled: true,
        };
        connection
            .process_request(wallet_handle, request, vec![], )
            .await?;
        Ok(connection)
    }

    pub fn from_parts(
        source_id: String,
        thread_id: String,
        pairwise_info: PairwiseInfo,
        state: SmConnectionState,
        autohop_enabled: bool,
    ) -> Self {
        match state {
            SmConnectionState::Inviter(state) => Self {
                connection_sm: SmConnection::Inviter(SmConnectionInviter::from(
                    source_id,
                    thread_id,
                    pairwise_info,
                    state,
                )),
                autohop_enabled,
            },
            SmConnectionState::Invitee(state) => Self {
                connection_sm: SmConnection::Invitee(SmConnectionInvitee::from(
                    source_id,
                    thread_id,
                    pairwise_info,
                    state,
                )),
                autohop_enabled,
            },
        }
    }

    // ----------------------------- GETTERS ------------------------------------
    pub fn source_id(&self) -> String {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.source_id(),
            SmConnection::Invitee(sm_invitee) => sm_invitee.source_id(),
        }
        .into()
    }

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

    pub fn get_source_id(&self) -> String {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.source_id(),
            SmConnection::Invitee(sm_invitee) => sm_invitee.source_id(),
        }
        .to_string()
    }

    pub fn get_protocols(&self) -> Vec<ProtocolDescriptor> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.get_protocols(),
            SmConnection::Invitee(sm_invitee) => sm_invitee.get_protocols(),
        }
    }

    pub fn get_remote_protocols(&self) -> Option<Vec<ProtocolDescriptor>> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.get_remote_protocols(),
            SmConnection::Invitee(sm_invitee) => sm_invitee.get_remote_protocols(),
        }
    }

    pub async fn their_did_doc(&self) -> Option<DidDoc> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.their_did_doc(),
            SmConnection::Invitee(sm_invitee) => sm_invitee.their_did_doc().await,
        }
    }

    pub async fn bootstrap_did_doc(&self) -> Option<DidDoc> {
        match &self.connection_sm {
            SmConnection::Inviter(_sm_inviter) => None, // TODO: Inviter can remember bootstrap agent too, but we don't need it
            SmConnection::Invitee(sm_invitee) => sm_invitee.bootstrap_did_doc().await,
        }
    }

    pub fn is_in_null_state(&self) -> bool {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.is_in_null_state(),
            SmConnection::Invitee(sm_invitee) => sm_invitee.is_in_null_state(),
        }
    }

    pub fn is_in_final_state(&self) -> bool {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.is_in_final_state(),
            SmConnection::Invitee(sm_invitee) => sm_invitee.is_in_final_state(),
        }
    }

    pub fn get_invite_details(&self) -> Option<&Invitation> {
        trace!("Connection::get_invite_details >>>");
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.get_invitation(),
            SmConnection::Invitee(_sm_invitee) => None,
        }
    }

    async fn get_expected_sender_vk(&self) -> VcxResult<String> {
        self.remote_vk().await.map_err(|_err| {
            VcxError::from_msg(
                VcxErrorKind::NotReady,
                "Verkey of connection counterparty \
                is not known, hence it would be impossible to authenticate message downloaded by id.",
            )
        })
    }

    pub async fn get_connection_info(&self) -> VcxResult<String> {
        trace!("Connection::get_connection_info >>>");
        todo!()
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
        service_endpoint: String
    ) -> VcxResult<()> {
        trace!("Connection::process_request >>> request: {:?}", request);
        let connection_sm = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                let send_message = self.send_message_closure_connection(wallet_handle);
                let new_pairwise_info = PairwiseInfo::create(wallet_handle).await?;
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

    pub async fn send_response(&mut self, wallet_handle: WalletHandle) -> VcxResult<()> {
        trace!("Connection::send_response >>>");
        let connection_sm = match self.connection_sm.clone() {
            SmConnection::Inviter(sm_inviter) => {
                if let InviterFullState::Requested(_) = sm_inviter.state_object() {
                    let send_message = self.send_message_closure_connection(wallet_handle);
                    sm_inviter.handle_send_response(send_message).await?
                } else {
                    return Err(VcxError::from_msg(VcxErrorKind::NotReady, "Invalid action"));
                }
            }
            SmConnection::Invitee(_) => {
                return Err(VcxError::from_msg(VcxErrorKind::NotReady, "Invalid action"));
            }
        };
        self.connection_sm = SmConnection::Inviter(connection_sm);
        Ok(())
    }

    pub fn get_invite_details(&self) -> Option<&Invitation> {
        trace!("Connection::get_invite_details >>>");
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.get_invitation(),
            SmConnection::Invitee(_sm_invitee) => None,
        }
    }

    fn find_message_to_update_state(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.find_message_to_update_state(messages),
            SmConnection::Invitee(sm_invitee) => sm_invitee.find_message_to_update_state(messages),
        }
    }

    pub fn update_state_with_message(
        &mut self,
        wallet_handle: WalletHandle,
        message: Option<A2AMessage>,
    ) -> BoxFuture<'_, VcxResult<()>> {
        Box::pin(async move {
            let (new_connection_sm, can_autohop) = match &self.connection_sm {
                SmConnection::Inviter(_) => self.step_inviter(wallet_handle, message).await?,
                SmConnection::Invitee(_) => self.step_invitee(wallet_handle, message).await?,
            };
            *self = new_connection_sm;
            if can_autohop && self.autohop_enabled {
                let res = self.update_state_with_message(wallet_handle, None).await;
                res
            } else {
                Ok(())
            }
        })
    }

    fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        for (uid, message) in messages {
            match message {
                A2AMessage::Ping(_)
                | A2AMessage::PingResponse(_)
                | A2AMessage::OutOfBandHandshakeReuse(_)
                | A2AMessage::OutOfBandHandshakeReuseAccepted(_)
                | A2AMessage::Query(_)
                | A2AMessage::Disclose(_) => return Some((uid, message)),
                _ => {}
            }
        }
        None
    }

    pub async fn handle_message(&mut self, message: A2AMessage, wallet_handle: WalletHandle) -> VcxResult<()> {
        let did_doc = self.their_did_doc().await.ok_or(VcxError::from_msg(
            VcxErrorKind::NotReady,
            format!(
                "Can't answer message {:?} because counterparty did doc is not available",
                message
            ),
        ))?;
        let pw_vk = &self.pairwise_info().pw_vk;
        match message {
            A2AMessage::Ping(ping) => {
                info!("Answering ping, thread: {}", ping.get_thread_id());
                if ping.response_requested {
                    send_message(
                        wallet_handle,
                        pw_vk.to_string(),
                        did_doc.clone(),
                        build_ping_response(&ping).to_a2a_message(),
                    )
                    .await?;
                }
            }
            A2AMessage::OutOfBandHandshakeReuse(handshake_reuse) => {
                info!(
                    "Answering OutOfBandHandshakeReuse message, thread: {}",
                    handshake_reuse.get_thread_id()
                );
                let msg = build_handshake_reuse_accepted_msg(&handshake_reuse)?;
                send_message(wallet_handle, pw_vk.to_string(), did_doc.clone(), msg.to_a2a_message()).await?;
            }
            A2AMessage::Query(query) => {
                let supported_protocols = ProtocolRegistry::init().get_protocols_for_query(query.query.as_deref());
                info!(
                    "Answering discovery protocol query, @id: {}, with supported protocols: {:?}",
                    query.id.0, &supported_protocols
                );
                respond_discovery_query(wallet_handle, query, &did_doc, pw_vk, supported_protocols).await?;
            }
            A2AMessage::Disclose(disclose) => {
                info!("Handling disclose message, thread: {}", disclose.get_thread_id());
                self.connection_sm = self.handle_disclose(disclose).await?;
            }
            _ => {
                // todo: implement to_string for A2AMessage, printing only type of the message, not entire payload
                // todo: attempt to print @id / thread_id of the message
                info!("Message of type {:?} will not be answered", message);
            }
        }
        Ok(())
    }

    async fn step_inviter(
        &self,
        wallet_handle: WalletHandle,
        message: Option<A2AMessage>,
    ) -> VcxResult<(Self, bool)> {
        match self.connection_sm.clone() {
            SmConnection::Inviter(sm_inviter) => {
                let (sm_inviter, can_autohop) = match message {
                    Some(message) => match message {
                        A2AMessage::ConnectionRequest(request) => {
                            let send_message = self.send_message_closure_connection(wallet_handle);
                            let new_pairwise_info = PairwiseInfo::create(wallet_handle).await?;
                            let sm_connection = sm_inviter
                                .handle_connection_request(
                                    wallet_handle,
                                    request,
                                    &new_pairwise_info,
                                    vec![],
                                    "service_endpoint".to_string(),
                                    send_message,
                                )
                                .await?;
                            (sm_connection, true)
                        }
                        msg @ A2AMessage::Ack(_) | msg @ A2AMessage::Ping(_) => {
                            (sm_inviter.handle_confirmation_message(&msg).await?, false)
                        }
                        A2AMessage::ConnectionProblemReport(problem_report) => {
                            (sm_inviter.handle_problem_report(problem_report)?, false)
                        }
                        _ => (sm_inviter.clone(), false),
                    },
                    None => {
                        if let InviterFullState::Requested(_) = sm_inviter.state_object() {
                            let send_message = self.send_message_closure_connection(wallet_handle);
                            (
                                sm_inviter.handle_send_response(send_message).await?,
                                None,
                                false,
                            )
                        } else {
                            (sm_inviter.clone(), false)
                        }
                    }
                };

                let connection = Self {
                    connection_sm: SmConnection::Inviter(sm_inviter),
                    autohop_enabled: self.autohop_enabled,
                };

                Ok((connection, can_autohop))
            }
            SmConnection::Invitee(_) => Err(VcxError::from_msg(
                VcxErrorKind::NotReady,
                "Invalid operation, called \
                _step_inviter on Invitee connection.",
            )),
        }
    }

    async fn step_invitee(&self, wallet_handle: WalletHandle, message: Option<A2AMessage>) -> VcxResult<(Self, bool)> {
        match self.connection_sm.clone() {
            SmConnection::Invitee(sm_invitee) => {
                let (sm_invitee, can_autohop) = match message {
                    Some(message) => match message {
                        A2AMessage::ConnectionInvitationPublic(invitation) => {
                            (sm_invitee.handle_invitation(Invitation::Public(invitation))?, false)
                        }
                        A2AMessage::ConnectionInvitationPairwise(invitation) => {
                            (sm_invitee.handle_invitation(Invitation::Pairwise(invitation))?, false)
                        }
                        A2AMessage::ConnectionResponse(response) => {
                            let send_message = self.send_message_closure_connection(wallet_handle);
                            (sm_invitee.handle_connection_response(response, send_message).await?, true)
                        }
                        A2AMessage::ConnectionProblemReport(problem_report) => {
                            (sm_invitee.handle_problem_report(problem_report)?, false)
                        }
                        _ => (sm_invitee, false),
                    },
                    None => {
                        let send_message = self.send_message_closure_connection(wallet_handle);
                        (sm_invitee.handle_send_ack(send_message).await?, false)
                    }
                };
                let connection = Self {
                    connection_sm: SmConnection::Invitee(sm_invitee),
                    autohop_enabled: self.autohop_enabled,
                };
                Ok((connection, can_autohop))
            }
            SmConnection::Inviter(_) => Err(VcxError::from_msg(
                VcxErrorKind::NotReady,
                "Invalid operation, called \
                _step_invitee on Inviter connection.",
            )),
        }
    }

    async fn handle_disclose(&self, disclose: Disclose) -> VcxResult<SmConnection> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                Ok(SmConnection::Inviter(sm_inviter.clone().handle_disclose(disclose)?))
            }
            SmConnection::Invitee(sm_invitee) => {
                Ok(SmConnection::Invitee(sm_invitee.clone().handle_disclose(disclose)?))
            }
        }
    }

    // ----------------------------- MSG SENDING ------------------------------------
    pub async fn send_response(&mut self, wallet_handle: WalletHandle) -> VcxResult<()> {
        trace!("Connection::send_response >>>");
        let connection_sm = match self.connection_sm.clone() {
            SmConnection::Inviter(sm_inviter) => {
                if let InviterFullState::Requested(_) = sm_inviter.state_object() {
                    sm_inviter.handle_send_response(wallet_handle, &send_message).await?
                } else {
                    return Err(VcxError::from_msg(VcxErrorKind::NotReady, "Invalid action"));
                }
            }
            SmConnection::Invitee(_) => {
                return Err(VcxError::from_msg(VcxErrorKind::NotReady, "Invalid action"));
            }
        };
        self.connection_sm = SmConnection::Inviter(connection_sm);
        Ok(())
    }

    pub async fn connect(&mut self, wallet_handle: WalletHandle, service_endpoint: String) -> VcxResult<()> {
        trace!("Connection::connect >>> source_id: {}", self.source_id());
        self.connection_sm = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => SmConnection::Inviter(sm_inviter.clone().create_invitation(
                vec![],
                service_endpoint,
            )?),
            SmConnection::Invitee(sm_invitee) => {
                SmConnection::Invitee(
                    sm_invitee
                        .clone()
                        .send_connection_request(
                            cloud_agent_info.routing_keys(agency_client)?,
                            cloud_agent_info.service_endpoint(agency_client)?,
                            self.send_message_closure_connection(wallet_handle)
                        )
                        .await?
                )
            }
        };
        Ok(())
    }

    pub async fn update_message_status(&self, uid: &str, agency_client: &AgencyClient) -> VcxResult<()> {
        trace!("Connection::update_message_status >>> uid: {:?}", uid);
        self.cloud_agent_info()
            .ok_or(VcxError::from_msg(
                VcxErrorKind::NoAgentInformation,
                "Missing cloud agent info",
            ))?
            .update_message_status(agency_client, self.pairwise_info(), uid.to_string())
            .await
    }

    pub async fn get_messages_noauth(&self, agency_client: &AgencyClient) -> VcxResult<HashMap<String, A2AMessage>> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                let messages = self
                    .cloud_agent_info()
                    .ok_or(VcxError::from_msg(
                        VcxErrorKind::NoAgentInformation,
                        "Missing cloud agent info",
                    ))?
                    .get_messages_noauth(agency_client, sm_inviter.pairwise_info(), None)
                    .await?;
                Ok(messages)
            }
            SmConnection::Invitee(sm_invitee) => {
                let messages = self
                    .cloud_agent_info()
                    .ok_or(VcxError::from_msg(
                        VcxErrorKind::NoAgentInformation,
                        "Missing cloud agent info",
                    ))?
                    .get_messages_noauth(agency_client, sm_invitee.pairwise_info(), None)
                    .await?;
                Ok(messages)
            }
        }
    }

    pub async fn get_messages(&self, agency_client: &AgencyClient) -> VcxResult<HashMap<String, A2AMessage>> {
        let expected_sender_vk = self.get_expected_sender_vk().await?;
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => Ok(self
                .cloud_agent_info()
                .ok_or(VcxError::from_msg(
                    VcxErrorKind::NoAgentInformation,
                    "Missing cloud agent info",
                ))?
                .get_messages(agency_client, &expected_sender_vk, sm_inviter.pairwise_info())
                .await?),
            SmConnection::Invitee(sm_invitee) => Ok(self
                .cloud_agent_info()
                .ok_or(VcxError::from_msg(
                    VcxErrorKind::NoAgentInformation,
                    "Missing cloud agent info",
                ))?
                .get_messages(agency_client, &expected_sender_vk, sm_invitee.pairwise_info())
                .await?),
        }
    }

    async fn get_expected_sender_vk(&self) -> VcxResult<String> {
        self.remote_vk().await.map_err(|_err| {
            VcxError::from_msg(
                VcxErrorKind::NotReady,
                "Verkey of Connection counterparty \
                is not known, hence it would be impossible to authenticate message downloaded by id.",
            )
        })
    }

    pub async fn get_message_by_id(&self, msg_id: &str, agency_client: &AgencyClient) -> VcxResult<A2AMessage> {
        trace!("Connection: get_message_by_id >>> msg_id: {}", msg_id);
        let expected_sender_vk = self.get_expected_sender_vk().await?;
        self.cloud_agent_info()
            .ok_or(VcxError::from_msg(
                VcxErrorKind::NoAgentInformation,
                "Missing cloud agent info",
            ))?
            .get_message_by_id(agency_client, msg_id, &expected_sender_vk, self.pairwise_info())
            .await
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

    fn build_basic_message(message: &str) -> A2AMessage {
        match ::serde_json::from_str::<A2AMessage>(message) {
            Ok(a2a_message) => a2a_message,
            Err(_) => BasicMessage::create()
                .set_content(message.to_string())
                .set_time()
                .set_out_time()
                .to_a2a_message(),
        }
    }

    pub async fn send_generic_message(&self, wallet_handle: WalletHandle, message: &str) -> VcxResult<String> {
        trace!("Connection::send_generic_message >>> message: {:?}", message);
        let message = Self::build_basic_message(message);
        let send_message = self.send_message_closure(wallet_handle).await?;
        send_message(message).await.map(|_| String::new())
    }

    pub async fn send_a2a_message(&self, wallet_handle: WalletHandle, message: &A2AMessage) -> VcxResult<String> {
        trace!("Connection::send_a2a_message >>> message: {:?}", message);
        let send_message = self.send_message_closure(wallet_handle).await?;
        send_message(message.clone()).await.map(|_| String::new())
    }

    pub async fn send_ping(
        &mut self,
        wallet_handle: WalletHandle,
        comment: Option<String>,
    ) -> VcxResult<TrustPingSender> {
        let mut trust_ping = TrustPingSender::build(true, comment);
        trust_ping
            .send_ping(self.send_message_closure(wallet_handle).await?)
            .await?;
        Ok(trust_ping)
    }

    pub async fn send_handshake_reuse(&self, wallet_handle: WalletHandle, oob_msg: &str) -> VcxResult<()> {
        trace!("Connection::send_handshake_reuse >>>");
        // todo: oob_msg argument should be typed OutOfBandInvitation, not string
        let oob = match serde_json::from_str::<A2AMessage>(oob_msg) {
            Ok(a2a_msg) => match a2a_msg {
                A2AMessage::OutOfBandInvitation(oob) => oob,
                a => {
                    return Err(VcxError::from_msg(
                        VcxErrorKind::SerializationError,
                        format!("Received invalid message type: {:?}", a),
                    ));
                }
            },
            Err(err) => {
                return Err(VcxError::from_msg(
                    VcxErrorKind::SerializationError,
                    format!("Failed to deserialize message, err: {:?}", err),
                ));
            }
        };
        let send_message = self.send_message_closure(wallet_handle).await?;
        send_message(build_handshake_reuse_msg(&oob).to_a2a_message()).await
    }

    pub async fn send_discovery_query(
        &self,
        wallet_handle: WalletHandle,
        query: Option<String>,
        comment: Option<String>,
    ) -> VcxResult<()> {
        trace!(
            "Connection::send_discovery_features_query >>> query: {:?}, comment: {:?}",
            query,
            comment
        );
        let did_doc = self.their_did_doc().await.ok_or(VcxError::from_msg(
            VcxErrorKind::NotReady,
            format!("Can't send handshake-reuse to the counterparty, because their did doc is not available"),
        ))?;
        send_discovery_query(wallet_handle, query, comment, &did_doc, &self.pairwise_info().pw_vk).await?;
        Ok(())
    }

    pub async fn delete(&self) -> VcxResult<()> {
        trace!("Connection: delete >>> {:?}", self.source_id());
        todo!()
    }
}
