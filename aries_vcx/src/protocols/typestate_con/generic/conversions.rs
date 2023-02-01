use super::{GenericConnection, GenericState, InviteeState, InviterState};
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    protocols::typestate_con::{
        common::states::{complete::CompleteState, responded::RespondedState},
        initiation_type::{Invitee, Inviter},
        invitee::states::{
            initial::Initial as InviteeInitial, invited::Invited as InviteeInvited,
            requested::Requested as InviteeRequested,
        },
        inviter::states::{
            initial::Initial as InviterInitial, invited::Invited as InviterInvited,
            requested::Requested as InviterRequested,
        },
        Connection,
    },
};

/// Macro used for boilerplace implementation of the
/// [`From`] trait from a concrete connection state to the equivalent vague state.
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
        impl<S> TryFrom<GenericState> for ($init_type, S)
        where
            S: TryFrom<$state, Error = AriesVcxError>,
        {
            type Error = AriesVcxError;

            fn try_from(value: GenericState) -> Result<Self, Self::Error> {
                match value {
                    GenericState::$good_var(s) => S::try_from(s).map(|s| ($init_type, s)),
                    GenericState::$bad_var(_) => Err(AriesVcxError::from_msg(
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

// ---------------------------- From Concrete State to Vague State implementations ----------------------------
impl<I, S> From<Connection<I, S>> for GenericConnection
where
    GenericState: From<(I, S)>,
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

from_concrete_to_vague!(Inviter, InviterState, Inviter, GenericState);
from_concrete_to_vague!(Invitee, InviteeState, Invitee, GenericState);

from_concrete_to_vague!(InviterInitial, Initial, InviterState);
from_concrete_to_vague!(InviterInvited, Invited, InviterState);
from_concrete_to_vague!(InviterRequested, Requested, InviterState);
from_concrete_to_vague!(RespondedState, Responded, InviterState);
from_concrete_to_vague!(CompleteState, Complete, InviterState);

from_concrete_to_vague!(InviteeInitial, Initial, InviteeState);
from_concrete_to_vague!(InviteeInvited, Invited, InviteeState);
from_concrete_to_vague!(InviteeRequested, Requested, InviteeState);
from_concrete_to_vague!(RespondedState, Responded, InviteeState);
from_concrete_to_vague!(CompleteState, Complete, InviteeState);

// ---------------------------- Try From Vague State to Concrete State implementations ----------------------------
impl<I, S> TryFrom<GenericConnection> for Connection<I, S>
where
    (I, S): TryFrom<GenericState, Error = AriesVcxError>,
{
    type Error = AriesVcxError;

    fn try_from(value: GenericConnection) -> Result<Self, Self::Error> {
        let (initiation_type, state) = TryFrom::try_from(value.state)?;
        let con = Connection::from_parts(value.source_id, value.pairwise_info, initiation_type, state);
        Ok(con)
    }
}

try_from_vague_to_concrete!(InviterState, Inviter, Invitee, Inviter);
try_from_vague_to_concrete!(InviteeState, Invitee, Inviter, Invitee);

try_from_vague_to_concrete!(InviterState, Initial, InviterInitial);
try_from_vague_to_concrete!(InviterState, Invited, InviterInvited);
try_from_vague_to_concrete!(InviterState, Requested, InviterRequested);
try_from_vague_to_concrete!(InviterState, Responded, RespondedState);
try_from_vague_to_concrete!(InviterState, Complete, CompleteState);

try_from_vague_to_concrete!(InviteeState, Initial, InviteeInitial);
try_from_vague_to_concrete!(InviteeState, Invited, InviteeInvited);
try_from_vague_to_concrete!(InviteeState, Requested, InviteeRequested);
try_from_vague_to_concrete!(InviteeState, Responded, RespondedState);
try_from_vague_to_concrete!(InviteeState, Complete, CompleteState);
