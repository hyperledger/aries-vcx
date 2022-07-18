use core::fmt;
use std::clone::Clone;
use std::collections::HashMap;

use futures::future::BoxFuture;
use futures::stream::StreamExt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, MapAccess, Visitor};
use serde_json::Value;

use agency_client::api::downloaded_message::DownloadedMessage;
use agency_client::MessageStatusCode;

use crate::error::prelude::*;
use crate::handlers::connection::cloud_agent::CloudAgentInfo;
use crate::handlers::connection::legacy_agent_info::LegacyAgentInfo;
use crate::handlers::connection::public_agent::PublicAgent;
use crate::libindy::utils::wallet::get_main_wallet_handle;
use crate::protocols::SendClosure;
use crate::messages::a2a::A2AMessage;
use crate::messages::basic_message::message::BasicMessage;
use crate::messages::connection::did_doc::DidDoc;
use crate::messages::connection::invite::Invitation;
use crate::messages::connection::request::Request;
use crate::messages::discovery::disclose::ProtocolDescriptor;
use crate::protocols::connection::invitee::state_machine::{InviteeFullState, InviteeState, SmConnectionInvitee};
use crate::protocols::connection::inviter::state_machine::{InviterFullState, InviterState, SmConnectionInviter};
use crate::protocols::connection::pairwise_info::PairwiseInfo;
use crate::utils::send_message;
use crate::utils::serialization::SerializableObjectWithState;

#[derive(Clone, PartialEq)]
pub struct Connection {
    connection_sm: SmConnection,
    cloud_agent_info: CloudAgentInfo,
    autohop_enabled: bool,
}

#[derive(Clone, PartialEq)]
pub enum SmConnection
{
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
    pub async fn create(source_id: &str, autohop_enabled: bool) -> VcxResult<Self> {
        trace!("Connection::create >>> source_id: {}", source_id);
        let pairwise_info = PairwiseInfo::create().await?;
        let cloud_agent_info = CloudAgentInfo::create(&pairwise_info).await?;
        Ok(Self {
            cloud_agent_info,
            connection_sm: SmConnection::Inviter(SmConnectionInviter::new(source_id, pairwise_info)),
            autohop_enabled,
        })
    }

    pub async fn create_with_invite(source_id: &str, invitation: Invitation, autohop_enabled: bool) -> VcxResult<Self> {
        trace!("Connection::create_with_invite >>> source_id: {}, invitation: {:?}", source_id, invitation);
        let pairwise_info = PairwiseInfo::create().await?;
        let cloud_agent_info = CloudAgentInfo::create(&pairwise_info).await?;
        let mut connection = Self {
            cloud_agent_info,
            connection_sm: SmConnection::Invitee(SmConnectionInvitee::new(source_id, pairwise_info)),
            autohop_enabled,
        };
        connection.process_invite(invitation)?;
        Ok(connection)
    }

    pub async fn create_with_request(request: Request, public_agent: &PublicAgent) -> VcxResult<Self> {
        trace!("Connection::create_with_request >>> request: {:?}, public_agent: {:?}", request, public_agent);
        let pairwise_info: PairwiseInfo = public_agent.into();
        let mut connection = Self {
            cloud_agent_info: public_agent.cloud_agent_info(),
            connection_sm: SmConnection::Inviter(SmConnectionInviter::new(&request.id.0, pairwise_info)),
            autohop_enabled: true,
        };
        connection.process_request(request).await
    }

    pub fn from_parts(source_id: String, thread_id: String, pairwise_info: PairwiseInfo, cloud_agent_info: CloudAgentInfo, state: SmConnectionState, autohop_enabled: bool) -> Self {
        match state {
            SmConnectionState::Inviter(state) => {
                Self {
                    cloud_agent_info,
                    connection_sm: SmConnection::Inviter(SmConnectionInviter::from(source_id, thread_id, pairwise_info, state)),
                    autohop_enabled,
                }
            }
            SmConnectionState::Invitee(state) => {
                Self {
                    cloud_agent_info,
                    connection_sm: SmConnection::Invitee(SmConnectionInvitee::from(source_id, thread_id, pairwise_info, state)),
                    autohop_enabled,
                }
            }
        }
    }

    pub fn source_id(&self) -> String {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                sm_inviter.source_id()
            }
            SmConnection::Invitee(sm_invitee) => {
                sm_invitee.source_id()
            }
        }.into()
    }

    pub fn get_thread_id(&self) -> String {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                sm_inviter.get_thread_id()
            }
            SmConnection::Invitee(sm_invitee) => {
                sm_invitee.get_thread_id()
            }
        }.into()
    }

    pub fn get_state(&self) -> ConnectionState {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                ConnectionState::Inviter(sm_inviter.get_state())
            }
            SmConnection::Invitee(sm_invitee) => {
                ConnectionState::Invitee(sm_invitee.get_state())
            }
        }
    }

    pub fn pairwise_info(&self) -> &PairwiseInfo {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                sm_inviter.pairwise_info()
            }
            SmConnection::Invitee(sm_invitee) => {
                sm_invitee.pairwise_info()
            }
        }
    }

    pub fn cloud_agent_info(&self) -> CloudAgentInfo {
        self.cloud_agent_info.clone()
    }

    // pub fn bootstrap_agent_info(&self) -> Option<&PairwiseInfo> {
    //     match &self.connection_sm {
    //         SmConnection::Inviter(sm_inviter) => {
    //             sm_inviter.prev_agent_info()
    //         }
    //         SmConnection::Invitee(_sm_invitee) => None
    //     }
    // }

    pub fn remote_did(&self) -> VcxResult<String> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                sm_inviter.remote_did()
            }
            SmConnection::Invitee(sm_invitee) => {
                sm_invitee.remote_did()
            }
        }
    }

    pub fn remote_vk(&self) -> VcxResult<String> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                sm_inviter.remote_vk()
            }
            SmConnection::Invitee(sm_invitee) => {
                sm_invitee.remote_vk()
            }
        }
    }

    pub fn state_object(&self) -> SmConnectionState {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                SmConnectionState::Inviter(sm_inviter.state_object().clone())
            }
            SmConnection::Invitee(sm_invitee) => {
                SmConnectionState::Invitee(sm_invitee.state_object().clone())
            }
        }
    }

    pub fn get_source_id(&self) -> String {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                sm_inviter.source_id()
            }
            SmConnection::Invitee(sm_invitee) => {
                sm_invitee.source_id()
            }
        }.to_string()
    }

    pub fn get_protocols(&self) -> Vec<ProtocolDescriptor> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                sm_inviter.get_protocols()
            }
            SmConnection::Invitee(sm_invitee) => {
                sm_invitee.get_protocols()
            }
        }
    }

    pub fn get_remote_protocols(&self) -> Option<Vec<ProtocolDescriptor>> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                sm_inviter.get_remote_protocols()
            }
            SmConnection::Invitee(sm_invitee) => {
                sm_invitee.get_remote_protocols()
            }
        }
    }

    pub fn is_in_null_state(&self) -> bool {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                sm_inviter.is_in_null_state()
            }
            SmConnection::Invitee(sm_invitee) => {
                sm_invitee.is_in_null_state()
            }
        }
    }

    pub fn their_did_doc(&self) -> Option<DidDoc> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                sm_inviter.their_did_doc()
            }
            SmConnection::Invitee(sm_invitee) => {
                sm_invitee.their_did_doc()
            }
        }
    }

    pub fn bootstrap_did_doc(&self) -> Option<DidDoc> {
        match &self.connection_sm {
            SmConnection::Inviter(_sm_inviter) => None, // TODO: Inviter can remember bootstrap agent too, but we don't need it
            SmConnection::Invitee(sm_invitee) => sm_invitee.bootstrap_did_doc()
        }
    }

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

    async fn process_request(&mut self, request: Request) -> VcxResult<Self> {
        trace!("Connection::process_request >>> request: {:?}", request);
        let (connection_sm, new_cloud_agent_info) = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                let new_pairwise_info = PairwiseInfo::create().await?;
                let new_cloud_agent = CloudAgentInfo::create(&new_pairwise_info).await?;
                let new_routing_keys = new_cloud_agent.routing_keys()?;
                let new_service_endpoint = new_cloud_agent.service_endpoint()?;
                (SmConnection::Inviter(sm_inviter.clone().handle_connection_request(request, &new_pairwise_info, new_routing_keys, new_service_endpoint, send_message).await?), new_cloud_agent)
            }
            SmConnection::Invitee(_) => {
                return Err(VcxError::from_msg(VcxErrorKind::NotReady, "Invalid action"));
            }
        };
        Ok(Self {
            connection_sm,
            cloud_agent_info: new_cloud_agent_info,
            autohop_enabled: self.autohop_enabled,
        })
    }

    pub fn get_invite_details(&self) -> Option<&Invitation> {
        trace!("Connection::get_invite_details >>>");
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                sm_inviter.get_invitation().clone()
            }
            SmConnection::Invitee(_sm_invitee) => {
                None
            }
        }
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                sm_inviter.find_message_to_handle(messages)
            }
            SmConnection::Invitee(sm_invitee) => {
                sm_invitee.find_message_to_handle(messages)
            }
        }
    }

    pub fn needs_message(&self) -> bool {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                sm_inviter.needs_message()
            }
            SmConnection::Invitee(sm_invitee) => {
                sm_invitee.needs_message()
            }
        }
    }

    // fn _get_bootstrap_agent_messages(&self, remote_vk: VcxResult<String>, bootstrap_agent_info: Option<&PairwiseInfo>) -> VcxResult<Option<(HashMap<String, A2AMessage>, PairwiseInfo)>> {
    //     let expected_sender_vk = match remote_vk {
    //         Ok(vk) => vk,
    //         Err(_) => return Ok(None)
    //     };
    //     if let Some(bootstrap_agent_info) = bootstrap_agent_info {
    //         trace!("Connection::_get_bootstrap_agent_messages >>> Inviter found no message to handle on main connection agent. Will check bootstrap agent.");
    //         let messages = bootstrap_agent_info.get_messages(&expected_sender_vk)?;
    //         return Ok(Some((messages, bootstrap_agent_info.clone())));
    //     }
    //     Ok(None)
    // }

    fn _update_state(&mut self, message: Option<A2AMessage>) -> BoxFuture<'_, VcxResult<()>> {
        Box::pin(async move {
            let (new_connection_sm, can_autohop) = match &self.connection_sm {
                SmConnection::Inviter(_) => {
                    self._step_inviter(message).await?
                }
                SmConnection::Invitee(_) => {
                    self._step_invitee(message).await?
                }
            };
            *self = new_connection_sm;
            if can_autohop && self.autohop_enabled.clone() {
                let res = self._update_state(None).await;
                res
            } else {
                Ok(())
            }
        })
    }

    pub async fn update_state(&mut self) -> VcxResult<()> {
        if self.is_in_null_state() {
            warn!("Connection::update_state :: update state on connection in null state is ignored");
            return Ok(());
        }

        let messages = self.get_messages_noauth().await?;
        trace!("Connection::update_state >>> retrieved messages {:?}", messages);

        match self.find_message_to_handle(messages) {
            Some((uid, message)) => {
                trace!("Connection::update_state >>> handling message uid: {:?}", uid);
                self._update_state(Some(message)).await?;
                self.cloud_agent_info().clone().update_message_status(self.pairwise_info(), uid).await?;
            }
            None => {
                // Todo: Restore lookup into bootstrap cloud agent
                // self.bootstrap_agent_info()
                // if let Some((messages, bootstrap_agent_info)) = self._get_bootstrap_agent_messages(self.remote_vk(), )? {
                //     if let Some((uid, message)) = self.find_message_to_handle(messages) {
                //         trace!("Connection::update_state >>> handling message found on bootstrap agent uid: {:?}", uid);
                //         self._update_state(Some(message))?;
                //         bootstrap_agent_info.update_message_status(uid)?;
                //     }
                // } else {
                trace!("Connection::update_state >>> trying to update state without message");
                self._update_state(None).await?;
                // }
            }
        }
        Ok(())
    }

    pub async fn update_state_with_message(&mut self, message: &A2AMessage) -> VcxResult<()> {
        trace!("Connection: update_state_with_message: {:?}", message);
        if self.is_in_null_state() {
            warn!("Connection::update_state_with_message :: update state on connection in null state is ignored");
            return Ok(());
        }
        self._update_state(Some(message.clone())).await?;
        Ok(())
    }

    async fn _step_inviter(&self, message: Option<A2AMessage>) -> VcxResult<(Self, bool)> {
        match self.connection_sm.clone() {
            SmConnection::Inviter(sm_inviter) => {
                let (sm_inviter, new_cloud_agent_info, can_autohop) = match message {
                    Some(message) => match message {
                        A2AMessage::ConnectionRequest(request) => {
                            let new_pairwise_info = PairwiseInfo::create().await?;
                            let new_cloud_agent = CloudAgentInfo::create(&new_pairwise_info).await?;
                            let new_routing_keys = new_cloud_agent.routing_keys()?;
                            let new_service_endpoint = new_cloud_agent.service_endpoint()?;
                            let sm_connection = sm_inviter.handle_connection_request(request, &new_pairwise_info, new_routing_keys, new_service_endpoint, send_message).await?;
                            (sm_connection, Some(new_cloud_agent), true)
                        }
                        A2AMessage::Ack(ack) => {
                            (sm_inviter.handle_ack(ack, send_message).await?, None, false)
                        }
                        A2AMessage::Ping(ping) => {
                            (sm_inviter.handle_ping(ping, send_message).await?, None, false)
                        }
                        A2AMessage::ConnectionProblemReport(problem_report) => {
                            (sm_inviter.handle_problem_report(problem_report)?, None, false)
                        }
                        A2AMessage::PingResponse(ping_response) => {
                            (sm_inviter.handle_ping_response(ping_response)?, None, false)
                        }
                        A2AMessage::OutOfBandHandshakeReuse(reuse) => {
                            (sm_inviter.handle_handshake_reuse(reuse, send_message).await?, None, false)
                        }
                        A2AMessage::Query(query) => {
                            (sm_inviter.handle_discovery_query(query, send_message).await?, None, false)
                        }
                        A2AMessage::Disclose(disclose) => {
                            (sm_inviter.handle_disclose(disclose)?, None, false)
                        }
                        _ => {
                            (sm_inviter.clone(), None, false)
                        }
                    }
                    None => {
                        if let InviterFullState::Requested(_) = sm_inviter.state_object() {
                            (sm_inviter.handle_send_response(&send_message).await?, None, false)
                        } else {
                            (sm_inviter.clone(), None, false)
                        }
                    }
                };

                let connection = Self {
                    cloud_agent_info: new_cloud_agent_info.unwrap_or(self.cloud_agent_info.clone()),
                    connection_sm: SmConnection::Inviter(sm_inviter),
                    autohop_enabled: self.autohop_enabled.clone(),
                };

                Ok((connection, can_autohop))
            }
            SmConnection::Invitee(_) => {
                Err(VcxError::from_msg(VcxErrorKind::NotReady, "Invalid operation, called \
                _step_inviter on Invitee connection."))
            }
        }
    }

    async fn _step_invitee(&self, message: Option<A2AMessage>) -> VcxResult<(Self, bool)> {
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
                            (sm_invitee.handle_connection_response(response)?, true)
                        }
                        A2AMessage::Ack(ack) => {
                            (sm_invitee.handle_ack(ack)?, false)
                        }
                        A2AMessage::Ping(ping) => {
                            (sm_invitee.handle_ping(ping, send_message).await?, false)
                        }
                        A2AMessage::ConnectionProblemReport(problem_report) => {
                            (sm_invitee.handle_problem_report(problem_report)?, false)
                        }
                        A2AMessage::PingResponse(ping_response) => {
                            (sm_invitee.handle_ping_response(ping_response)?, false)
                        }
                        A2AMessage::OutOfBandHandshakeReuse(reuse) => {
                            (sm_invitee.handle_handshake_reuse(reuse, send_message).await?, false)
                        }
                        A2AMessage::Query(query) => {
                            (sm_invitee.handle_discovery_query(query, send_message).await?, false)
                        }
                        A2AMessage::Disclose(disclose) => {
                            (sm_invitee.handle_disclose(disclose)?, false)
                        }
                        _ => {
                            (sm_invitee.clone(), false)
                        }
                    }
                    None => {
                        (sm_invitee.handle_send_ack(&send_message).await?, false)
                    }
                };
                let connection = Self {
                    connection_sm: SmConnection::Invitee(sm_invitee),
                    cloud_agent_info: self.cloud_agent_info.clone(),
                    autohop_enabled: self.autohop_enabled.clone(),
                };
                Ok((connection, can_autohop))
            }
            SmConnection::Inviter(_) => {
                Err(VcxError::from_msg(VcxErrorKind::NotReady, "Invalid operation, called \
                _step_invitee on Inviter connection."))
            }
        }
    }

    pub async fn connect(&mut self) -> VcxResult<()> {
        trace!("Connection::connect >>> source_id: {}", self.source_id());
        self.connection_sm = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                SmConnection::Inviter(sm_inviter.clone().handle_connect(self.cloud_agent_info.routing_keys()?, self.cloud_agent_info.service_endpoint()?)?)
            }
            SmConnection::Invitee(sm_invitee) => {
                SmConnection::Invitee(sm_invitee.clone().handle_connect(self.cloud_agent_info.routing_keys()?, self.cloud_agent_info.service_endpoint()?, send_message).await?)
            }
        };
        Ok(())
    }

    pub async fn update_message_status(&self, uid: &str) -> VcxResult<()> {
        trace!("Connection::update_message_status >>> uid: {:?}", uid);
        self.cloud_agent_info().update_message_status(self.pairwise_info(), uid.to_string()).await
    }

    pub async fn get_messages_noauth(&self) -> VcxResult<HashMap<String, A2AMessage>> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                let messages = self.cloud_agent_info().get_messages_noauth(sm_inviter.pairwise_info(), None).await?;
                Ok(messages)
            }
            SmConnection::Invitee(sm_invitee) => {
                let messages = self.cloud_agent_info().get_messages_noauth(sm_invitee.pairwise_info(), None).await?;
                Ok(messages)
            }
        }
    }

    pub async fn get_messages(&self) -> VcxResult<HashMap<String, A2AMessage>> {
        let expected_sender_vk = self.get_expected_sender_vk()?;
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                Ok(self.cloud_agent_info().get_messages(&expected_sender_vk, sm_inviter.pairwise_info()).await?)
            }
            SmConnection::Invitee(sm_invitee) => {
                Ok(self.cloud_agent_info().get_messages(&expected_sender_vk, sm_invitee.pairwise_info()).await?)
            }
        }
    }

    fn get_expected_sender_vk(&self) -> VcxResult<String> {
        self.remote_vk()
            .map_err(|_err|
                VcxError::from_msg(VcxErrorKind::NotReady, "Verkey of connection counterparty \
                is not known, hence it would be impossible to authenticate message downloaded by id.")
            )
    }

    pub async fn get_message_by_id(&self, msg_id: &str) -> VcxResult<A2AMessage> {
        trace!("Connection: get_message_by_id >>> msg_id: {}", msg_id);
        let expected_sender_vk = self.get_expected_sender_vk()?;
        self.cloud_agent_info().get_message_by_id(msg_id, &expected_sender_vk, self.pairwise_info()).await
    }

    pub fn send_message_closure(&self) -> VcxResult<SendClosure> {
        trace!("send_message_closure >>>");
        let did_doc = self.their_did_doc()
            .ok_or(VcxError::from_msg(VcxErrorKind::NotReady, "Cannot send message: Remote Connection information is not set"))?;
        let sender_vk = self.pairwise_info().pw_vk.clone();
        Ok(Box::new(move |message: A2AMessage| {
            Box::pin(send_message(sender_vk.clone(), did_doc.clone(), message))
        }))
    }

    fn parse_generic_message(message: &str) -> A2AMessage {
        match ::serde_json::from_str::<A2AMessage>(message) {
            Ok(a2a_message) => a2a_message,
            Err(_) => {
                BasicMessage::create()
                    .set_content(message.to_string())
                    .set_time()
                    .to_a2a_message()
            }
        }
    }

    pub async fn send_generic_message(&self, message: &str) -> VcxResult<String> {
        trace!("Connection::send_generic_message >>> message: {:?}", message);
        let message = Self::parse_generic_message(message);
        let send_message = self.send_message_closure()?;
        send_message(message).await.map(|_| String::new())
    }

    pub async fn send_ping(&mut self, comment: Option<String>) -> VcxResult<()> {
        trace!("Connection::send_ping >>> comment: {:?}", comment);
        self.connection_sm = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                SmConnection::Inviter(sm_inviter.clone().handle_send_ping(comment, send_message).await?)
            }
            SmConnection::Invitee(sm_invitee) => {
                SmConnection::Invitee(sm_invitee.clone().handle_send_ping(comment, send_message).await?)
            }
        };
        Ok(())
    }

    pub async fn send_handshake_reuse(&self, oob_msg: &str) -> VcxResult<()> {
        trace!("Connection::send_handshake_reuse >>>");
        let oob = match serde_json::from_str::<A2AMessage>(oob_msg) {
            Ok(a2a_msg) => match a2a_msg {
                A2AMessage::OutOfBandInvitation(oob) => oob,
                a @ _ => { return Err(VcxError::from_msg(VcxErrorKind::SerializationError, format!("Received invalid message type: {:?}", a))); }
            }
            Err(err) => { return Err(VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to deserialize message, err: {:?}", err))); }
        };
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                SmConnection::Inviter(sm_inviter.clone().handle_send_handshake_reuse(oob, send_message).await?)
            }
            SmConnection::Invitee(sm_invitee) => {
                SmConnection::Invitee(sm_invitee.clone().handle_send_handshake_reuse(oob, send_message).await?)
            }
        };
        Ok(())
    }

    pub async fn delete(&self) -> VcxResult<()> {
        trace!("Connection: delete >>> {:?}", self.source_id());
        self.cloud_agent_info().destroy(self.pairwise_info()).await
    }

    pub async fn send_discovery_features(&mut self, query: Option<String>, comment: Option<String>) -> VcxResult<()> {
        trace!("Connection::send_discovery_features_query >>> query: {:?}, comment: {:?}", query, comment);
        self.connection_sm = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                SmConnection::Inviter(sm_inviter.clone().handle_discover_features(query, comment, send_message).await?)
            }
            SmConnection::Invitee(sm_invitee) => {
                SmConnection::Invitee(sm_invitee.clone().handle_discover_features(query, comment, send_message).await?)
            }
        };
        Ok(())
    }

    pub fn get_connection_info(&self) -> VcxResult<String> {
        trace!("Connection::get_connection_info >>>");

        let agent_info = self.cloud_agent_info().clone();
        let pairwise_info = self.pairwise_info();
        let recipient_keys = vec!(pairwise_info.pw_vk.clone());

        let current = SideConnectionInfo {
            did: pairwise_info.pw_did.clone(),
            recipient_keys: recipient_keys.clone(),
            routing_keys: agent_info.routing_keys()?,
            service_endpoint: agent_info.service_endpoint()?,
            protocols: Some(self.get_protocols()),
        };

        let remote = match self.their_did_doc() {
            Some(did_doc) =>
                Some(SideConnectionInfo {
                    did: did_doc.id.clone(),
                    recipient_keys: did_doc.recipient_keys(),
                    routing_keys: did_doc.routing_keys(),
                    service_endpoint: did_doc.get_endpoint(),
                    protocols: self.get_remote_protocols(),
                }),
            None => None
        };

        let connection_info = ConnectionInfo { my: current, their: remote };

        let connection_info_json = serde_json::to_string(&connection_info)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidState, format!("Cannot serialize ConnectionInfo: {:?}", err)))?;

        return Ok(connection_info_json);
    }

    pub async fn download_messages(&self, status_codes: Option<Vec<MessageStatusCode>>, uids: Option<Vec<String>>) -> VcxResult<Vec<DownloadedMessage>> {
        let wallet_handle = get_main_wallet_handle().0; // todo: agency client wallet handle should be used
        match self.get_state() {
            ConnectionState::Invitee(InviteeState::Initial) |
            ConnectionState::Inviter(InviterState::Initial) |
            ConnectionState::Inviter(InviterState::Invited) => {
                let msgs = futures::stream::iter(self.cloud_agent_info()
                    .download_encrypted_messages(uids, status_codes, self.pairwise_info())
                    .await?)
                    .then(|msg| msg.decrypt_noauth(wallet_handle) )
                    .filter_map(|res| async { res.ok() })
                    .collect::<Vec<DownloadedMessage>>()
                    .await;
                Ok(msgs)
            }
            _ => {
                let expected_sender_vk = self.remote_vk()?;
                let msgs = futures::stream::iter(self.cloud_agent_info()
                    .download_encrypted_messages(uids, status_codes, self.pairwise_info())
                    .await?)
                    .then(|msg| msg.decrypt_auth(wallet_handle, &expected_sender_vk))
                    .filter_map(|res| async { res.ok() })
                    .collect::<Vec<DownloadedMessage>>()
                    .await;
                Ok(msgs)
            }
        }
    }

    pub fn to_string(&self) -> VcxResult<String> {
        serde_json::to_string(&self)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize Connection: {:?}", err)))
    }

    pub fn from_string(connection_data: &str) -> VcxResult<Self> {
        serde_json::from_str(connection_data)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize Connection: {:?}", err)))
    }
}

impl Serialize for Connection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let (state, pairwise_info, cloud_agent_info, source_id, thread_id) = self.to_owned().into();
        let data = LegacyAgentInfo {
            pw_did: pairwise_info.pw_did,
            pw_vk: pairwise_info.pw_vk,
            agent_did: cloud_agent_info.agent_did,
            agent_vk: cloud_agent_info.agent_vk,
        };
        let object = SerializableObjectWithState::V1 { data, state, source_id, thread_id };
        serializer.serialize_some(&object)
    }
}

struct ConnectionVisitor;

impl<'de> Visitor<'de> for ConnectionVisitor {
    type Value = Connection;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("serialized Connection object")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, <A as MapAccess<'de>>::Error>
        where
            A: MapAccess<'de>
    {
        let mut map_value = serde_json::Map::new();
        while let Some(key) = map.next_key()? {
            let k: String = key;
            let v: Value = map.next_value()?;
            map_value.insert(k, v);
        }
        let obj = Value::from(map_value);
        let ver: SerializableObjectWithState<LegacyAgentInfo, SmConnectionState> = serde_json::from_value(obj)
            .map_err(|err| A::Error::custom(err.to_string()))?;
        match ver {
            SerializableObjectWithState::V1 { data, state, source_id, thread_id } => {
                let pairwise_info = PairwiseInfo { pw_did: data.pw_did, pw_vk: data.pw_vk };
                let cloud_agent_info = CloudAgentInfo { agent_did: data.agent_did, agent_vk: data.agent_vk };
                Ok((state, pairwise_info, cloud_agent_info, source_id, thread_id).into())
            }
        }
    }
}

impl<'de> Deserialize<'de> for Connection {
    fn deserialize<D>(deserializer: D) -> Result<Connection, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ConnectionVisitor)
    }
}

impl Into<(SmConnectionState, PairwiseInfo, CloudAgentInfo, String, String)> for Connection {
    fn into(self) -> (SmConnectionState, PairwiseInfo, CloudAgentInfo, String, String) {
        (self.state_object(), self.pairwise_info().to_owned(), self.cloud_agent_info().to_owned(), self.source_id(), self.get_thread_id())
    }
}

impl From<(SmConnectionState, PairwiseInfo, CloudAgentInfo, String, String)> for Connection {
    fn from((state, pairwise_info, cloud_agent_info, source_id, thread_id): (SmConnectionState, PairwiseInfo, CloudAgentInfo, String, String)) -> Connection {
        Connection::from_parts(source_id, thread_id, pairwise_info, cloud_agent_info, state, true)
    }
}

#[cfg(test)]
mod tests {
    use crate::handlers::connection::public_agent::tests::_public_agent;
    use crate::messages::connection::invite::test_utils::{_pairwise_invitation, _pairwise_invitation_random_id, _public_invitation, _public_invitation_random_id};
    use crate::messages::connection::request::tests::_request;
    use crate::utils::devsetup::SetupMocks;

    use super::*;

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_create_with_pairwise_invite() {
        let _setup = SetupMocks::init();
        let connection = Connection::create_with_invite("abc", Invitation::Pairwise(_pairwise_invitation()), true).await.unwrap();
        assert_eq!(connection.get_state(), ConnectionState::Invitee(InviteeState::Invited));
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_create_with_public_invite() {
        let _setup = SetupMocks::init();
        let connection = Connection::create_with_invite("abc", Invitation::Public(_public_invitation()), true).await.unwrap();
        assert_eq!(connection.get_state(), ConnectionState::Invitee(InviteeState::Invited));
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_connect_sets_correct_thread_id_based_on_invitation_type() {
        let _setup = SetupMocks::init();

        let pub_inv = _public_invitation_random_id();
        let mut connection = Connection::create_with_invite("abcd", Invitation::Public(pub_inv.clone()), true).await.unwrap();
        connection.connect().await.unwrap();
        assert_eq!(connection.get_state(), ConnectionState::Invitee(InviteeState::Requested));
        assert_ne!(connection.get_thread_id(), pub_inv.id.0);

        let pw_inv = _pairwise_invitation_random_id();
        let mut connection = Connection::create_with_invite("dcba", Invitation::Pairwise(pw_inv.clone()), true).await.unwrap();
        connection.connect().await.unwrap();
        assert_eq!(connection.get_state(), ConnectionState::Invitee(InviteeState::Requested));
        assert_eq!(connection.get_thread_id(), pw_inv.id.0);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_create_with_request() {
        let _setup = SetupMocks::init();
        let connection = Connection::create_with_request(_request(), &_public_agent()).await.unwrap();
        assert_eq!(connection.get_state(), ConnectionState::Inviter(InviterState::Requested));
    }
}
