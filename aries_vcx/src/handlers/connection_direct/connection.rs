use core::fmt;
use std::clone::Clone;
use std::collections::HashMap;

use futures::future::BoxFuture;
use futures::stream::StreamExt;
use indy_sys::WalletHandle;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, MapAccess, Visitor};
use serde_json::Value;

use crate::error::prelude::*;
use crate::handlers::connection::public_agent::PublicAgent;
use crate::handlers::connection_direct::connection_info::LegacyAgentInfo2;
use crate::messages::a2a::A2AMessage;
use crate::messages::basic_message::message::BasicMessage;
use crate::messages::connection::did_doc::DidDoc;
use crate::messages::connection::invite::Invitation;
use crate::messages::connection::request::Request;
use crate::messages::discovery::disclose::ProtocolDescriptor;
use crate::protocols::connection::invitee::state_machine::{InviteeFullState, InviteeState, SmConnectionInvitee};
use crate::protocols::connection::inviter::state_machine::{InviterFullState, InviterState, SmConnectionInviter};
use crate::protocols::connection::pairwise_info::PairwiseInfo;
use crate::protocols::SendClosure;
use crate::utils::send_message;
use crate::utils::serialization::SerializableObjectWithState;

#[derive(Clone, PartialEq)]
pub struct ConnectionDirect {
    connection_sm: SmConnection,
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

impl ConnectionDirect {
    pub async fn create(source_id: &str, autohop_enabled: bool) -> VcxResult<Self> {
        trace!("Connection::create >>> source_id: {}", source_id);
        // todo: Would be cleaner to pass wallet_handle as argument instead of reading off AgencyClient
        let pairwise_info = PairwiseInfo::create(agency_client.get_wallet_handle()).await?;
        Ok(Self {
            connection_sm: SmConnection::Inviter(SmConnectionInviter::new(source_id, pairwise_info)),
            autohop_enabled,
        })
    }

    pub async fn create_with_invite(source_id: &str, invitation: Invitation, autohop_enabled: bool) -> VcxResult<Self> {
        trace!("Connection::create_with_invite >>> source_id: {}, invitation: {:?}", source_id, invitation);
        let pairwise_info = PairwiseInfo::create(agency_client.get_wallet_handle()).await?;
        let mut connection = Self {
            connection_sm: SmConnection::Invitee(SmConnectionInvitee::new(source_id, pairwise_info)),
            autohop_enabled,
        };
        connection.process_invite(invitation)?;
        Ok(connection)
    }

    pub async fn create_with_request(wallet_handle: WalletHandle, request: Request, public_agent: &PublicAgent, service_endpoint: &str) -> VcxResult<Self> {
        trace!("Connection::create_with_request >>> request: {:?}, public_agent: {:?}", request, public_agent);
        let pairwise_info: PairwiseInfo = public_agent.into();
        let mut connection = Self {
            connection_sm: SmConnection::Inviter(SmConnectionInviter::new(&request.id.0, pairwise_info)),
            autohop_enabled: true,
        };
        connection.process_request(wallet_handle, request, service_endpoint).await
    }

    pub fn from_parts(source_id: String, thread_id: String, pairwise_info: PairwiseInfo, state: SmConnectionState, autohop_enabled: bool) -> Self {
        match state {
            SmConnectionState::Inviter(state) => {
                Self {
                    connection_sm: SmConnection::Inviter(SmConnectionInviter::from(source_id, thread_id, pairwise_info, state)),
                    autohop_enabled,
                }
            }
            SmConnectionState::Invitee(state) => {
                Self {
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

    async fn process_request(&mut self, wallet_handle: WalletHandle, request: Request, new_service_endpoint: &str) -> VcxResult<Self> {
        trace!("Connection::process_request >>> request: {:?}", request);
        let (connection_sm) = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                let new_pairwise_info = PairwiseInfo::create(wallet_handle).await?;
                let new_routing_keys: Vec<String> = vec!();
                (SmConnection::Inviter(sm_inviter.clone().handle_connection_request(wallet_handle, request, &new_pairwise_info, new_routing_keys, new_service_endpoint.into(), send_message).await?))
            }
            SmConnection::Invitee(_) => {
                return Err(VcxError::from_msg(VcxErrorKind::NotReady, "Invalid action"));
            }
        };
        Ok(Self {
            connection_sm,
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
    
    fn _update_state(&mut self, wallet_handle: WalletHandle, message: Option<A2AMessage>, service_endpoint: &str) -> BoxFuture<'_, VcxResult<()>> {
        Box::pin(async move {
            let (new_connection_sm, can_autohop) = match &self.connection_sm {
                SmConnection::Inviter(_) => {
                    self._step_inviter(wallet_handle, message, service_endpoint).await?
                }
                SmConnection::Invitee(_) => {
                    self._step_invitee(wallet_handle, message).await?
                }
            };
            *self = new_connection_sm;
            if can_autohop && self.autohop_enabled.clone() {
                let res = self._update_state(wallet_handle, None, service_endpoint).await;
                res
            } else {
                Ok(())
            }
        })
    }

    pub async fn update_state(&mut self, wallet_handle: WalletHandle) -> VcxResult<()> {
        if self.is_in_null_state() {
            warn!("Connection::update_state :: update state on connection in null state is ignored");
            return Ok(());
        }
        trace!("Connection::update_state >>> trying to update state without message");
        self._update_state(wallet_handle, None, agency_client.clone()).await?;
        Ok(())
    }

    pub async fn update_state_with_message(&mut self, wallet_handle: WalletHandle, message: &A2AMessage, service_endpoint: &str) -> VcxResult<()> {
        trace!("Connection: update_state_with_message: {:?}", message);
        if self.is_in_null_state() {
            warn!("Connection::update_state_with_message :: update state on connection in null state is ignored");
            return Ok(());
        }
        self._update_state(wallet_handle, Some(message.clone()), service_endpoint).await?;
        Ok(())
    }

    async fn _step_inviter(&self, wallet_handle: WalletHandle, message: Option<A2AMessage>, new_service_endpoint: &str) -> VcxResult<(Self, bool)> {
        match self.connection_sm.clone() {
            SmConnection::Inviter(sm_inviter) => {
                let (sm_inviter, can_autohop) = match message {
                    Some(message) => match message {
                        A2AMessage::ConnectionRequest(request) => {
                            let new_pairwise_info = PairwiseInfo::create(wallet_handle).await?;
                            let new_routing_keys: Vector<String> = vec!();
                            let sm_connection = sm_inviter.handle_connection_request(
                                wallet_handle, request, &new_pairwise_info, new_routing_keys, new_service_endpoint.into(), send_message
                            ).await?;
                            (sm_connection, true)
                        }
                        A2AMessage::Ack(ack) => {
                            (sm_inviter.handle_ack(wallet_handle, ack, send_message).await?, false)
                        }
                        A2AMessage::Ping(ping) => {
                            (sm_inviter.handle_ping(wallet_handle, ping, send_message).await?, false)
                        }
                        A2AMessage::ConnectionProblemReport(problem_report) => {
                            (sm_inviter.handle_problem_report(problem_report)?, false)
                        }
                        A2AMessage::PingResponse(ping_response) => {
                            (sm_inviter.handle_ping_response(ping_response)?, false)
                        }
                        A2AMessage::OutOfBandHandshakeReuse(reuse) => {
                            (sm_inviter.handle_handshake_reuse(wallet_handle, reuse, send_message).await?, false)
                        }
                        A2AMessage::Query(query) => {
                            (sm_inviter.handle_discovery_query(wallet_handle, query, send_message).await?, false)
                        }
                        A2AMessage::Disclose(disclose) => {
                            (sm_inviter.handle_disclose(disclose)?, false)
                        }
                        _ => {
                            (sm_inviter.clone(), false)
                        }
                    }
                    None => {
                        if let InviterFullState::Requested(_) = sm_inviter.state_object() {
                            (sm_inviter.handle_send_response(wallet_handle, &send_message).await?, false)
                        } else {
                            (sm_inviter.clone(), false)
                        }
                    }
                };

                let connection = Self {
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

    async fn _step_invitee(&self, wallet_handle: WalletHandle, message: Option<A2AMessage>) -> VcxResult<(Self, bool)> {
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
                            (sm_invitee.handle_ping(wallet_handle, ping, send_message).await?, false)
                        }
                        A2AMessage::ConnectionProblemReport(problem_report) => {
                            (sm_invitee.handle_problem_report(problem_report)?, false)
                        }
                        A2AMessage::PingResponse(ping_response) => {
                            (sm_invitee.handle_ping_response(ping_response)?, false)
                        }
                        A2AMessage::OutOfBandHandshakeReuse(reuse) => {
                            (sm_invitee.handle_handshake_reuse(wallet_handle, reuse, send_message).await?, false)
                        }
                        A2AMessage::Query(query) => {
                            (sm_invitee.handle_discovery_query(wallet_handle, query, send_message).await?, false)
                        }
                        A2AMessage::Disclose(disclose) => {
                            (sm_invitee.handle_disclose(disclose)?, false)
                        }
                        _ => {
                            (sm_invitee.clone(), false)
                        }
                    }
                    None => {
                        (sm_invitee.handle_send_ack(wallet_handle, &send_message).await?, false)
                    }
                };
                let connection = Self {
                    connection_sm: SmConnection::Invitee(sm_invitee),
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

    pub async fn connect(&mut self, wallet_handle: WalletHandle, service_endpoint: &str) -> VcxResult<()> {
        trace!("Connection::connect >>> source_id: {}", self.source_id());
        self.connection_sm = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                SmConnection::Inviter(sm_inviter.clone().handle_connect(vec!(), service_endpoint.into())?)
            }
            SmConnection::Invitee(sm_invitee) => {
                SmConnection::Invitee(sm_invitee.clone().handle_connect(wallet_handle, vec!(), service_endpoint.into(), send_message).await?)
            }
        };
        Ok(())
    }

    fn get_expected_sender_vk(&self) -> VcxResult<String> {
        self.remote_vk()
            .map_err(|_err|
                VcxError::from_msg(VcxErrorKind::NotReady, "Verkey of connection counterparty \
                is not known, hence it would be impossible to authenticate message downloaded by id.")
            )
    }

    pub fn send_message_closure(&self, wallet_handle: WalletHandle) -> VcxResult<SendClosure> {
        trace!("send_message_closure >>>");
        let did_doc = self.their_did_doc()
            .ok_or(VcxError::from_msg(VcxErrorKind::NotReady, "Cannot send message: Remote Connection information is not set"))?;
        let sender_vk = self.pairwise_info().pw_vk.clone();
        Ok(Box::new(move |message: A2AMessage| {
            Box::pin(send_message(wallet_handle, sender_vk.clone(), did_doc.clone(), message))
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

    pub async fn send_generic_message(&self, wallet_handle: WalletHandle, message: &str) -> VcxResult<String> {
        trace!("Connection::send_generic_message >>> message: {:?}", message);
        let message = Self::parse_generic_message(message);
        let send_message = self.send_message_closure(wallet_handle)?;
        send_message(message).await.map(|_| String::new())
    }

    pub async fn send_ping(&mut self, wallet_handle: WalletHandle, comment: Option<String>) -> VcxResult<()> {
        trace!("Connection::send_ping >>> comment: {:?}", comment);
        self.connection_sm = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                SmConnection::Inviter(sm_inviter.clone().handle_send_ping(wallet_handle, comment, send_message).await?)
            }
            SmConnection::Invitee(sm_invitee) => {
                SmConnection::Invitee(sm_invitee.clone().handle_send_ping(wallet_handle, comment, send_message).await?)
            }
        };
        Ok(())
    }

    pub async fn send_handshake_reuse(&self, wallet_handle: WalletHandle, oob_msg: &str) -> VcxResult<()> {
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
                SmConnection::Inviter(sm_inviter.clone().handle_send_handshake_reuse(wallet_handle, oob, send_message).await?)
            }
            SmConnection::Invitee(sm_invitee) => {
                SmConnection::Invitee(sm_invitee.clone().handle_send_handshake_reuse(wallet_handle, oob, send_message).await?)
            }
        };
        Ok(())
    }

    pub async fn send_discovery_features(&mut self, wallet_handle: WalletHandle, query: Option<String>, comment: Option<String>) -> VcxResult<()> {
        trace!("Connection::send_discovery_features_query >>> query: {:?}, comment: {:?}", query, comment);
        self.connection_sm = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                SmConnection::Inviter(sm_inviter.clone().handle_discover_features(wallet_handle, query, comment, send_message).await?)
            }
            SmConnection::Invitee(sm_invitee) => {
                SmConnection::Invitee(sm_invitee.clone().handle_discover_features(wallet_handle, query, comment, send_message).await?)
            }
        };
        Ok(())
    }

    pub fn get_connection_info(&self, service_endpoint: &str) -> VcxResult<String> {
        trace!("Connection::get_connection_info >>>");

        let pairwise_info = self.pairwise_info();
        let recipient_keys = vec!(pairwise_info.pw_vk.clone());

        let current = SideConnectionInfo {
            did: pairwise_info.pw_did.clone(),
            recipient_keys: recipient_keys.clone(),
            routing_keys: vec!(),
            service_endpoint: service_endpoint.into(),
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

    pub fn to_string(&self) -> VcxResult<String> {
        serde_json::to_string(&self)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize Connection: {:?}", err)))
    }

    pub fn from_string(connection_data: &str) -> VcxResult<Self> {
        serde_json::from_str(connection_data)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize Connection: {:?}", err)))
    }
}

impl Serialize for ConnectionDirect {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let (state, pairwise_info, source_id, thread_id) = self.to_owned().into();
        let data = LegacyAgentInfo2 {
            pw_did: pairwise_info.pw_did,
            pw_vk: pairwise_info.pw_vk,
        };
        let object = SerializableObjectWithState::V1 { data, state, source_id, thread_id };
        serializer.serialize_some(&object)
    }
}

struct ConnectionVisitor;

impl<'de> Visitor<'de> for ConnectionVisitor {
    type Value = ConnectionDirect;

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
        let ver: SerializableObjectWithState<LegacyAgentInfo2, SmConnectionState> = serde_json::from_value(obj)
            .map_err(|err| A::Error::custom(err.to_string()))?;
        match ver {
            SerializableObjectWithState::V1 { data, state, source_id, thread_id } => {
                let pairwise_info = PairwiseInfo { pw_did: data.pw_did, pw_vk: data.pw_vk };
                Ok((state, pairwise_info, source_id, thread_id).into())
            }
        }
    }
}

impl<'de> Deserialize<'de> for ConnectionDirect {
    fn deserialize<D>(deserializer: D) -> Result<ConnectionDirect, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ConnectionVisitor)
    }
}

impl Into<(SmConnectionState, PairwiseInfo, String, String)> for ConnectionDirect {
    fn into(self) -> (SmConnectionState, PairwiseInfo, String, String) {
        (self.state_object(), self.pairwise_info().to_owned(), self.source_id(), self.get_thread_id())
    }
}

impl From<(SmConnectionState, PairwiseInfo, String, String)> for ConnectionDirect {
    fn from((state, pairwise_info, source_id, thread_id): (SmConnectionState, PairwiseInfo, String, String)) -> ConnectionDirect {
        ConnectionDirect::from_parts(source_id, thread_id, pairwise_info, state, true)
    }
}

#[cfg(test)]
mod tests {
    use indy_sys::WalletHandle;

    use crate::handlers::connection::public_agent::tests::_public_agent;
    use crate::messages::connection::invite::test_utils::{_pairwise_invitation, _pairwise_invitation_random_id, _public_invitation, _public_invitation_random_id};
    use crate::messages::connection::request::tests::_request;
    use crate::utils::devsetup::SetupMocks;

    use super::*;
    
    fn _service_endpoint() -> &'static str {
        "http://localhost:8080"
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_create_with_pairwise_invite() {
        let _setup = SetupMocks::init();
        let connection = ConnectionDirect::create_with_invite("abc", Invitation::Pairwise(_pairwise_invitation()), true).await.unwrap();
        assert_eq!(connection.get_state(), ConnectionState::Invitee(InviteeState::Invited));
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_create_with_public_invite() {
        let _setup = SetupMocks::init();
        let connection = ConnectionDirect::create_with_invite("abc", Invitation::Public(_public_invitation()), true).await.unwrap();
        assert_eq!(connection.get_state(), ConnectionState::Invitee(InviteeState::Invited));
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_connect_sets_correct_thread_id_based_on_invitation_type() {
        let _setup = SetupMocks::init();

        let pub_inv = _public_invitation_random_id();
        let mut connection = ConnectionDirect::create_with_invite("abcd", Invitation::Public(pub_inv.clone()), true).await.unwrap();
        connection.connect(WalletHandle(0), _service_endpoint()).await.unwrap();
        assert_eq!(connection.get_state(), ConnectionState::Invitee(InviteeState::Requested));
        assert_ne!(connection.get_thread_id(), pub_inv.id.0);

        let pw_inv = _pairwise_invitation_random_id();
        let mut connection = ConnectionDirect::create_with_invite("dcba", Invitation::Pairwise(pw_inv.clone()), true).await.unwrap();
        connection.connect(WalletHandle(0), _service_endpoint()).await.unwrap();
        assert_eq!(connection.get_state(), ConnectionState::Invitee(InviteeState::Requested));
        assert_eq!(connection.get_thread_id(), pw_inv.id.0);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_create_with_request() {
        let _setup = SetupMocks::init();
        let connection = ConnectionDirect::create_with_request(WalletHandle(0), _request(), &_public_agent(), _service_endpoint()).await.unwrap();
        assert_eq!(connection.get_state(), ConnectionState::Inviter(InviterState::Requested));
    }
}
