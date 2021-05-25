use std::collections::HashMap;

use crate::error::prelude::*;
use crate::aries::handlers::connection::agent_info::AgentInfo;
use crate::aries::handlers::connection::invitee::state_machine::{InviteeState, SmConnectionInvitee};
use crate::aries::handlers::connection::inviter::state_machine::{InviterState, SmConnectionInviter};
use crate::aries::messages::a2a::A2AMessage;
use crate::aries::messages::basic_message::message::BasicMessage;
use crate::aries::messages::connection::did_doc::DidDoc;
use crate::aries::messages::connection::invite::Invitation;
use crate::aries::messages::discovery::disclose::ProtocolDescriptor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    connection_sm: SmConnection
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmConnection {
    Inviter(SmConnectionInviter),
    Invitee(SmConnectionInvitee),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmConnectionState {
    Inviter(InviterState),
    Invitee(InviteeState),
}

#[derive(Debug, Serialize)]
struct ConnectionInfo {
    my: SideConnectionInfo,
    their: Option<SideConnectionInfo>,
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
    /**
    Create Inviter connection state machine
     */
    pub fn create(source_id: &str) -> Connection {
        trace!("Connection::create >>> source_id: {}", source_id);

        Connection {
            connection_sm: SmConnection::Inviter(SmConnectionInviter::new(source_id))
        }
    }

    pub fn from_parts(source_id: String, agent_info: AgentInfo, state: SmConnectionState) -> Connection {
        match state {
            SmConnectionState::Inviter(state) => {
                Connection { connection_sm: SmConnection::Inviter(SmConnectionInviter::from(source_id, agent_info, state)) }
            }
            SmConnectionState::Invitee(state) => {
                Connection { connection_sm: SmConnection::Invitee(SmConnectionInvitee::from(source_id, agent_info, state)) }
            }
        }
    }

    /**
    Create Invitee connection state machine
     */
    pub fn create_with_invite(source_id: &str, invitation: Invitation) -> VcxResult<Connection> {
        trace!("Connection::create_with_invite >>> source_id: {}", source_id);

        let mut connection = Connection {
            connection_sm: SmConnection::Invitee(SmConnectionInvitee::new(source_id))
        };

        connection.process_invite(invitation)?;

        Ok(connection)
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

    pub fn state(&self) -> u32 {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                sm_inviter.state()
            }
            SmConnection::Invitee(sm_invitee) => {
                sm_invitee.state()
            }
        }
    }

    pub fn agent_info(&self) -> &AgentInfo {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                sm_inviter.agent_info()
            }
            SmConnection::Invitee(sm_invitee) => {
                sm_invitee.agent_info()
            }
        }
    }

    pub fn bootstrap_agent_info(&self) -> Option<&AgentInfo> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                sm_inviter.prev_agent_info()
            }
            SmConnection::Invitee(_sm_invitee) => None
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

    /**
    Invitee operation
     */
    pub fn process_invite(&mut self, invitation: Invitation) -> VcxResult<()> {
        trace!("Connection::process_invite >>> invitation: {:?}", invitation);
        self.connection_sm = match &self.connection_sm {
            SmConnection::Inviter(_sm_inviter) => {
                return Err(VcxError::from_msg(VcxErrorKind::NotReady, "Invalid action"))
            }
            SmConnection::Invitee(sm_invitee) => {
                SmConnection::Invitee(sm_invitee.clone().transition_receive_invitation(invitation)?)
            }
        };
        Ok(())
    }

    /**
    If called on Inviter in Invited state returns invitation to connect with him. Returns error in other states.
    If called on Invitee, returns error
     */
    pub fn get_invite_details(&self) -> Option<String> {
        trace!("Connection::get_invite_details >>>");
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                sm_inviter.get_invitation().map(|invitation| {
                    json!(invitation.to_a2a_message()).to_string()
                })
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

    /**
    If called on Inviter, creates initial connection agent and generates invitation
    If called on Invitee, creates connection agent and send connection request using info from connection invitation
     */
    pub fn connect(&mut self) -> VcxResult<()> {
        trace!("Connection::connect >>> source_id: {}", self.source_id());
        self.connection_sm = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                SmConnection::Inviter(sm_inviter.clone().transition_connect()?)
            }
            SmConnection::Invitee(sm_invitee) => {
                SmConnection::Invitee(sm_invitee.clone().transition_connect()?)
            }
        };
        Ok(())
    }

    /**
    Perform state machine transition using supplied message.
     */
    pub fn update_state_with_message(&mut self, message: &A2AMessage) -> VcxResult<()> {
        trace!("Connection: update_state_with_message: {:?}", message);
        if self.is_in_null_state() {
            warn!("Connection::update_state_with_message :: update state on connection in null state is ignored");
            return Ok(());
        }

        self.update_with_message(message.clone())?;

        Ok(())
    }


    /**
    Updates status of a message (received from connection counterparty) in agency.
     */
    pub fn update_message_status(&self, uid: String) -> VcxResult<()> {
        trace!("Connection::update_message_status >>> uid: {:?}", uid);
        self.agent_info().update_message_status(uid)
    }

    /**
Get messages received from connection counterparty.
 */
    pub fn get_messages_noauth(&self) -> VcxResult<HashMap<String, A2AMessage>> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                let messages = sm_inviter.agent_info().get_messages_noauth()?;
                Ok(messages)
            }
            SmConnection::Invitee(sm_invitee) => {
                let messages = sm_invitee.agent_info().get_messages_noauth()?;
                Ok(messages)
            }
        }
    }

    /**
    Get messages received from connection counterparty.
     */
    pub fn get_messages(&self) -> VcxResult<HashMap<String, A2AMessage>> {
        let expected_sender_vk = self.get_expected_sender_vk()?;
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                let messages = sm_inviter.agent_info().get_messages(&expected_sender_vk)?;
                Ok(messages)
            }
            SmConnection::Invitee(sm_invitee) => {
                let messages = sm_invitee.agent_info().get_messages(&expected_sender_vk)?;
                Ok(messages)
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

    /**
    Get messages received from connection counterparty by id.
     */
    pub fn get_message_by_id(&self, msg_id: &str) -> VcxResult<A2AMessage> {
        trace!("Connection: get_message_by_id >>> msg_id={}", msg_id);
        let expected_sender_vk = self.get_expected_sender_vk()?;
        self.agent_info().get_message_by_id(msg_id, &expected_sender_vk)
    }

    pub fn send_message_closure(&self) -> VcxResult<impl Fn(&A2AMessage) -> VcxResult<()>> {
        trace!("send_message_closure >>>");
        let did_doc = self.their_did_doc()
            .ok_or(VcxError::from_msg(VcxErrorKind::NotReady, "Cannot send message: Remote Connection information is not set"))?;
        let sender_vk = self.agent_info().pw_vk.clone();
        return Ok(move |a2a_message: &A2AMessage| {
            did_doc.send_message(a2a_message, &sender_vk)
        })
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

    pub fn send_generic_message(&self, message: &str) -> VcxResult<String> {
        trace!("Connection::send_generic_message >>> message: {:?}", message);

        let message = Connection::parse_generic_message(message);
        let send_message = self.send_message_closure()?;
        send_message(&message).map(|_| String::new())
    }

    pub fn send_ping(&mut self, comment: Option<String>) -> VcxResult<()> {
        trace!("Connection::send_ping >>> comment: {:?}", comment);
        // self.handle_message(DidExchangeMessages::DiscoverFeatures((query, comment)))
        self.connection_sm = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                SmConnection::Inviter(sm_inviter.clone().transition_send_ping(comment)?)
            }
            SmConnection::Invitee(sm_invitee) => {
                SmConnection::Invitee(sm_invitee.clone().transition_send_ping(comment)?)
            }
        };
        Ok(())
    }

    pub fn delete(&self) -> VcxResult<()> {
        trace!("Connection: delete >>> {:?}", self.source_id());
        self.agent_info().delete()
    }

    fn _step_inviter(&mut self, sm_inviter: SmConnectionInviter, message: A2AMessage) -> VcxResult<SmConnection> {
        let sm_inviter = match message {
            A2AMessage::ConnectionRequest(request) => {
                sm_inviter.transition_receive_connection_request(request)
            }
            A2AMessage::Ack(ack) => {
                sm_inviter.transition_receive_ack(ack)
            }
            A2AMessage::Ping(ping) => {
                sm_inviter.transition_receive_ping(ping)
            }
            A2AMessage::ConnectionProblemReport(problem_report) => {
                sm_inviter.transition_receive_problem_report(problem_report)
            }
            A2AMessage::PingResponse(ping_response) => {
                sm_inviter.transition_ping_response_received(ping_response)
            }
            // A2AMessage::Disclose((query_, comment)) => {
            //     sm_inviter.transition_discover_features_received(query_, comment) // todo
            // }
            A2AMessage::Query(query) => {
                sm_inviter.transition_discovery_query_received(query)
            }
            A2AMessage::Disclose(disclose) => {
                sm_inviter.transition_disclose_received(disclose)
            }
            _ => {
                Ok(sm_inviter)
            }
        }?;
        Ok(SmConnection::Inviter(sm_inviter))
    }

    fn _step_invitee(&mut self, sm_invitee: SmConnectionInvitee, message: A2AMessage) -> VcxResult<SmConnection> {
        let sm_invitee = match message {
            A2AMessage::ConnectionInvitation(invitation) => {
                sm_invitee.transition_receive_invitation(invitation)
            },
            A2AMessage::ConnectionResponse(response) => {
                sm_invitee.transition_receive_connection_response(response)
            }
            A2AMessage::Ack(ack) => {
                sm_invitee.transition_receive_ack(ack)
            }
            A2AMessage::Ping(ping) => {
                sm_invitee.transition_receive_ping(ping)
            }
            A2AMessage::ConnectionProblemReport(problem_report) => {
                sm_invitee.transition_receive_problem_report(problem_report)
            }
            A2AMessage::PingResponse(ping_response) => {
                sm_invitee.transition_ping_response_received(ping_response)
            }
            // DidExchangeMessages::DiscoverFeatures((query_, comment)) => { // todo
            //     sm_invitee.transition_discover_features_received(query_, comment)
            // }
            A2AMessage::Query(query) => {
                sm_invitee.transition_discovery_query_received(query)
            }
            A2AMessage::Disclose(disclose) => {
                sm_invitee.transition_disclose_received(disclose)
            }
            _ => {
                Ok(sm_invitee)
            }
        }?;
        Ok(SmConnection::Invitee(sm_invitee))
    }

    pub fn update_with_message(&mut self, message: A2AMessage) -> VcxResult<()>{
        self.connection_sm = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                self._step_inviter(sm_inviter.clone(), message)?
            }
            SmConnection::Invitee(sm_invitee) => {
                self._step_invitee(sm_invitee.clone(), message)?
            }
        };
        Ok(())
    }

    pub fn send_discovery_features(&mut self, query: Option<String>, comment: Option<String>) -> VcxResult<()> {
        trace!("Connection::send_discovery_features_query >>> query: {:?}, comment: {:?}", query, comment);
        self.connection_sm = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                SmConnection::Inviter(sm_inviter.clone().transition_discover_features_received(query, comment)?)
            }
            SmConnection::Invitee(sm_invitee) => {
                SmConnection::Invitee(sm_invitee.clone().transition_discover_features_received(query, comment)?)
            }
        };
        Ok(())
    }

    pub fn get_connection_info(&self) -> VcxResult<String> {
        trace!("Connection::get_connection_info >>>");

        let agent_info = self.agent_info().clone();

        let current = SideConnectionInfo {
            did: agent_info.pw_did.clone(),
            recipient_keys: agent_info.recipient_keys().clone(),
            routing_keys: agent_info.routing_keys()?,
            service_endpoint: agent_info.agency_endpoint()?,
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
}
