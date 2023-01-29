use std::sync::Arc;

use messages::{a2a::A2AMessage, diddoc::aries::diddoc::AriesDidDoc, protocols::connection::invite::Invitation};

use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    plugins::wallet::base_wallet::BaseWallet,
    protocols::typestate_con::{
        common::states::{complete::CompleteState, responded::RespondedState},
        initiation_type::{Invitee, Inviter},
        invitee::states::{
            initial::InitialState as InviteeInitial, invited::InvitedState as InviteeInvited,
            requested::RequestedState as InviteeRequested,
        },
        inviter::states::{
            initial::InitialState as InviterInitial, invited::InvitedState as InviterInvited,
            requested::RequestedState as InviterRequested,
        },
        pairwise_info::PairwiseInfo,
        traits::{TheirDidDoc, ThreadId},
        Connection, Transport,
    },
};

/// Macro used for boilerplace implementation of the
/// [`From`] trait from a concrete connection state to the vague state.
macro_rules! from_concrete_to_vague {
    ($from:ident, $var:ident, $to:ident) => {
        impl From<$from> for $to {
            fn from(value: $from) -> Self {
                Self::$var(value)
            }
        }
    };

    ($init_type:ident, $state:ident, $var:ident, $to:ident) => {
        impl<S> From<($init_type, S)> for $to
        where
            $state: From<S>,
        {
            fn from(value: ($init_type, S)) -> Self {
                let (_, state) = value;
                let serde_state = From::from(state);
                Self::$var(serde_state)
            }
        }
    };
}

/// Macro used for boilerplace implementation of the
/// [`TryFrom`] trait from a vague connection state to a concrete state.
macro_rules! try_from_vague_to_concrete {
    ($from:ident, $var:ident, $to:ident) => {
        impl TryFrom<$from> for $to {
            type Error = AriesVcxError;

            fn try_from(value: $from) -> Result<Self, Self::Error> {
                match value {
                    $from::$var(s) => Ok(s),
                    _ => Err(AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidState,
                        format!("unexpected connection state: {:?}!", value),
                    )),
                }
            }
        }
    };

    ($state:ident, $good_var:ident, $bad_var:ident, $init_type:ident) => {
        impl<S> TryFrom<VagueState> for ($init_type, S)
        where
            S: TryFrom<$state, Error = AriesVcxError>,
        {
            type Error = AriesVcxError;

            fn try_from(value: VagueState) -> Result<Self, Self::Error> {
                match value {
                    VagueState::$good_var(s) => S::try_from(s).map(|s| ($init_type, s)),
                    VagueState::$bad_var(_) => Err(AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidState,
                        concat!(
                            "Expected ",
                            stringify!(VagueState::$good_var),
                            "connection state, found ",
                            stringify!(VagueState::$bad_var),
                        ),
                    )),
                }
            }
        }
    };
}

/// Helper type mainly used for deserialization of a [`Connection`].
/// It does not expose methods to advance the connection protocol
/// It does, however, expose some methods agnostic to the [`Connection`] type.
#[derive(Debug, Serialize, Deserialize)]
pub struct VagueConnection {
    source_id: String,
    pairwise_info: PairwiseInfo,
    state: VagueState,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum VagueState {
    Inviter(VagueInviterState),
    Invitee(VagueInviteeState),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum VagueInviterState {
    Initial(InviterInitial),
    Invited(InviterInvited),
    Requested(InviterRequested),
    Responded(RespondedState),
    Complete(CompleteState),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum VagueInviteeState {
    Initial(InviteeInitial),
    Invited(InviteeInvited),
    Requested(InviteeRequested),
    Responded(RespondedState),
    Complete(CompleteState),
}

// ---------------------------- From Concrete State to Vague State implementations ----------------------------
impl<I, S> From<Connection<I, S>> for VagueConnection
where
    VagueState: From<(I, S)>,
{
    fn from(value: Connection<I, S>) -> Self {
        let state = From::from((value.initiation_type, value.state));
        Self {
            source_id: value.source_id,
            pairwise_info: value.pairwise_info,
            state,
        }
    }
}

from_concrete_to_vague!(Inviter, VagueInviterState, Inviter, VagueState);
from_concrete_to_vague!(Invitee, VagueInviteeState, Invitee, VagueState);

from_concrete_to_vague!(InviterInitial, Initial, VagueInviterState);
from_concrete_to_vague!(InviterInvited, Invited, VagueInviterState);
from_concrete_to_vague!(InviterRequested, Requested, VagueInviterState);
from_concrete_to_vague!(RespondedState, Responded, VagueInviterState);
from_concrete_to_vague!(CompleteState, Complete, VagueInviterState);

from_concrete_to_vague!(InviteeInitial, Initial, VagueInviteeState);
from_concrete_to_vague!(InviteeInvited, Invited, VagueInviteeState);
from_concrete_to_vague!(InviteeRequested, Requested, VagueInviteeState);
from_concrete_to_vague!(RespondedState, Responded, VagueInviteeState);
from_concrete_to_vague!(CompleteState, Complete, VagueInviteeState);

// ---------------------------- Try From Vague State to Concrete State implementations ----------------------------
impl<I, S> TryFrom<VagueConnection> for Connection<I, S>
where
    (I, S): TryFrom<VagueState, Error = AriesVcxError>,
{
    type Error = AriesVcxError;

    fn try_from(value: VagueConnection) -> Result<Self, Self::Error> {
        let (initiation_type, state) = TryFrom::try_from(value.state)?;
        let con = Connection::from_parts(value.source_id, value.pairwise_info, initiation_type, state);
        Ok(con)
    }
}

try_from_vague_to_concrete!(VagueInviterState, Inviter, Invitee, Inviter);
try_from_vague_to_concrete!(VagueInviteeState, Invitee, Inviter, Invitee);

try_from_vague_to_concrete!(VagueInviterState, Initial, InviterInitial);
try_from_vague_to_concrete!(VagueInviterState, Invited, InviterInvited);
try_from_vague_to_concrete!(VagueInviterState, Requested, InviterRequested);
try_from_vague_to_concrete!(VagueInviterState, Responded, RespondedState);
try_from_vague_to_concrete!(VagueInviterState, Complete, CompleteState);

try_from_vague_to_concrete!(VagueInviteeState, Initial, InviteeInitial);
try_from_vague_to_concrete!(VagueInviteeState, Invited, InviteeInvited);
try_from_vague_to_concrete!(VagueInviteeState, Requested, InviteeRequested);
try_from_vague_to_concrete!(VagueInviteeState, Responded, RespondedState);
try_from_vague_to_concrete!(VagueInviteeState, Complete, CompleteState);

/// Small sized enum used for determining
/// a connection's state in terms of initiation type.
#[derive(Clone, Copy, Debug)]
pub enum State {
    Invitee(ConState),
    Inviter(ConState),
}

/// Small sized enum used for determining
/// a connection's state in terms of connection stage.
#[derive(Clone, Copy, Debug)]
pub enum ConState {
    Initial,
    Invited,
    Requested,
    Responded,
    Complete,
}

impl From<State> for u32 {
    fn from(value: State) -> Self {
        match value {
            State::Invitee(v) => v as u32,
            State::Inviter(v) => v as u32,
        }
    }
}

impl From<&VagueState> for State {
    fn from(value: &VagueState) -> Self {
        match value {
            VagueState::Invitee(v) => Self::Invitee(v.into()),
            VagueState::Inviter(v) => Self::Inviter(v.into()),
        }
    }
}

impl From<&VagueInviterState> for ConState {
    fn from(value: &VagueInviterState) -> Self {
        match value {
            VagueInviterState::Initial(_) => Self::Initial,
            VagueInviterState::Invited(_) => Self::Invited,
            VagueInviterState::Requested(_) => Self::Requested,
            VagueInviterState::Responded(_) => Self::Responded,
            VagueInviterState::Complete(_) => Self::Complete,
        }
    }
}

impl From<&VagueInviteeState> for ConState {
    fn from(value: &VagueInviteeState) -> Self {
        match value {
            VagueInviteeState::Initial(_) => Self::Initial,
            VagueInviteeState::Invited(_) => Self::Invited,
            VagueInviteeState::Requested(_) => Self::Requested,
            VagueInviteeState::Responded(_) => Self::Responded,
            VagueInviteeState::Complete(_) => Self::Complete,
        }
    }
}

impl VagueConnection {
    pub fn state(&self) -> State {
        (&self.state).into()
    }

    pub fn thread_id(&self) -> Option<&str> {
        match &self.state {
            VagueState::Invitee(VagueInviteeState::Initial(_)) => None,
            VagueState::Invitee(VagueInviteeState::Invited(s)) => Some(s.thread_id()),
            VagueState::Invitee(VagueInviteeState::Requested(s)) => Some(s.thread_id()),
            VagueState::Invitee(VagueInviteeState::Responded(s)) => Some(s.thread_id()),
            VagueState::Invitee(VagueInviteeState::Complete(s)) => Some(s.thread_id()),
            VagueState::Inviter(VagueInviterState::Initial(s)) => Some(s.thread_id()),
            VagueState::Inviter(VagueInviterState::Invited(s)) => s.thread_id(),
            VagueState::Inviter(VagueInviterState::Requested(s)) => Some(s.thread_id()),
            VagueState::Inviter(VagueInviterState::Responded(s)) => Some(s.thread_id()),
            VagueState::Inviter(VagueInviterState::Complete(s)) => Some(s.thread_id()),
        }
    }

    pub fn pairwise_info(&self) -> &PairwiseInfo {
        &self.pairwise_info
    }

    pub fn their_did_doc(&self) -> Option<&AriesDidDoc> {
        match &self.state {
            VagueState::Invitee(VagueInviteeState::Initial(_)) => None,
            VagueState::Invitee(VagueInviteeState::Invited(s)) => Some(s.their_did_doc()),
            VagueState::Invitee(VagueInviteeState::Requested(s)) => Some(s.their_did_doc()),
            VagueState::Invitee(VagueInviteeState::Responded(s)) => Some(s.their_did_doc()),
            VagueState::Invitee(VagueInviteeState::Complete(s)) => Some(s.their_did_doc()),
            VagueState::Inviter(VagueInviterState::Initial(_)) => None,
            VagueState::Inviter(VagueInviterState::Invited(_)) => None,
            VagueState::Inviter(VagueInviterState::Requested(s)) => Some(s.their_did_doc()),
            VagueState::Inviter(VagueInviterState::Responded(s)) => Some(s.their_did_doc()),
            VagueState::Inviter(VagueInviterState::Complete(s)) => Some(s.their_did_doc()),
        }
    }

    pub fn remote_did(&self) -> Option<&str> {
        self.their_did_doc().map(|d| d.id.as_str())
    }

    pub fn remote_vk(&self) -> VcxResult<String> {
        let did_doc = self.their_did_doc().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::NotReady,
            "No DidDoc present",
        ))?;

        did_doc
            .recipient_keys()?
            .first()
            .map(ToOwned::to_owned)
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Can't resolve recipient key from the counterparty diddoc.",
            ))
    }

    pub fn invitation(&self) -> Option<&Invitation> {
        match &self.state {
            VagueState::Inviter(VagueInviterState::Initial(s)) => Some(&s.invitation),
            _ => None,
        }
    }

    pub async fn send_message<T>(
        &self,
        wallet: &Arc<dyn BaseWallet>,
        message: &A2AMessage,
        transport: &T,
    ) -> VcxResult<()>
    where
        T: Transport,
    {
        let sender_verkey = &self.pairwise_info().pw_vk;
        let did_doc = self.their_did_doc().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::NotReady,
            "No DidDoc present",
        ))?;

        Connection::<(), ()>::basic_send_message(wallet, message, sender_verkey, did_doc, transport).await
    }
}
