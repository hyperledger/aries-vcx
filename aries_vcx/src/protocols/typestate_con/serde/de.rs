use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind},
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
        Connection,
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

/// Type used for deserialization of a [`Connection`].
/// This struct cannot be used for anything useful directly,
/// but the inner `state` has to be matched to get the concrete [`Connection`] type.
#[derive(Debug, Deserialize)]
pub struct VagueConnection {
    source_id: String,
    pairwise_info: PairwiseInfo,
    state: VagueState,
}

#[derive(Debug, Deserialize)]
pub enum VagueState {
    Inviter(VagueInviterState),
    Invitee(VagueInviteeState),
}

#[derive(Debug, Deserialize)]
pub enum VagueInviterState {
    Initial(InviterInitial),
    Invited(InviterInvited),
    Requested(InviterRequested),
    Responded(RespondedState),
    Complete(CompleteState),
}

#[derive(Debug, Deserialize)]
pub enum VagueInviteeState {
    Initial(InviteeInitial),
    Invited(InviteeInvited),
    Requested(InviteeRequested),
    Responded(RespondedState),
    Complete(CompleteState),
}

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
