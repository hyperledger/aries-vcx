use serde::Serialize;

use crate::protocols::typestate_con::{
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
};

impl<I, S> Serialize for Connection<I, S>
where
    for<'a> SerializableConnection<'a>: From<&'a Connection<I, S>>,
{
    fn serialize<Serializer>(&self, serializer: Serializer) -> Result<Serializer::Ok, Serializer::Error>
    where
        Serializer: ::serde::Serializer,
    {
        SerializableConnection::from(self).serialize(serializer)
    }
}

/// Type used for serialization of a [`Connection`].
/// This struct is used transparently, under the hood, to convert a reference
/// of a [`Connection`] (so we don't clone unnecessarily) to itself and then serialize it.
#[derive(Debug, Serialize)]
pub struct SerializableConnection<'a> {
    pub(super) source_id: &'a str,
    pub(super) pairwise_info: &'a PairwiseInfo,
    pub(super) state: RefState<'a>,
}

impl<'a> SerializableConnection<'a> {
    fn new(source_id: &'a str, pairwise_info: &'a PairwiseInfo, state: RefState<'a>) -> Self {
        Self {
            source_id,
            pairwise_info,
            state,
        }
    }
}

impl<'a, I, S> From<&'a Connection<I, S>> for SerializableConnection<'a>
where
    RefState<'a>: From<(&'a I, &'a S)>,
    I: 'a,
    S: 'a,
{
    fn from(value: &'a Connection<I, S>) -> Self {
        let state = From::from((&value.initiation_type, &value.state));
        Self::new(&value.source_id, &value.pairwise_info, state)
    }
}

#[derive(Debug, Serialize)]
pub enum RefState<'a> {
    Inviter(RefInviterState<'a>),
    Invitee(RefInviteeState<'a>),
}

impl<'a, S> From<(&'a Inviter, &'a S)> for RefState<'a>
where
    RefInviterState<'a>: From<&'a S>,
    S: 'a,
{
    fn from(value: (&'a Inviter, &'a S)) -> Self {
        let (_, state) = value;
        let serde_state = From::from(state);
        Self::Inviter(serde_state)
    }
}

impl<'a, S> From<(&'a Invitee, &'a S)> for RefState<'a>
where
    RefInviteeState<'a>: From<&'a S>,
    S: 'a,
{
    fn from(value: (&'a Invitee, &'a S)) -> Self {
        let (_, state) = value;
        let serde_state = From::from(state);
        Self::Invitee(serde_state)
    }
}

#[derive(Debug, Serialize)]
pub enum RefInviterState<'a> {
    Initial(&'a InviterInitial),
    Invited(&'a InviterInvited),
    Requested(&'a InviterRequested),
    Responded(&'a RespondedState),
    Complete(&'a CompleteState),
}

impl<'a> From<&'a InviterInitial> for RefInviterState<'a> {
    fn from(value: &'a InviterInitial) -> Self {
        Self::Initial(value)
    }
}

impl<'a> From<&'a InviterInvited> for RefInviterState<'a> {
    fn from(value: &'a InviterInvited) -> Self {
        Self::Invited(value)
    }
}

impl<'a> From<&'a InviterRequested> for RefInviterState<'a> {
    fn from(value: &'a InviterRequested) -> Self {
        Self::Requested(value)
    }
}

impl<'a> From<&'a RespondedState> for RefInviterState<'a> {
    fn from(value: &'a RespondedState) -> Self {
        Self::Responded(value)
    }
}

impl<'a> From<&'a CompleteState> for RefInviterState<'a> {
    fn from(value: &'a CompleteState) -> Self {
        Self::Complete(value)
    }
}

#[derive(Debug, Serialize)]
pub enum RefInviteeState<'a> {
    Initial(&'a InviteeInitial),
    Invited(&'a InviteeInvited),
    Requested(&'a InviteeRequested),
    Responded(&'a RespondedState),
    Complete(&'a CompleteState),
}

impl<'a> From<&'a InviteeInitial> for RefInviteeState<'a> {
    fn from(value: &'a InviteeInitial) -> Self {
        Self::Initial(value)
    }
}

impl<'a> From<&'a InviteeInvited> for RefInviteeState<'a> {
    fn from(value: &'a InviteeInvited) -> Self {
        Self::Invited(value)
    }
}

impl<'a> From<&'a InviteeRequested> for RefInviteeState<'a> {
    fn from(value: &'a InviteeRequested) -> Self {
        Self::Requested(value)
    }
}

impl<'a> From<&'a RespondedState> for RefInviteeState<'a> {
    fn from(value: &'a RespondedState) -> Self {
        Self::Responded(value)
    }
}

impl<'a> From<&'a CompleteState> for RefInviteeState<'a> {
    fn from(value: &'a CompleteState) -> Self {
        Self::Complete(value)
    }
}

/// Compile-time assurance that the serialization type
/// of the [`Connection`], if modified, will be modified along the deserialization type.
#[cfg(test)]
mod tests {
    use crate::protocols::typestate_con::serde::de::*;

    use super::*;

    impl<'a> From<&'a VagueInviteeState> for RefInviteeState<'a> {
        fn from(value: &'a VagueInviteeState) -> Self {
            match value {
                VagueInviteeState::Initial(s) => Self::Initial(s),
                VagueInviteeState::Invited(s) => Self::Invited(s),
                VagueInviteeState::Requested(s) => Self::Requested(s),
                VagueInviteeState::Responded(s) => Self::Responded(s),
                VagueInviteeState::Complete(s) => Self::Complete(s),
            }
        }
    }

    impl<'a> From<&'a VagueInviterState> for RefInviterState<'a> {
        fn from(value: &'a VagueInviterState) -> Self {
            match value {
                VagueInviterState::Initial(s) => Self::Initial(s),
                VagueInviterState::Invited(s) => Self::Invited(s),
                VagueInviterState::Requested(s) => Self::Requested(s),
                VagueInviterState::Responded(s) => Self::Responded(s),
                VagueInviterState::Complete(s) => Self::Complete(s),
            }
        }
    }

    impl<'a> From<&'a VagueState> for RefState<'a> {
        fn from(value: &'a VagueState) -> Self {
            match value {
                VagueState::Invitee(s) => Self::Invitee(s.into()),
                VagueState::Inviter(s) => Self::Inviter(s.into()),
            }
        }
    }

    impl<'a> From<&'a VagueConnection> for SerializableConnection<'a> {
        fn from(value: &'a VagueConnection) -> Self {
            let VagueConnection {
                source_id,
                pairwise_info,
                state,
            } = value;

            Self {
                source_id,
                pairwise_info,
                state: state.into(),
            }
        }
    }
}
