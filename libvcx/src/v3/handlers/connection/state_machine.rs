use std::collections::HashMap;

use api::VcxStateType;
use error::prelude::*;
use v3::handlers::connection::agent_info::AgentInfo;
use v3::handlers::connection::messages::DidExchangeMessages;
use v3::handlers::connection::states::complete::CompleteState;
use v3::handlers::connection::states::invited::InvitedState;
use v3::handlers::connection::states::null::NullState;
use v3::handlers::connection::states::requested::RequestedState;
use v3::handlers::connection::states::responded::RespondedState;
use v3::messages::a2a::A2AMessage;
use v3::messages::a2a::protocol_registry::ProtocolRegistry;
use v3::messages::ack::Ack;
use v3::messages::connection::did_doc::DidDoc;
use v3::messages::connection::invite::Invitation;
use v3::messages::connection::problem_report::{ProblemCode, ProblemReport};
use v3::messages::connection::request::Request;
use v3::messages::connection::response::{Response, SignedResponse};
use v3::messages::discovery::disclose::{Disclose, ProtocolDescriptor};
use v3::messages::discovery::query::Query;
use v3::messages::trust_ping::ping::Ping;
use v3::messages::trust_ping::ping_response::PingResponse;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Actor {
    Inviter,
    Invitee,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidExchangeSM {
    pub(super) source_id: String,
    pub(super) agent_info: AgentInfo,
    pub(super) state: ActorDidExchangeState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActorDidExchangeState {
    Inviter(DidExchangeState),
    Invitee(DidExchangeState),
}

/// Transitions of Inviter Connection state
/// Null -> Invited
/// Invited -> Responded, Null
/// Responded -> Complete, Null
/// Completed
///
/// Transitions of Invitee Connection state
/// Null -> Invited
/// Invited -> Requested, Null
/// Requested -> Completed, Null
/// Completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DidExchangeState {
    Null(NullState),
    Invited(InvitedState),
    Requested(RequestedState),
    Responded(RespondedState),
    Completed(CompleteState),
}

impl DidExchangeState {
    pub fn code(&self) -> u32 {
        match self {
            DidExchangeState::Null(_) => VcxStateType::VcxStateInitialized as u32,
            DidExchangeState::Invited(_) => VcxStateType::VcxStateOfferSent as u32,
            DidExchangeState::Requested(_) => VcxStateType::VcxStateRequestReceived as u32,
            DidExchangeState::Responded(_) => VcxStateType::VcxStateRequestReceived as u32,
            DidExchangeState::Completed(_) => VcxStateType::VcxStateAccepted as u32,
        }
    }
}

impl DidExchangeSM {
    pub fn new(actor: Actor, source_id: &str) -> Self {
        match actor {
            Actor::Inviter => {
                DidExchangeSM::_build_inviter(source_id)
            }
            Actor::Invitee => {
                DidExchangeSM::_build_invitee(source_id)
            }
        }
    }

    pub fn is_in_null_state(&self) -> bool {
        match self.state {
            ActorDidExchangeState::Inviter(DidExchangeState::Null(_)) => true,
            ActorDidExchangeState::Invitee(DidExchangeState::Null(_)) => true,
            _ => false
        }
    }

    pub fn from(source_id: String, agent_info: AgentInfo, state: ActorDidExchangeState) -> Self {
        DidExchangeSM {
            source_id,
            agent_info,
            state,
        }
    }

    pub fn agent_info(&self) -> &AgentInfo {
        &self.agent_info
    }

    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    pub fn state(&self) -> u32 {
        match self.state {
            ActorDidExchangeState::Inviter(ref state) | ActorDidExchangeState::Invitee(ref state) => state.code(),
        }
    }

    pub fn state_object(&self) -> &ActorDidExchangeState {
        &self.state
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        trace!("DidExchangeSM::find_message_to_handle >>> messages: {:?}", messages);

        for (uid, message) in messages {
            match &self.state {
                ActorDidExchangeState::Inviter(inviter_state) => {
                    if self.inviter_can_handle_message(inviter_state, &message) {
                        return Some((uid, message));
                    }
                }
                ActorDidExchangeState::Invitee(invitee_state) => {
                    if self.invitee_can_handle_message(invitee_state, &message) {
                        return Some((uid, message));
                    }
                }
            }
        }
        None
    }

    pub fn step(self, message: DidExchangeMessages) -> VcxResult<DidExchangeSM> {
        trace!("DidExchangeStateSM::step >>> message: {:?}", message);
        let DidExchangeSM { source_id, mut agent_info, state } = self;

        let (new_state, agent_info) = match state {
            ActorDidExchangeState::Inviter(inviter_state) => {
                DidExchangeSM::inviter_step(inviter_state, message, &source_id, agent_info)?
            }
            ActorDidExchangeState::Invitee(invitee_state) => {
                DidExchangeSM::invitee_step(invitee_state, message, &source_id, agent_info)?
            }
        };
        Ok(DidExchangeSM { source_id, agent_info, state: new_state })
    }

    pub fn their_did_doc(&self) -> Option<DidDoc> {
        match self.state {
            ActorDidExchangeState::Inviter(ref state) =>
                match state {
                    DidExchangeState::Null(_) => None,
                    DidExchangeState::Invited(_state) => None,
                    DidExchangeState::Requested(ref state) => Some(state.did_doc.clone()),
                    DidExchangeState::Responded(ref state) => Some(state.did_doc.clone()),
                    DidExchangeState::Completed(ref state) => Some(state.did_doc.clone()),
                },
            ActorDidExchangeState::Invitee(ref state) =>
                match state {
                    DidExchangeState::Null(_) => None,
                    DidExchangeState::Invited(ref state) => Some(DidDoc::from(state.invitation.clone())),
                    DidExchangeState::Requested(ref state) => Some(state.did_doc.clone()),
                    DidExchangeState::Responded(ref state) => Some(state.did_doc.clone()),
                    DidExchangeState::Completed(ref state) => Some(state.did_doc.clone()),
                }
        }
    }

    pub fn get_invitation(&self) -> Option<&Invitation> {
        match self.state {
            ActorDidExchangeState::Inviter(DidExchangeState::Invited(ref state)) |
            ActorDidExchangeState::Invitee(DidExchangeState::Invited(ref state)) => Some(&state.invitation),
            _ => None
        }
    }

    pub fn get_protocols(&self) -> Vec<ProtocolDescriptor> {
        ProtocolRegistry::init().protocols()
    }

    pub fn get_remote_protocols(&self) -> Option<Vec<ProtocolDescriptor>> {
        match self.state {
            ActorDidExchangeState::Inviter(DidExchangeState::Completed(ref state)) |
            ActorDidExchangeState::Invitee(DidExchangeState::Completed(ref state)) => state.protocols.clone(),
            _ => None
        }
    }

    pub fn remote_did(&self) -> VcxResult<String> {
        self.their_did_doc()
            .map(|did_doc: DidDoc| did_doc.id.clone())
            .ok_or(VcxError::from_msg(VcxErrorKind::NotReady, "Remote Connection DID is not set"))
    }

    pub fn remote_vk(&self) -> VcxResult<String> {
        self.their_did_doc()
            .and_then(|did_doc| did_doc.recipient_keys().get(0).cloned())
            .ok_or(VcxError::from_msg(VcxErrorKind::NotReady, "Remote Connection Verkey is not set"))
    }

    pub fn prev_agent_info(&self) -> Option<&AgentInfo> {
        match self.state {
            ActorDidExchangeState::Inviter(DidExchangeState::Responded(ref state)) => Some(&state.prev_agent_info),
            _ => None
        }
    }

    pub fn actor(&self) -> Actor {
        match self.state {
            ActorDidExchangeState::Inviter(_) => Actor::Inviter,
            ActorDidExchangeState::Invitee(_) => Actor::Invitee
        }
    }
}