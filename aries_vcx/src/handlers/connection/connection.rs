use core::fmt;
use std::clone::Clone;
use std::collections::HashMap;

use futures::future::BoxFuture;
use futures::stream::StreamExt;
use indy_sys::{WalletHandle, PoolHandle};
use serde::de::{Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use agency_client::agency_client::AgencyClient;
use agency_client::api::downloaded_message::DownloadedMessage;
use agency_client::MessageStatusCode;

use crate::did_doc::DidDoc;
use crate::error::prelude::*;
use crate::handlers::connection::cloud_agent::CloudAgentInfo;
use crate::handlers::connection::legacy_agent_info::LegacyAgentInfo;
use crate::handlers::connection::public_agent::PublicAgent;
use crate::handlers::discovery::{respond_discovery_query, send_discovery_query};
use crate::handlers::trust_ping::TrustPingSender;
use crate::messages::a2a::protocol_registry::ProtocolRegistry;
use crate::messages::a2a::A2AMessage;
use crate::messages::basic_message::message::BasicMessage;
use crate::messages::connection::invite::Invitation;
use crate::messages::connection::request::Request;
use crate::messages::discovery::disclose::{Disclose, ProtocolDescriptor};
use crate::protocols::connection::invitee::state_machine::{InviteeFullState, InviteeState, SmConnectionInvitee};
use crate::protocols::connection::inviter::state_machine::{InviterFullState, InviterState, SmConnectionInviter};
use crate::protocols::connection::pairwise_info::PairwiseInfo;
use crate::protocols::oob::{build_handshake_reuse_accepted_msg, build_handshake_reuse_msg};
use crate::protocols::trustping::build_ping_response;
use crate::protocols::SendClosure;
use crate::utils::send_message;
use crate::utils::serialization::SerializableObjectWithState;

#[derive(Clone, PartialEq)]
pub struct Connection {
    connection_sm: SmConnection,
    cloud_agent_info: CloudAgentInfo,
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
    pub async fn create(
        source_id: &str,
        wallet_handle: WalletHandle,
        agency_client: &AgencyClient,
        autohop_enabled: bool,
    ) -> VcxResult<Self> {
        trace!("Connection::create >>> source_id: {}", source_id);
        let pairwise_info = PairwiseInfo::create(wallet_handle).await?;
        let cloud_agent_info = CloudAgentInfo::create(agency_client, &pairwise_info).await?;
        Ok(Self {
            cloud_agent_info,
            connection_sm: SmConnection::Inviter(SmConnectionInviter::new(source_id, pairwise_info)),
            autohop_enabled,
        })
    }

    pub async fn create_with_invite(
        source_id: &str,
        wallet_handle: WalletHandle,
        agency_client: &AgencyClient,
        invitation: Invitation,
        autohop_enabled: bool,
    ) -> VcxResult<Self> {
        trace!(
            "Connection::create_with_invite >>> source_id: {}, invitation: {:?}",
            source_id,
            invitation
        );
        let pairwise_info = PairwiseInfo::create(wallet_handle).await?;
        let cloud_agent_info = CloudAgentInfo::create(agency_client, &pairwise_info).await?;
        let mut connection = Self {
            cloud_agent_info,
            connection_sm: SmConnection::Invitee(SmConnectionInvitee::new(source_id, pairwise_info)),
            autohop_enabled,
        };
        connection.process_invite(invitation)?;
        Ok(connection)
    }

    pub async fn create_with_request(
        wallet_handle: WalletHandle,
        request: Request,
        public_agent: &PublicAgent,
        agency_client: &AgencyClient,
    ) -> VcxResult<Self> {
        trace!(
            "Connection::create_with_request >>> request: {:?}, public_agent: {:?}",
            request,
            public_agent
        );
        let pairwise_info: PairwiseInfo = public_agent.into();
        let mut connection = Self {
            cloud_agent_info: public_agent.cloud_agent_info(),
            connection_sm: SmConnection::Inviter(SmConnectionInviter::new(&request.id.0, pairwise_info)),
            autohop_enabled: true,
        };
        connection.process_request(wallet_handle, request, agency_client).await
    }

    pub fn from_parts(
        source_id: String,
        thread_id: String,
        pairwise_info: PairwiseInfo,
        cloud_agent_info: CloudAgentInfo,
        state: SmConnectionState,
        autohop_enabled: bool,
    ) -> Self {
        match state {
            SmConnectionState::Inviter(state) => Self {
                cloud_agent_info,
                connection_sm: SmConnection::Inviter(SmConnectionInviter::from(
                    source_id,
                    thread_id,
                    pairwise_info,
                    state,
                )),
                autohop_enabled,
            },
            SmConnectionState::Invitee(state) => Self {
                cloud_agent_info,
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

    pub fn cloud_agent_info(&self) -> CloudAgentInfo {
        self.cloud_agent_info.clone()
    }

    pub fn remote_did(&self, pool_handle: PoolHandle) -> VcxResult<String> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.remote_did(),
            SmConnection::Invitee(sm_invitee) => sm_invitee.remote_did(pool_handle),
        }
    }

    pub fn remote_vk(&self, pool_handle: PoolHandle) -> VcxResult<String> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.remote_vk(),
            SmConnection::Invitee(sm_invitee) => sm_invitee.remote_vk(pool_handle),
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

    pub fn their_did_doc(&self, pool_handle: PoolHandle) -> Option<DidDoc> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.their_did_doc(),
            SmConnection::Invitee(sm_invitee) => sm_invitee.their_did_doc(pool_handle),
        }
    }

    pub fn bootstrap_did_doc(&self, pool_handle: PoolHandle) -> Option<DidDoc> {
        match &self.connection_sm {
            SmConnection::Inviter(_sm_inviter) => None, // TODO: Inviter can remember bootstrap agent too, but we don't need it
            SmConnection::Invitee(sm_invitee) => sm_invitee.bootstrap_did_doc(pool_handle),
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

    async fn process_request(
        &mut self,
        wallet_handle: WalletHandle,
        request: Request,
        agency_client: &AgencyClient,
    ) -> VcxResult<Self> {
        trace!("Connection::process_request >>> request: {:?}", request);
        let (connection_sm, new_cloud_agent_info) = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                let new_pairwise_info = PairwiseInfo::create(wallet_handle).await?;
                let new_cloud_agent = CloudAgentInfo::create(agency_client, &new_pairwise_info).await?;
                let new_routing_keys = new_cloud_agent.routing_keys(agency_client)?;
                let new_service_endpoint = agency_client.get_agency_url_full();
                (
                    SmConnection::Inviter(
                        sm_inviter
                            .clone()
                            .handle_connection_request(
                                wallet_handle,
                                request,
                                &new_pairwise_info,
                                new_routing_keys,
                                new_service_endpoint,
                                send_message,
                            )
                            .await?,
                    ),
                    new_cloud_agent,
                )
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
        agency_client: AgencyClient,
        message: Option<A2AMessage>,
    ) -> BoxFuture<'_, VcxResult<()>> {
        Box::pin(async move {
            let (new_connection_sm, can_autohop) = match &self.connection_sm {
                SmConnection::Inviter(_) => self.step_inviter(wallet_handle, message, &agency_client).await?,
                SmConnection::Invitee(_) => self.step_invitee(wallet_handle, message).await?,
            };
            *self = new_connection_sm;
            if can_autohop && self.autohop_enabled {
                let res = self.update_state_with_message(wallet_handle, agency_client, None).await;
                res
            } else {
                Ok(())
            }
        })
    }

    pub async fn find_and_handle_message(
        &mut self,
        wallet_handle: WalletHandle,
        pool_handle: PoolHandle,
        agency_client: &AgencyClient,
    ) -> VcxResult<()> {
        if !self.is_in_final_state() {
            warn!("Connection::find_and_handle_message >> connection is not in final state, skipping");
            return Ok(());
        }
        let messages = self.get_messages_noauth(agency_client).await?;
        match self.find_message_to_handle(messages) {
            Some((uid, message)) => {
                self.handle_message(message, wallet_handle, pool_handle).await?;
                self.update_message_status(&uid, agency_client).await?;
            }
            None => {}
        };
        Ok(())
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

    pub async fn handle_message(&mut self, message: A2AMessage, wallet_handle: WalletHandle, pool_handle: PoolHandle) -> VcxResult<()> {
        let did_doc = self.their_did_doc(pool_handle).ok_or(VcxError::from_msg(
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

    pub async fn find_message_and_update_state(
        &mut self,
        wallet_handle: WalletHandle,
        agency_client: &AgencyClient,
    ) -> VcxResult<()> {
        if self.is_in_null_state() {
            warn!("Connection::update_state :: update state on connection in null state is ignored");
            return Ok(());
        }
        if self.is_in_final_state() {
            warn!("Connection::update_state :: update state on connection in final state is ignored");
            return Ok(());
        }
        trace!(
            "Connection::update_state >>> before update_state {:?}",
            self.get_state()
        );

        let messages = self.get_messages_noauth(agency_client).await?;
        trace!("Connection::update_state >>> retrieved messages {:?}", messages);

        match self.find_message_to_update_state(messages) {
            Some((uid, message)) => {
                trace!("Connection::update_state >>> handling message uid: {:?}", uid);
                self.update_state_with_message(wallet_handle, agency_client.clone(), Some(message))
                    .await?;
                self.cloud_agent_info()
                    .update_message_status(agency_client, self.pairwise_info(), uid)
                    .await?;
            }
            None => {
                trace!("Connection::update_state >>> trying to update state without message");
                self.update_state_with_message(wallet_handle, agency_client.clone(), None)
                    .await?;
            }
        }

        trace!("Connection::update_state >>> after update_state {:?}", self.get_state());
        Ok(())
    }

    async fn step_inviter(
        &self,
        wallet_handle: WalletHandle,
        message: Option<A2AMessage>,
        agency_client: &AgencyClient,
    ) -> VcxResult<(Self, bool)> {
        match self.connection_sm.clone() {
            SmConnection::Inviter(sm_inviter) => {
                let (sm_inviter, new_cloud_agent_info, can_autohop) = match message {
                    Some(message) => match message {
                        A2AMessage::ConnectionRequest(request) => {
                            let new_pairwise_info = PairwiseInfo::create(wallet_handle).await?;
                            let new_cloud_agent = CloudAgentInfo::create(agency_client, &new_pairwise_info).await?;
                            let new_routing_keys = new_cloud_agent.routing_keys(agency_client)?;
                            let new_service_endpoint = new_cloud_agent.service_endpoint(agency_client)?;
                            let sm_connection = sm_inviter
                                .handle_connection_request(
                                    wallet_handle,
                                    request,
                                    &new_pairwise_info,
                                    new_routing_keys,
                                    new_service_endpoint,
                                    send_message,
                                )
                                .await?;
                            (sm_connection, Some(new_cloud_agent), true)
                        }
                        msg @ A2AMessage::Ack(_) | msg @ A2AMessage::Ping(_) => {
                            (sm_inviter.handle_confirmation_message(&msg).await?, None, false)
                        }
                        A2AMessage::ConnectionProblemReport(problem_report) => {
                            (sm_inviter.handle_problem_report(problem_report)?, None, false)
                        }
                        _ => (sm_inviter.clone(), None, false),
                    },
                    None => {
                        if let InviterFullState::Requested(_) = sm_inviter.state_object() {
                            (
                                sm_inviter.handle_send_response(wallet_handle, &send_message).await?,
                                None,
                                false,
                            )
                        } else {
                            (sm_inviter.clone(), None, false)
                        }
                    }
                };

                let connection = Self {
                    cloud_agent_info: new_cloud_agent_info.unwrap_or(self.cloud_agent_info.clone()),
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
                            (sm_invitee.handle_connection_response(response)?, true)
                        }
                        A2AMessage::ConnectionProblemReport(problem_report) => {
                            (sm_invitee.handle_problem_report(problem_report)?, false)
                        }
                        _ => (sm_invitee, false),
                    },
                    None => (sm_invitee.handle_send_ack(wallet_handle, &send_message).await?, false),
                };
                let connection = Self {
                    connection_sm: SmConnection::Invitee(sm_invitee),
                    cloud_agent_info: self.cloud_agent_info.clone(),
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

    pub async fn connect(&mut self, wallet_handle: WalletHandle, pool_handle: PoolHandle, agency_client: &AgencyClient) -> VcxResult<()> {
        trace!("Connection::connect >>> source_id: {}", self.source_id());
        self.connection_sm = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => SmConnection::Inviter(sm_inviter.clone().create_invitation(
                self.cloud_agent_info.routing_keys(agency_client)?,
                self.cloud_agent_info.service_endpoint(agency_client)?,
            )?),
            SmConnection::Invitee(sm_invitee) => SmConnection::Invitee(
                sm_invitee
                    .clone()
                    .send_connection_request(
                        wallet_handle,
                        pool_handle,
                        self.cloud_agent_info.routing_keys(agency_client)?,
                        self.cloud_agent_info.service_endpoint(agency_client)?,
                        send_message,
                    )
                    .await?,
            ),
        };
        Ok(())
    }

    pub async fn update_message_status(&self, uid: &str, agency_client: &AgencyClient) -> VcxResult<()> {
        trace!("Connection::update_message_status >>> uid: {:?}", uid);
        self.cloud_agent_info()
            .update_message_status(agency_client, self.pairwise_info(), uid.to_string())
            .await
    }

    pub async fn get_messages_noauth(&self, agency_client: &AgencyClient) -> VcxResult<HashMap<String, A2AMessage>> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                let messages = self
                    .cloud_agent_info()
                    .get_messages_noauth(agency_client, sm_inviter.pairwise_info(), None)
                    .await?;
                Ok(messages)
            }
            SmConnection::Invitee(sm_invitee) => {
                let messages = self
                    .cloud_agent_info()
                    .get_messages_noauth(agency_client, sm_invitee.pairwise_info(), None)
                    .await?;
                Ok(messages)
            }
        }
    }

    pub async fn get_messages(&self, pool_handle: PoolHandle, agency_client: &AgencyClient) -> VcxResult<HashMap<String, A2AMessage>> {
        let expected_sender_vk = self.get_expected_sender_vk(pool_handle)?;
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => Ok(self
                .cloud_agent_info()
                .get_messages(agency_client, &expected_sender_vk, sm_inviter.pairwise_info())
                .await?),
            SmConnection::Invitee(sm_invitee) => Ok(self
                .cloud_agent_info()
                .get_messages(agency_client, &expected_sender_vk, sm_invitee.pairwise_info())
                .await?),
        }
    }

    fn get_expected_sender_vk(&self, pool_handle: PoolHandle) -> VcxResult<String> {
        self.remote_vk(pool_handle).map_err(|_err| {
            VcxError::from_msg(
                VcxErrorKind::NotReady,
                "Verkey of connection counterparty \
                is not known, hence it would be impossible to authenticate message downloaded by id.",
            )
        })
    }

    pub async fn get_message_by_id(&self, pool_handle: PoolHandle, msg_id: &str, agency_client: &AgencyClient) -> VcxResult<A2AMessage> {
        trace!("Connection: get_message_by_id >>> msg_id: {}", msg_id);
        let expected_sender_vk = self.get_expected_sender_vk(pool_handle)?;
        self.cloud_agent_info()
            .get_message_by_id(agency_client, msg_id, &expected_sender_vk, self.pairwise_info())
            .await
    }

    pub fn send_message_closure(&self, wallet_handle: WalletHandle, pool_handle: PoolHandle) -> VcxResult<SendClosure> {
        trace!("send_message_closure >>>");
        let did_doc = self.their_did_doc(pool_handle).ok_or(VcxError::from_msg(
            VcxErrorKind::NotReady,
            "Cannot send message: Remote Connection information is not set",
        ))?;
        let sender_vk = self.pairwise_info().pw_vk.clone();
        Ok(Box::new(move |message: A2AMessage| {
            Box::pin(send_message(wallet_handle, sender_vk.clone(), did_doc.clone(), message))
        }))
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

    pub async fn send_generic_message(&self, wallet_handle: WalletHandle, pool_handle: PoolHandle, message: &str) -> VcxResult<String> {
        trace!("Connection::send_generic_message >>> message: {:?}", message);
        let message = Self::build_basic_message(message);
        let send_message = self.send_message_closure(wallet_handle, pool_handle)?;
        send_message(message).await.map(|_| String::new())
    }

    pub async fn send_a2a_message(&self, wallet_handle: WalletHandle, pool_handle: PoolHandle, message: &A2AMessage) -> VcxResult<String> {
        trace!("Connection::send_a2a_message >>> message: {:?}", message);
        let send_message = self.send_message_closure(wallet_handle, pool_handle)?;
        send_message(message.clone()).await.map(|_| String::new())
    }

    pub async fn send_ping(
        &mut self,
        wallet_handle: WalletHandle,
        pool_handle: PoolHandle,
        comment: Option<String>,
    ) -> VcxResult<TrustPingSender> {
        let mut trust_ping = TrustPingSender::build(true, comment);
        trust_ping.send_ping(self.send_message_closure(wallet_handle, pool_handle)?).await?;
        Ok(trust_ping)
    }

    pub async fn send_handshake_reuse(&self, wallet_handle: WalletHandle, pool_handle: PoolHandle, oob_msg: &str) -> VcxResult<()> {
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
        let did_doc = self.their_did_doc(pool_handle).ok_or(VcxError::from_msg(
            VcxErrorKind::NotReady,
            format!("Can't send handshake-reuse to the counterparty, because their did doc is not available"),
        ))?;
        send_message(
            wallet_handle,
            self.pairwise_info().pw_vk.clone(),
            did_doc.clone(),
            build_handshake_reuse_msg(&oob).to_a2a_message(),
        )
        .await
    }

    pub async fn delete(&self, agency_client: &AgencyClient) -> VcxResult<()> {
        trace!("Connection: delete >>> {:?}", self.source_id());
        self.cloud_agent_info()
            .destroy(agency_client, self.pairwise_info())
            .await
    }

    pub async fn send_discovery_query(
        &self,
        wallet_handle: WalletHandle,
        pool_handle: PoolHandle,
        query: Option<String>,
        comment: Option<String>,
    ) -> VcxResult<()> {
        trace!(
            "Connection::send_discovery_features_query >>> query: {:?}, comment: {:?}",
            query,
            comment
        );
        let did_doc = self.their_did_doc(pool_handle).ok_or(VcxError::from_msg(
            VcxErrorKind::NotReady,
            format!("Can't send handshake-reuse to the counterparty, because their did doc is not available"),
        ))?;
        send_discovery_query(wallet_handle, query, comment, &did_doc, &self.pairwise_info().pw_vk).await?;
        Ok(())
    }

    pub fn get_connection_info(&self, pool_handle: PoolHandle, agency_client: &AgencyClient) -> VcxResult<String> {
        trace!("Connection::get_connection_info >>>");

        let agent_info = self.cloud_agent_info();
        let pairwise_info = self.pairwise_info();
        let recipient_keys = vec![pairwise_info.pw_vk.clone()];

        let current = SideConnectionInfo {
            did: pairwise_info.pw_did.clone(),
            recipient_keys,
            routing_keys: agent_info.routing_keys(agency_client)?,
            service_endpoint: agent_info.service_endpoint(agency_client)?,
            protocols: Some(self.get_protocols()),
        };

        let remote = match self.their_did_doc(pool_handle) {
            Some(did_doc) => Some(SideConnectionInfo {
                did: did_doc.id.clone(),
                recipient_keys: did_doc.recipient_keys(),
                routing_keys: did_doc.routing_keys(),
                service_endpoint: did_doc.get_endpoint(),
                protocols: self.get_remote_protocols(),
            }),
            None => None,
        };

        let connection_info = ConnectionInfo {
            my: current,
            their: remote,
        };

        let connection_info_json = serde_json::to_string(&connection_info).map_err(|err| {
            VcxError::from_msg(
                VcxErrorKind::InvalidState,
                format!("Cannot serialize ConnectionInfo: {:?}", err),
            )
        })?;

        Ok(connection_info_json)
    }

    pub async fn download_messages(
        &self,
        pool_handle: PoolHandle,
        agency_client: &AgencyClient,
        status_codes: Option<Vec<MessageStatusCode>>,
        uids: Option<Vec<String>>,
    ) -> VcxResult<Vec<DownloadedMessage>> {
        match self.get_state() {
            ConnectionState::Invitee(InviteeState::Initial)
            | ConnectionState::Inviter(InviterState::Initial)
            | ConnectionState::Inviter(InviterState::Invited) => {
                let msgs = futures::stream::iter(
                    self.cloud_agent_info()
                        .download_encrypted_messages(agency_client, uids, status_codes, self.pairwise_info())
                        .await?,
                )
                .then(|msg| msg.decrypt_noauth(agency_client.get_wallet_handle()))
                .filter_map(|res| async { res.ok() })
                .collect::<Vec<DownloadedMessage>>()
                .await;
                Ok(msgs)
            }
            _ => {
                let expected_sender_vk = self.remote_vk(pool_handle)?;
                let msgs = futures::stream::iter(
                    self.cloud_agent_info()
                        .download_encrypted_messages(agency_client, uids, status_codes, self.pairwise_info())
                        .await?,
                )
                .then(|msg| msg.decrypt_auth(agency_client.get_wallet_handle(), &expected_sender_vk))
                .filter_map(|res| async { res.ok() })
                .collect::<Vec<DownloadedMessage>>()
                .await;
                Ok(msgs)
            }
        }
    }

    pub fn to_string(&self) -> VcxResult<String> {
        serde_json::to_string(&self).map_err(|err| {
            VcxError::from_msg(
                VcxErrorKind::SerializationError,
                format!("Cannot serialize Connection: {:?}", err),
            )
        })
    }

    pub fn from_string(connection_data: &str) -> VcxResult<Self> {
        serde_json::from_str(connection_data).map_err(|err| {
            VcxError::from_msg(
                VcxErrorKind::InvalidJson,
                format!("Cannot deserialize Connection: {:?}", err),
            )
        })
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
        let object = SerializableObjectWithState::V1 {
            data,
            state,
            source_id,
            thread_id,
        };
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
        A: MapAccess<'de>,
    {
        let mut map_value = serde_json::Map::new();
        while let Some(key) = map.next_key()? {
            let k: String = key;
            let v: Value = map.next_value()?;
            map_value.insert(k, v);
        }
        let obj = Value::from(map_value);
        let ver: SerializableObjectWithState<LegacyAgentInfo, SmConnectionState> =
            serde_json::from_value(obj).map_err(|err| A::Error::custom(err.to_string()))?;
        match ver {
            SerializableObjectWithState::V1 {
                data,
                state,
                source_id,
                thread_id,
            } => {
                let pairwise_info = PairwiseInfo {
                    pw_did: data.pw_did,
                    pw_vk: data.pw_vk,
                };
                let cloud_agent_info = CloudAgentInfo {
                    agent_did: data.agent_did,
                    agent_vk: data.agent_vk,
                };
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
        (
            self.state_object(),
            self.pairwise_info().to_owned(),
            self.cloud_agent_info(),
            self.source_id(),
            self.get_thread_id(),
        )
    }
}

impl From<(SmConnectionState, PairwiseInfo, CloudAgentInfo, String, String)> for Connection {
    fn from(
        (state, pairwise_info, cloud_agent_info, source_id, thread_id): (
            SmConnectionState,
            PairwiseInfo,
            CloudAgentInfo,
            String,
            String,
        ),
    ) -> Connection {
        Connection::from_parts(source_id, thread_id, pairwise_info, cloud_agent_info, state, true)
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod tests {
    use indy_sys::WalletHandle;

    use agency_client::testing::mocking::enable_agency_mocks;

    use crate::handlers::connection::public_agent::test_utils::_public_agent;
    use crate::messages::connection::invite::test_utils::{
        _pairwise_invitation, _pairwise_invitation_random_id, _public_invitation, _public_invitation_random_id,
    };
    use crate::messages::connection::request::unit_tests::_request;
    use crate::messages::connection::response::test_utils::_signed_response;
    use crate::messages::discovery::disclose::test_utils::_disclose;
    use crate::messages::discovery::query::test_utils::_query;
    use crate::utils::devsetup::{SetupIndyMocks, SetupMocks};
    use crate::utils::mockdata::mockdata_connection::{
        CONNECTION_SM_INVITEE_COMPLETED, CONNECTION_SM_INVITEE_INVITED, CONNECTION_SM_INVITEE_REQUESTED,
        CONNECTION_SM_INVITER_COMPLETED,
    };

    use super::*;

    fn _dummy_pool_handle() -> PoolHandle {
        0
    }

    #[tokio::test]
    async fn test_create_with_pairwise_invite() {
        let _setup = SetupMocks::init();
        let agency_client = AgencyClient::new();
        enable_agency_mocks();
        let connection = Connection::create_with_invite(
            "abc",
            WalletHandle(0),
            &agency_client,
            Invitation::Pairwise(_pairwise_invitation()),
            true,
        )
        .await
        .unwrap();
        assert_eq!(connection.get_state(), ConnectionState::Invitee(InviteeState::Invited));
    }

    #[tokio::test]
    async fn test_create_with_public_invite() {
        let _setup = SetupMocks::init();
        let agency_client = AgencyClient::new();
        enable_agency_mocks();
        let connection = Connection::create_with_invite(
            "abc",
            WalletHandle(0),
            &agency_client,
            Invitation::Public(_public_invitation()),
            true,
        )
        .await
        .unwrap();
        assert_eq!(connection.get_state(), ConnectionState::Invitee(InviteeState::Invited));
    }

    #[tokio::test]
    async fn test_connect_sets_correct_thread_id_based_on_invitation_type() {
        let _setup = SetupMocks::init();
        let agency_client = AgencyClient::new();
        enable_agency_mocks();

        let pub_inv = _public_invitation_random_id();
        let mut connection = Connection::create_with_invite(
            "abcd",
            WalletHandle(0),
            &agency_client,
            Invitation::Public(pub_inv.clone()),
            true,
        )
        .await
        .unwrap();
        connection.connect(WalletHandle(0), _dummy_pool_handle(), &agency_client).await.unwrap();
        assert_eq!(
            connection.get_state(),
            ConnectionState::Invitee(InviteeState::Requested)
        );
        assert_ne!(connection.get_thread_id(), pub_inv.id.0);

        let pw_inv = _pairwise_invitation_random_id();
        let mut connection = Connection::create_with_invite(
            "dcba",
            WalletHandle(0),
            &agency_client,
            Invitation::Pairwise(pw_inv.clone()),
            true,
        )
        .await
        .unwrap();
        connection.connect(WalletHandle(0), _dummy_pool_handle(), &agency_client).await.unwrap();
        assert_eq!(
            connection.get_state(),
            ConnectionState::Invitee(InviteeState::Requested)
        );
        assert_eq!(connection.get_thread_id(), pw_inv.id.0);
    }

    #[tokio::test]
    async fn test_create_with_request() {
        let _setup = SetupMocks::init();
        let agency_client = AgencyClient::new();
        enable_agency_mocks();
        let connection = Connection::create_with_request(WalletHandle(0), _request(), &_public_agent(), &agency_client)
            .await
            .unwrap();
        assert_eq!(
            connection.get_state(),
            ConnectionState::Inviter(InviterState::Requested)
        );
    }

    #[tokio::test]
    // todo
    async fn test_should_find_messages_to_answer() {
        let _setup = SetupMocks::init();
        let agency_client = AgencyClient::new();
        enable_agency_mocks();
        let connection = Connection::create_with_request(WalletHandle(0), _request(), &_public_agent(), &agency_client)
            .await
            .unwrap();
        assert_eq!(
            connection.get_state(),
            ConnectionState::Inviter(InviterState::Requested)
        );
    }

    #[tokio::test]
    async fn test_deserialize_connection_inviter_completed() {
        let _setup = SetupMocks::init();

        let connection = Connection::from_string(CONNECTION_SM_INVITER_COMPLETED).unwrap();
        let _second_string = connection.to_string();

        assert_eq!(connection.pairwise_info().pw_did, "2ZHFFhzA2XtTD6hJqzL7ux");
        assert_eq!(
            connection.pairwise_info().pw_vk,
            "rCw3x5h1jS6gPo7rRrt3EYbXXe5nNjnGbdf1jAwUxuj"
        );
        assert_eq!(connection.cloud_agent_info().agent_did, "EZrZyu4bfydm4ByNm56kPP");
        assert_eq!(
            connection.cloud_agent_info().agent_vk,
            "8Ps2WosJ9AV1eXPoJKsEJdM3NchPhSyS8qFt6LQUTKv2"
        );
        assert_eq!(
            connection.get_state(),
            ConnectionState::Inviter(InviterState::Completed)
        );
    }

    fn test_deserialize_and_serialize(sm_serialized: &str) {
        let original_object: Value = serde_json::from_str(sm_serialized).unwrap();
        let connection = Connection::from_string(sm_serialized).unwrap();
        let reserialized = connection.to_string().unwrap();
        let reserialized_object: Value = serde_json::from_str(&reserialized).unwrap();

        assert_eq!(original_object, reserialized_object);
    }

    #[tokio::test]
    async fn test_deserialize_and_serialize_should_produce_the_same_object() {
        let _setup = SetupMocks::init();

        test_deserialize_and_serialize(CONNECTION_SM_INVITEE_INVITED);
        test_deserialize_and_serialize(CONNECTION_SM_INVITEE_REQUESTED);
        test_deserialize_and_serialize(CONNECTION_SM_INVITEE_COMPLETED);
        test_deserialize_and_serialize(CONNECTION_SM_INVITER_COMPLETED);
    }

    fn _dummy_agency_client() -> AgencyClient {
        AgencyClient::new()
    }

    #[tokio::test]
    async fn test_serialize_deserialize() {
        let _setup = SetupMocks::init();

        let connection = Connection::create(
            "test_serialize_deserialize",
            WalletHandle(0),
            &_dummy_agency_client(),
            true,
        )
        .await
        .unwrap();
        let first_string = connection.to_string().unwrap();

        let connection2 = Connection::from_string(&first_string).unwrap();
        let second_string = connection2.to_string().unwrap();

        assert_eq!(first_string, second_string);
    }

    #[tokio::test]
    async fn test_serialize_deserialize_serde() {
        let _setup = SetupMocks::init();

        let connection = Connection::create(
            "test_serialize_deserialize",
            WalletHandle(0),
            &_dummy_agency_client(),
            true,
        )
        .await
        .unwrap();
        let first_string = serde_json::to_string(&connection).unwrap();

        let connection: Connection = serde_json::from_str(&first_string).unwrap();
        let second_string = serde_json::to_string(&connection).unwrap();
        assert_eq!(first_string, second_string);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_find_message_to_handle_from_completed_state() {
        let _setup = SetupIndyMocks::init();

        let connection = Connection::from_string(CONNECTION_SM_INVITER_COMPLETED).unwrap();
        // Query
        {
            let messages = map!(
                "key_1".to_string() => A2AMessage::ConnectionRequest(_request()),
                "key_2".to_string() => A2AMessage::ConnectionResponse(_signed_response()),
                "key_3".to_string() => A2AMessage::Query(_query())
            );

            let (uid, message) = connection.find_message_to_handle(messages).unwrap();
            assert_eq!("key_3", uid);
            assert_match!(A2AMessage::Query(_), message);
        }
        // Disclose
        {
            let messages = map!(
                "key_1".to_string() => A2AMessage::ConnectionRequest(_request()),
                "key_2".to_string() => A2AMessage::ConnectionResponse(_signed_response()),
                "key_3".to_string() => A2AMessage::Disclose(_disclose())
            );

            let (uid, message) = connection.find_message_to_handle(messages).unwrap();
            assert_eq!("key_3", uid);
            assert_match!(A2AMessage::Disclose(_), message);
        }
    }
}
