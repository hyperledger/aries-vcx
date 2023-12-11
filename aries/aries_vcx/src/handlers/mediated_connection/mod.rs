use std::{collections::HashMap, fmt};

use agency_client::{
    agency_client::AgencyClient, api::downloaded_message::DownloadedMessage, MessageStatusCode,
};
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use chrono::Utc;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use futures::{future::BoxFuture, StreamExt};
use messages::{
    decorators::timing::Timing,
    msg_fields::protocols::{
        basic_message::{BasicMessage, BasicMessageContent, BasicMessageDecorators},
        connection::{invitation::InvitationContent, request::Request, Connection},
        out_of_band::OutOfBand,
        trust_ping::TrustPing,
    },
    AriesMessage,
};
use serde::{
    de::{Error, MapAccess, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_json::Value;
use url::Url;
use uuid::Uuid;

use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    handlers::{
        mediated_connection::{
            cloud_agent::CloudAgentInfo, legacy_agent_info::LegacyAgentInfo, util::send_message,
        },
        trust_ping::TrustPingSender,
        util::AnyInvitation,
    },
    protocols::{
        connection::pairwise_info::PairwiseInfo,
        mediated_connection::{
            invitee::state_machine::{
                MediatedInviteeFullState, MediatedInviteeState, SmMediatedConnectionInvitee,
            },
            inviter::state_machine::{
                MediatedInviterFullState, MediatedInviterState, SmMediatedConnectionInviter,
            },
        },
        oob::{build_handshake_reuse_accepted_msg, build_handshake_reuse_msg},
        trustping::build_ping_response,
        SendClosure, SendClosureConnection,
    },
    utils::serialization::SerializableObjectWithState,
};

pub mod cloud_agent;
pub mod legacy_agent_info;
pub(crate) mod util;

#[derive(Clone, PartialEq)]
pub struct MediatedConnection {
    connection_sm: SmMediatedConnection,
    cloud_agent_info: Option<CloudAgentInfo>,
    autohop_enabled: bool,
}

#[derive(Clone, PartialEq)]
pub enum SmMediatedConnection {
    Inviter(SmMediatedConnectionInviter),
    Invitee(SmMediatedConnectionInvitee),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmConnectionState {
    Inviter(MediatedInviterFullState),
    Invitee(MediatedInviteeFullState),
}

#[derive(Debug, Serialize)]
struct ConnectionInfo {
    my: SideDetails,
    their: Option<SideDetails>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MediatedConnectionState {
    Inviter(MediatedInviterState),
    Invitee(MediatedInviteeState),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SideDetails {
    did: String,
    recipient_keys: Vec<String>,
    routing_keys: Vec<String>,
    service_endpoint: Url,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MediatedConnectionActor {
    Inviter,
    Invitee,
}

impl MediatedConnection {
    pub async fn create(
        source_id: &str,
        wallet: &impl BaseWallet,
        agency_client: &AgencyClient,
        autohop_enabled: bool,
    ) -> VcxResult<Self> {
        trace!("MediatedConnection::create >>> source_id: {}", source_id);
        let pairwise_info = PairwiseInfo::create(wallet).await?;
        let cloud_agent_info = Some(CloudAgentInfo::create(agency_client, &pairwise_info).await?);
        Ok(Self {
            cloud_agent_info,
            connection_sm: SmMediatedConnection::Inviter(SmMediatedConnectionInviter::new(
                source_id,
                pairwise_info,
            )),
            autohop_enabled,
        })
    }

    pub async fn create_with_invite(
        source_id: &str,
        wallet: &impl BaseWallet,
        agency_client: &AgencyClient,
        invitation: AnyInvitation,
        did_doc: AriesDidDoc,
        autohop_enabled: bool,
    ) -> VcxResult<Self> {
        trace!(
            "MediatedConnection::create_with_invite >>> source_id: {}, invitation: {:?}",
            source_id,
            invitation
        );
        let pairwise_info = PairwiseInfo::create(wallet).await?;
        let cloud_agent_info = Some(CloudAgentInfo::create(agency_client, &pairwise_info).await?);
        let mut connection = Self {
            cloud_agent_info,
            connection_sm: SmMediatedConnection::Invitee(SmMediatedConnectionInvitee::new(
                source_id,
                pairwise_info,
                did_doc,
            )),
            autohop_enabled,
        };
        connection.process_invite(invitation)?;
        Ok(connection)
    }

    pub async fn create_with_request(
        wallet: &impl BaseWallet,
        request: Request,
        pairwise_info: PairwiseInfo,
        agency_client: &AgencyClient,
    ) -> VcxResult<Self> {
        trace!(
            "MediatedConnection::create_with_request >>> request: {:?}, pairwise_info: {:?}",
            request,
            pairwise_info
        );
        let mut connection = Self {
            cloud_agent_info: None,
            connection_sm: SmMediatedConnection::Inviter(SmMediatedConnectionInviter::new(
                &request.id,
                pairwise_info,
            )),
            autohop_enabled: true,
        };
        connection
            .process_request(wallet, agency_client, request)
            .await?;
        Ok(connection)
    }

    pub fn from_parts(
        source_id: String,
        thread_id: String,
        pairwise_info: PairwiseInfo,
        cloud_agent_info: Option<CloudAgentInfo>,
        state: SmConnectionState,
        autohop_enabled: bool,
    ) -> Self {
        match state {
            SmConnectionState::Inviter(state) => Self {
                cloud_agent_info,
                connection_sm: SmMediatedConnection::Inviter(SmMediatedConnectionInviter::from(
                    source_id,
                    thread_id,
                    pairwise_info,
                    state,
                )),
                autohop_enabled,
            },
            SmConnectionState::Invitee(state) => Self {
                cloud_agent_info,
                connection_sm: SmMediatedConnection::Invitee(SmMediatedConnectionInvitee::from(
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
            SmMediatedConnection::Inviter(sm_inviter) => sm_inviter.source_id(),
            SmMediatedConnection::Invitee(sm_invitee) => sm_invitee.source_id(),
        }
        .into()
    }

    pub fn get_thread_id(&self) -> String {
        match &self.connection_sm {
            SmMediatedConnection::Inviter(sm_inviter) => sm_inviter.get_thread_id(),
            SmMediatedConnection::Invitee(sm_invitee) => sm_invitee.get_thread_id(),
        }
    }

    pub fn get_state(&self) -> MediatedConnectionState {
        match &self.connection_sm {
            SmMediatedConnection::Inviter(sm_inviter) => {
                MediatedConnectionState::Inviter(sm_inviter.get_state())
            }
            SmMediatedConnection::Invitee(sm_invitee) => {
                MediatedConnectionState::Invitee(sm_invitee.get_state())
            }
        }
    }

    pub fn pairwise_info(&self) -> &PairwiseInfo {
        match &self.connection_sm {
            SmMediatedConnection::Inviter(sm_inviter) => sm_inviter.pairwise_info(),
            SmMediatedConnection::Invitee(sm_invitee) => sm_invitee.pairwise_info(),
        }
    }

    pub fn cloud_agent_info(&self) -> Option<CloudAgentInfo> {
        self.cloud_agent_info.clone()
    }

    pub fn remote_did(&self) -> VcxResult<String> {
        match &self.connection_sm {
            SmMediatedConnection::Inviter(sm_inviter) => sm_inviter.remote_did(),
            SmMediatedConnection::Invitee(sm_invitee) => sm_invitee.remote_did(),
        }
    }

    pub fn remote_vk(&self) -> VcxResult<String> {
        match &self.connection_sm {
            SmMediatedConnection::Inviter(sm_inviter) => sm_inviter.remote_vk(),
            SmMediatedConnection::Invitee(sm_invitee) => sm_invitee.remote_vk(),
        }
    }

    pub fn state_object(&self) -> SmConnectionState {
        match &self.connection_sm {
            SmMediatedConnection::Inviter(sm_inviter) => {
                SmConnectionState::Inviter(sm_inviter.state_object().clone())
            }
            SmMediatedConnection::Invitee(sm_invitee) => {
                SmConnectionState::Invitee(sm_invitee.state_object().clone())
            }
        }
    }

    pub fn get_source_id(&self) -> String {
        match &self.connection_sm {
            SmMediatedConnection::Inviter(sm_inviter) => sm_inviter.source_id(),
            SmMediatedConnection::Invitee(sm_invitee) => sm_invitee.source_id(),
        }
        .to_string()
    }

    pub fn their_did_doc(&self) -> Option<AriesDidDoc> {
        match &self.connection_sm {
            SmMediatedConnection::Inviter(sm_inviter) => sm_inviter.their_did_doc(),
            SmMediatedConnection::Invitee(sm_invitee) => sm_invitee.their_did_doc(),
        }
    }

    pub fn bootstrap_did_doc(&self) -> Option<AriesDidDoc> {
        match &self.connection_sm {
            SmMediatedConnection::Inviter(_sm_inviter) => None, /* TODO: Inviter can remember
                                                                  * bootstrap */
            // agent too, but we don't need it
            SmMediatedConnection::Invitee(sm_invitee) => sm_invitee.bootstrap_did_doc(),
        }
    }

    pub fn is_in_null_state(&self) -> bool {
        match &self.connection_sm {
            SmMediatedConnection::Inviter(sm_inviter) => sm_inviter.is_in_null_state(),
            SmMediatedConnection::Invitee(sm_invitee) => sm_invitee.is_in_null_state(),
        }
    }

    pub fn is_in_final_state(&self) -> bool {
        match &self.connection_sm {
            SmMediatedConnection::Inviter(sm_inviter) => sm_inviter.is_in_final_state(),
            SmMediatedConnection::Invitee(sm_invitee) => sm_invitee.is_in_final_state(),
        }
    }

    pub fn process_invite(&mut self, invitation: AnyInvitation) -> VcxResult<()> {
        trace!(
            "MediatedConnection::process_invite >>> invitation: {:?}",
            invitation
        );
        self.connection_sm = match &self.connection_sm {
            SmMediatedConnection::Inviter(_sm_inviter) => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::NotReady,
                    "Invalid action",
                ));
            }
            SmMediatedConnection::Invitee(sm_invitee) => {
                SmMediatedConnection::Invitee(sm_invitee.clone().handle_invitation(invitation)?)
            }
        };
        Ok(())
    }

    pub async fn process_request(
        &mut self,
        wallet: &impl BaseWallet,
        agency_client: &AgencyClient,
        request: Request,
    ) -> VcxResult<()> {
        trace!(
            "MediatedConnection::process_request >>> request: {:?}",
            request
        );
        let (connection_sm, new_cloud_agent_info) = match &self.connection_sm {
            SmMediatedConnection::Inviter(sm_inviter) => {
                let send_message = self.send_message_closure_connection(wallet);
                let new_pairwise_info = PairwiseInfo::create(wallet).await?;
                let new_cloud_agent =
                    CloudAgentInfo::create(agency_client, &new_pairwise_info).await?;
                let new_routing_keys = new_cloud_agent.routing_keys(agency_client)?;
                let new_service_endpoint = agency_client.get_agency_url_full()?;
                (
                    SmMediatedConnection::Inviter(
                        sm_inviter
                            .clone()
                            .handle_connection_request(
                                wallet,
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
            SmMediatedConnection::Invitee(_) => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::NotReady,
                    "Invalid action",
                ));
            }
        };
        self.connection_sm = connection_sm;
        self.cloud_agent_info = Some(new_cloud_agent_info);
        Ok(())
    }

    pub async fn send_response(&mut self, wallet: &impl BaseWallet) -> VcxResult<()> {
        trace!("MediatedConnection::send_response >>>");
        let connection_sm = match self.connection_sm.clone() {
            SmMediatedConnection::Inviter(sm_inviter) => {
                if let MediatedInviterFullState::Requested(_) = sm_inviter.state_object() {
                    let send_message = self.send_message_closure_connection(wallet);
                    sm_inviter.handle_send_response(send_message).await?
                } else {
                    return Err(AriesVcxError::from_msg(
                        AriesVcxErrorKind::NotReady,
                        "Invalid action",
                    ));
                }
            }
            SmMediatedConnection::Invitee(_) => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::NotReady,
                    "Invalid action",
                ));
            }
        };
        self.connection_sm = SmMediatedConnection::Inviter(connection_sm);
        Ok(())
    }

    pub fn get_invite_details(&self) -> Option<&AnyInvitation> {
        trace!("MediatedConnection::get_invite_details >>>");
        match &self.connection_sm {
            SmMediatedConnection::Inviter(sm_inviter) => sm_inviter.get_invitation(),
            SmMediatedConnection::Invitee(_sm_invitee) => None,
        }
    }

    fn find_message_to_update_state(
        &self,
        messages: HashMap<String, AriesMessage>,
    ) -> Option<(String, AriesMessage)> {
        match &self.connection_sm {
            SmMediatedConnection::Inviter(sm_inviter) => {
                sm_inviter.find_message_to_update_state(messages)
            }
            SmMediatedConnection::Invitee(sm_invitee) => {
                sm_invitee.find_message_to_update_state(messages)
            }
        }
    }

    // TODO:::: check usage of this method in regards to profile usage
    // TODO:::: check usage of this method in regards to profile usage
    // TODO:::: check usage of this method in regards to profile usage
    pub fn update_state_with_message<'a>(
        &'a mut self,
        wallet: &'a impl BaseWallet,
        agency_client: AgencyClient,
        message: Option<AriesMessage>,
    ) -> BoxFuture<'a, VcxResult<()>> {
        Box::pin(async move {
            let (new_connection_sm, can_autohop) = match &self.connection_sm {
                SmMediatedConnection::Inviter(_) => {
                    self.step_inviter(wallet, message, &agency_client).await?
                }
                SmMediatedConnection::Invitee(_) => self.step_invitee(wallet, message).await?,
            };
            *self = new_connection_sm;
            if can_autohop && self.autohop_enabled {
                self.update_state_with_message(wallet, agency_client, None)
                    .await
            } else {
                Ok(())
            }
        })
    }

    pub async fn find_and_handle_message(
        &mut self,
        wallet: &impl BaseWallet,
        agency_client: &AgencyClient,
    ) -> VcxResult<()> {
        if !self.is_in_final_state() {
            warn!(
                "MediatedConnection::find_and_handle_message >> connection is not in final state, \
                 skipping"
            );
            return Ok(());
        }
        let messages = self.get_messages_noauth(agency_client).await?;
        if let Some((uid, message)) = self.find_message_to_handle(messages) {
            self.handle_message(message, wallet).await?;
            self.update_message_status(&uid, agency_client).await?;
        };
        Ok(())
    }

    fn find_message_to_handle(
        &self,
        messages: HashMap<String, AriesMessage>,
    ) -> Option<(String, AriesMessage)> {
        for (uid, message) in messages {
            match message {
                AriesMessage::TrustPing(TrustPing::Ping(_))
                | AriesMessage::TrustPing(TrustPing::PingResponse(_))
                | AriesMessage::OutOfBand(OutOfBand::HandshakeReuse(_))
                | AriesMessage::OutOfBand(OutOfBand::HandshakeReuseAccepted(_)) => {
                    return Some((uid, message))
                }
                _ => {}
            }
        }
        None
    }

    pub async fn handle_message(
        &mut self,
        message: AriesMessage,
        wallet: &impl BaseWallet,
    ) -> VcxResult<()> {
        let did_doc = self.their_did_doc().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::NotReady,
            format!(
                "Can't answer message {message:?} because counterparty did doc is not available"
            ),
        ))?;
        let pw_vk = &self.pairwise_info().pw_vk;
        match message {
            AriesMessage::TrustPing(TrustPing::Ping(ping)) => {
                let thread_id = ping
                    .decorators
                    .thread
                    .as_ref()
                    .map(|t| t.thid.as_str())
                    .unwrap_or(ping.id.as_str());

                info!("Answering TrustPing::Ping, thread: {}", thread_id);

                if ping.content.response_requested {
                    send_message(
                        wallet,
                        pw_vk.to_string(),
                        did_doc.clone(),
                        build_ping_response(&ping).into(),
                    )
                    .await?;
                }
            }
            AriesMessage::OutOfBand(OutOfBand::HandshakeReuse(handshake_reuse)) => {
                let thread_id = handshake_reuse.decorators.thread.thid.as_str();

                info!(
                    "Answering OutOfBand::HandshakeReuse message, thread: {}",
                    thread_id
                );

                let msg = build_handshake_reuse_accepted_msg(&handshake_reuse)?;
                send_message(wallet, pw_vk.to_string(), did_doc.clone(), msg.into()).await?;
            }
            _ => {
                // todo: implement to_string for A2AMessage, printing only type of the message, not
                // entire payload todo: attempt to print @id / thread_id of the
                // message
                info!("Message of type {:?} will not be answered", message);
            }
        }
        Ok(())
    }

    pub async fn find_message_and_update_state(
        &mut self,
        wallet: &impl BaseWallet,
        agency_client: &AgencyClient,
    ) -> VcxResult<()> {
        if self.is_in_null_state() {
            warn!(
                "MediatedConnection::update_state :: update state on connection in null state is \
                 ignored"
            );
            return Ok(());
        }
        if self.is_in_final_state() {
            warn!(
                "MediatedConnection::update_state :: update state on connection in final state is \
                 ignored"
            );
            return Ok(());
        }
        trace!(
            "MediatedConnection::update_state >>> before update_state {:?}",
            self.get_state()
        );

        let messages = self.get_messages_noauth(agency_client).await?;
        trace!(
            "MediatedConnection::update_state >>> retrieved messages {:?}",
            messages
        );

        match self.find_message_to_update_state(messages) {
            Some((uid, message)) => {
                trace!(
                    "MediatedConnection::update_state >>> handling message uid: {:?}",
                    uid
                );
                self.update_state_with_message(wallet, agency_client.clone(), Some(message))
                    .await?;
                self.cloud_agent_info()
                    .ok_or(AriesVcxError::from_msg(
                        AriesVcxErrorKind::NoAgentInformation,
                        "Missing cloud agent info",
                    ))?
                    .update_message_status(agency_client, self.pairwise_info(), uid)
                    .await?;
            }
            None => {
                trace!(
                    "MediatedConnection::update_state >>> trying to update state without message"
                );
                self.update_state_with_message(wallet, agency_client.clone(), None)
                    .await?;
            }
        }

        trace!(
            "MediatedConnection::update_state >>> after update_state {:?}",
            self.get_state()
        );
        Ok(())
    }

    async fn step_inviter(
        &self,
        wallet: &impl BaseWallet,
        message: Option<AriesMessage>,
        agency_client: &AgencyClient,
    ) -> VcxResult<(Self, bool)> {
        match self.connection_sm.clone() {
            SmMediatedConnection::Inviter(sm_inviter) => {
                let (sm_inviter, new_cloud_agent_info, can_autohop) = match message {
                    Some(message) => match message {
                        AriesMessage::Connection(Connection::Request(request)) => {
                            let send_message = self.send_message_closure_connection(wallet);
                            let new_pairwise_info = PairwiseInfo::create(wallet).await?;
                            let new_cloud_agent =
                                CloudAgentInfo::create(agency_client, &new_pairwise_info).await?;
                            let new_routing_keys = new_cloud_agent.routing_keys(agency_client)?;
                            let new_service_endpoint =
                                new_cloud_agent.service_endpoint(agency_client)?;

                            let sm_connection = sm_inviter
                                .handle_connection_request(
                                    wallet,
                                    request,
                                    &new_pairwise_info,
                                    new_routing_keys,
                                    new_service_endpoint,
                                    send_message,
                                )
                                .await?;
                            (sm_connection, Some(new_cloud_agent), true)
                        }
                        msg @ AriesMessage::Notification(_)
                        | msg @ AriesMessage::TrustPing(TrustPing::Ping(_)) => (
                            sm_inviter.handle_confirmation_message(&msg).await?,
                            None,
                            false,
                        ),
                        AriesMessage::Connection(Connection::ProblemReport(problem_report)) => (
                            sm_inviter.handle_problem_report(problem_report)?,
                            None,
                            false,
                        ),
                        _ => (sm_inviter.clone(), None, false),
                    },
                    None => {
                        if let MediatedInviterFullState::Requested(_) = sm_inviter.state_object() {
                            let send_message = self.send_message_closure_connection(wallet);
                            (
                                sm_inviter.handle_send_response(send_message).await?,
                                None,
                                false,
                            )
                        } else {
                            (sm_inviter.clone(), None, false)
                        }
                    }
                };

                let connection = Self {
                    cloud_agent_info: new_cloud_agent_info.or(self.cloud_agent_info.clone()),
                    connection_sm: SmMediatedConnection::Inviter(sm_inviter),
                    autohop_enabled: self.autohop_enabled,
                };

                Ok((connection, can_autohop))
            }
            SmMediatedConnection::Invitee(_) => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Invalid operation, called _step_inviter on Invitee connection.",
            )),
        }
    }

    async fn step_invitee(
        &self,
        wallet: &impl BaseWallet,
        message: Option<AriesMessage>,
    ) -> VcxResult<(Self, bool)> {
        match self.connection_sm.clone() {
            SmMediatedConnection::Invitee(sm_invitee) => {
                let (sm_invitee, can_autohop) = match message {
                    Some(message) => match message {
                        AriesMessage::Connection(Connection::Invitation(invitation))
                            if matches!(invitation.content, InvitationContent::Public(_)) =>
                        {
                            (
                                sm_invitee.handle_invitation(AnyInvitation::Con(invitation))?,
                                false,
                            )
                        }
                        AriesMessage::Connection(Connection::Invitation(invitation))
                            if matches!(invitation.content, InvitationContent::Pairwise(_)) =>
                        {
                            (
                                sm_invitee.handle_invitation(AnyInvitation::Con(invitation))?,
                                false,
                            )
                        }
                        AriesMessage::Connection(Connection::Response(response)) => {
                            let send_message = self.send_message_closure_connection(wallet);
                            (
                                sm_invitee
                                    .handle_connection_response(wallet, response, send_message)
                                    .await?,
                                true,
                            )
                        }
                        AriesMessage::Connection(Connection::ProblemReport(problem_report)) => {
                            (sm_invitee.handle_problem_report(problem_report)?, false)
                        }
                        _ => (sm_invitee, false),
                    },
                    None => {
                        let send_message = self.send_message_closure_connection(wallet);
                        (sm_invitee.handle_send_ack(send_message).await?, false)
                    }
                };
                let connection = Self {
                    connection_sm: SmMediatedConnection::Invitee(sm_invitee),
                    cloud_agent_info: self.cloud_agent_info.clone(),
                    autohop_enabled: self.autohop_enabled,
                };
                Ok((connection, can_autohop))
            }
            SmMediatedConnection::Inviter(_) => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Invalid operation, called _step_invitee on Inviter connection.",
            )),
        }
    }

    pub async fn connect<'a, 'b>(
        &'a mut self,
        wallet: &'b impl BaseWallet,
        agency_client: &AgencyClient,
        send_message: Option<SendClosureConnection<'b>>,
    ) -> VcxResult<()>
    where
        'a: 'b,
    {
        trace!(
            "MediatedConnection::connect >>> source_id: {}",
            self.source_id()
        );
        let cloud_agent_info = self
            .cloud_agent_info
            .clone()
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::NoAgentInformation,
                "Missing cloud agent info",
            ))?;
        let sm = match &self.connection_sm {
            SmMediatedConnection::Inviter(sm_inviter) => {
                SmMediatedConnection::Inviter(sm_inviter.clone().create_invitation(
                    cloud_agent_info.routing_keys(agency_client)?,
                    cloud_agent_info.service_endpoint(agency_client)?,
                )?)
            }
            SmMediatedConnection::Invitee(sm_invitee) => {
                let send_message =
                    send_message.unwrap_or(self.send_message_closure_connection(wallet));

                SmMediatedConnection::Invitee(
                    sm_invitee
                        .clone()
                        .send_connection_request(
                            cloud_agent_info.routing_keys(agency_client)?,
                            cloud_agent_info.service_endpoint(agency_client)?,
                            send_message,
                        )
                        .await?,
                )
            }
        };

        self.connection_sm = sm;
        Ok(())
    }

    pub async fn update_message_status(
        &self,
        uid: &str,
        agency_client: &AgencyClient,
    ) -> VcxResult<()> {
        trace!(
            "MediatedConnection::update_message_status >>> uid: {:?}",
            uid
        );
        self.cloud_agent_info()
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::NoAgentInformation,
                "Missing cloud agent info",
            ))?
            .update_message_status(agency_client, self.pairwise_info(), uid.to_string())
            .await
    }

    pub async fn get_messages_noauth(
        &self,
        agency_client: &AgencyClient,
    ) -> VcxResult<HashMap<String, AriesMessage>> {
        match &self.connection_sm {
            SmMediatedConnection::Inviter(sm_inviter) => {
                let messages = self
                    .cloud_agent_info()
                    .ok_or(AriesVcxError::from_msg(
                        AriesVcxErrorKind::NoAgentInformation,
                        "Missing cloud agent info",
                    ))?
                    .get_messages_noauth(agency_client, sm_inviter.pairwise_info(), None)
                    .await?;
                Ok(messages)
            }
            SmMediatedConnection::Invitee(sm_invitee) => {
                let messages = self
                    .cloud_agent_info()
                    .ok_or(AriesVcxError::from_msg(
                        AriesVcxErrorKind::NoAgentInformation,
                        "Missing cloud agent info",
                    ))?
                    .get_messages_noauth(agency_client, sm_invitee.pairwise_info(), None)
                    .await?;
                Ok(messages)
            }
        }
    }

    pub async fn get_messages(
        &self,
        agency_client: &AgencyClient,
    ) -> VcxResult<HashMap<String, AriesMessage>> {
        let expected_sender_vk = self.get_expected_sender_vk().await?;
        match &self.connection_sm {
            SmMediatedConnection::Inviter(sm_inviter) => Ok(self
                .cloud_agent_info()
                .ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::NoAgentInformation,
                    "Missing cloud agent info",
                ))?
                .get_messages(
                    agency_client,
                    &expected_sender_vk,
                    sm_inviter.pairwise_info(),
                )
                .await?),
            SmMediatedConnection::Invitee(sm_invitee) => Ok(self
                .cloud_agent_info()
                .ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::NoAgentInformation,
                    "Missing cloud agent info",
                ))?
                .get_messages(
                    agency_client,
                    &expected_sender_vk,
                    sm_invitee.pairwise_info(),
                )
                .await?),
        }
    }

    async fn get_expected_sender_vk(&self) -> VcxResult<String> {
        self.remote_vk().map_err(|_err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Verkey of Connection counterparty is not known, hence it would be impossible to \
                 authenticate message downloaded by id.",
            )
        })
    }

    pub async fn get_message_by_id(
        &self,
        msg_id: &str,
        agency_client: &AgencyClient,
    ) -> VcxResult<AriesMessage> {
        trace!(
            "MediatedConnection: get_message_by_id >>> msg_id: {}",
            msg_id
        );
        let expected_sender_vk = self.get_expected_sender_vk().await?;
        self.cloud_agent_info()
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::NoAgentInformation,
                "Missing cloud agent info",
            ))?
            .get_message_by_id(
                agency_client,
                msg_id,
                &expected_sender_vk,
                self.pairwise_info(),
            )
            .await
    }

    pub async fn send_message_closure<'a>(
        &self,
        wallet: &'a impl BaseWallet,
    ) -> VcxResult<SendClosure<'a>> {
        trace!("send_message_closure >>>");
        let did_doc = self.their_did_doc().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::NotReady,
            "Cannot send message: Remote Connection information is not set",
        ))?;
        let sender_vk = self.pairwise_info().pw_vk.clone();
        Ok(Box::new(move |message: AriesMessage| {
            Box::pin(send_message(wallet, sender_vk.clone(), did_doc, message))
        }))
    }

    fn send_message_closure_connection<'a>(
        &self,
        wallet: &'a impl BaseWallet,
    ) -> SendClosureConnection<'a> {
        trace!("send_message_closure_connection >>>");
        Box::new(
            move |message: AriesMessage, sender_vk: String, did_doc: AriesDidDoc| {
                Box::pin(send_message(wallet, sender_vk, did_doc, message))
            },
        )
    }

    fn build_basic_message(message: &str) -> AriesMessage {
        match ::serde_json::from_str::<AriesMessage>(message) {
            Ok(a2a_message) => a2a_message,
            Err(_) => {
                let now = Utc::now();

                let content = BasicMessageContent::builder()
                    .content(message.to_owned())
                    .sent_time(now)
                    .build();

                let decorators = BasicMessageDecorators::builder()
                    .timing(Timing::builder().out_time(now).build())
                    .build();

                BasicMessage::builder()
                    .id(Uuid::new_v4().to_string())
                    .content(content)
                    .decorators(decorators)
                    .build()
            }
        }
    }

    pub async fn send_generic_message(
        &self,
        wallet: &impl BaseWallet,
        message: &str,
    ) -> VcxResult<String> {
        trace!(
            "MediatedConnection::send_generic_message >>> message: {:?}",
            message
        );
        let message = Self::build_basic_message(message);
        let send_message = self.send_message_closure(wallet).await?;
        send_message(message).await.map(|_| String::new())
    }

    pub async fn send_a2a_message(
        &self,
        wallet: &impl BaseWallet,
        message: &AriesMessage,
    ) -> VcxResult<String> {
        trace!(
            "MediatedConnection::send_a2a_message >>> message: {:?}",
            message
        );
        let send_message = self.send_message_closure(wallet).await?;
        send_message(message.clone()).await.map(|_| String::new())
    }

    pub async fn send_ping(
        &mut self,
        wallet: &impl BaseWallet,
        comment: Option<String>,
    ) -> VcxResult<TrustPingSender> {
        let mut trust_ping = TrustPingSender::build(true, comment);
        trust_ping
            .send_ping(self.send_message_closure(wallet).await?)
            .await?;
        Ok(trust_ping)
    }

    pub async fn send_handshake_reuse(
        &self,
        wallet: &impl BaseWallet,
        oob_msg: &str,
    ) -> VcxResult<()> {
        trace!("MediatedConnection::send_handshake_reuse >>>");
        // todo: oob_msg argument should be typed OutOfBandInvitation, not string
        let oob = match serde_json::from_str::<AriesMessage>(oob_msg) {
            Ok(a2a_msg) => match a2a_msg {
                AriesMessage::OutOfBand(OutOfBand::Invitation(oob)) => oob,
                a => {
                    return Err(AriesVcxError::from_msg(
                        AriesVcxErrorKind::SerializationError,
                        format!("Received invalid message type: {:?}", a),
                    ));
                }
            },
            Err(err) => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::SerializationError,
                    format!("Failed to deserialize message, err: {:?}", err),
                ));
            }
        };
        let send_message = self.send_message_closure(wallet).await?;
        send_message(build_handshake_reuse_msg(&oob).into()).await
    }

    pub async fn delete(&self, agency_client: &AgencyClient) -> VcxResult<()> {
        trace!("Connection: delete >>> {:?}", self.source_id());
        self.cloud_agent_info()
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::NoAgentInformation,
                "Missing cloud agent info",
            ))?
            .destroy(agency_client, self.pairwise_info())
            .await
    }

    pub async fn get_connection_info(&self, agency_client: &AgencyClient) -> VcxResult<String> {
        trace!("MediatedConnection::get_connection_info >>>");

        let agent_info = self.cloud_agent_info().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::NoAgentInformation,
            "Missing cloud agent info",
        ))?;
        let pairwise_info = self.pairwise_info();
        let recipient_keys = vec![pairwise_info.pw_vk.clone()];

        let current = SideDetails {
            did: pairwise_info.pw_did.clone(),
            recipient_keys,
            routing_keys: agent_info.routing_keys(agency_client)?,
            service_endpoint: agent_info.service_endpoint(agency_client)?,
        };

        let remote = match self.their_did_doc() {
            Some(did_doc) => Some(SideDetails {
                did: did_doc.id.clone(),
                recipient_keys: did_doc.recipient_keys()?,
                routing_keys: did_doc.routing_keys(),
                service_endpoint: did_doc.get_endpoint().ok_or_else(|| {
                    AriesVcxError::from_msg(AriesVcxErrorKind::InvalidUrl, "No URL in DID Doc")
                })?,
            }),
            None => None,
        };

        let connection_info = ConnectionInfo {
            my: current,
            their: remote,
        };

        let connection_info_json = serde_json::to_string(&connection_info).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                format!("Cannot serialize ConnectionInfo: {:?}", err),
            )
        })?;

        Ok(connection_info_json)
    }

    pub async fn download_messages(
        &self,
        agency_client: &AgencyClient,
        status_codes: Option<Vec<MessageStatusCode>>,
        uids: Option<Vec<String>>,
    ) -> VcxResult<Vec<DownloadedMessage>> {
        match self.get_state() {
            MediatedConnectionState::Invitee(MediatedInviteeState::Initial)
            | MediatedConnectionState::Invitee(MediatedInviteeState::Requested)
            | MediatedConnectionState::Inviter(MediatedInviterState::Initial)
            | MediatedConnectionState::Inviter(MediatedInviterState::Invited) => {
                let msgs = futures::stream::iter(
                    self.cloud_agent_info()
                        .ok_or(AriesVcxError::from_msg(
                            AriesVcxErrorKind::NoAgentInformation,
                            "Missing cloud agent info",
                        ))?
                        .download_encrypted_messages(
                            agency_client,
                            uids,
                            status_codes,
                            self.pairwise_info(),
                        )
                        .await?,
                )
                .then(|msg| msg.decrypt_noauth(agency_client.get_wallet()))
                .filter_map(|res| async { res.ok() })
                .collect::<Vec<DownloadedMessage>>()
                .await;
                Ok(msgs)
            }
            _ => {
                let expected_sender_vk = self.remote_vk()?;
                let msgs = futures::stream::iter(
                    self.cloud_agent_info()
                        .ok_or(AriesVcxError::from_msg(
                            AriesVcxErrorKind::NoAgentInformation,
                            "Missing cloud agent info",
                        ))?
                        .download_encrypted_messages(
                            agency_client,
                            uids,
                            status_codes,
                            self.pairwise_info(),
                        )
                        .await?,
                )
                .then(|msg| msg.decrypt_auth(agency_client.get_wallet(), &expected_sender_vk))
                .filter_map(|res| async { res.ok() })
                .collect::<Vec<DownloadedMessage>>()
                .await;
                Ok(msgs)
            }
        }
    }

    pub fn to_string(&self) -> VcxResult<String> {
        serde_json::to_string(&self).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::SerializationError,
                format!("Cannot serialize Connection: {:?}", err),
            )
        })
    }

    pub fn from_string(connection_data: &str) -> VcxResult<Self> {
        serde_json::from_str(connection_data).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Cannot deserialize Connection: {:?}", err),
            )
        })
    }
}

impl Serialize for MediatedConnection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let (state, pairwise_info, cloud_agent_info, source_id, thread_id) = self.to_owned().into();
        let CloudAgentInfo {
            agent_did,
            agent_vk,
        } = cloud_agent_info.unwrap_or_default();
        let data = LegacyAgentInfo {
            pw_did: pairwise_info.pw_did,
            pw_vk: pairwise_info.pw_vk,
            agent_did,
            agent_vk,
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
    type Value = MediatedConnection;

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
                Ok((
                    state,
                    pairwise_info,
                    Some(cloud_agent_info),
                    source_id,
                    thread_id,
                )
                    .into())
            }
        }
    }
}

impl<'de> Deserialize<'de> for MediatedConnection {
    fn deserialize<D>(deserializer: D) -> Result<MediatedConnection, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ConnectionVisitor)
    }
}

impl From<MediatedConnection>
    for (
        SmConnectionState,
        PairwiseInfo,
        Option<CloudAgentInfo>,
        String,
        String,
    )
{
    fn from(
        s: MediatedConnection,
    ) -> (
        SmConnectionState,
        PairwiseInfo,
        Option<CloudAgentInfo>,
        String,
        String,
    ) {
        (
            s.state_object(),
            s.pairwise_info().to_owned(),
            s.cloud_agent_info(),
            s.source_id(),
            s.get_thread_id(),
        )
    }
}

impl
    From<(
        SmConnectionState,
        PairwiseInfo,
        Option<CloudAgentInfo>,
        String,
        String,
    )> for MediatedConnection
{
    fn from(
        (state, pairwise_info, cloud_agent_info, source_id, thread_id): (
            SmConnectionState,
            PairwiseInfo,
            Option<CloudAgentInfo>,
            String,
            String,
        ),
    ) -> MediatedConnection {
        MediatedConnection::from_parts(
            source_id,
            thread_id,
            pairwise_info,
            cloud_agent_info,
            state,
            true,
        )
    }
}
