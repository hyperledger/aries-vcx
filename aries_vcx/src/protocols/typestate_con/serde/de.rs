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

/// Type used for deserialization of a [`Connection`].
/// This struct cannot be used for anything useful directly,
/// but the inner `state` has to be matched to get the concrete [`Connection`] type.
#[derive(Debug, Deserialize)]
pub struct VagueConnection {
    source_id: String,
    pairwise_info: PairwiseInfo,
    pub state: State,
}

impl VagueConnection {
    fn new(source_id: String, pairwise_info: PairwiseInfo, state: State) -> Self {
        Self {
            source_id,
            pairwise_info,
            state,
        }
    }
}

impl<I, S> From<Connection<I, S>> for VagueConnection
where
    State: From<(I, S)>,
{
    fn from(value: Connection<I, S>) -> Self {
        let state = From::from((value.initiation_type, value.state));
        Self::new(value.source_id, value.pairwise_info, state)
    }
}

#[derive(Debug, Deserialize)]
pub enum State {
    Inviter(InviterState),
    Invitee(InviteeState),
}

impl<S> From<(Inviter, S)> for State
where
    InviterState: From<S>,
{
    fn from(value: (Inviter, S)) -> Self {
        let (_, state) = value;
        let serde_state = From::from(state);
        Self::Inviter(serde_state)
    }
}

impl<S> From<(Invitee, S)> for State
where
    InviteeState: From<S>,
{
    fn from(value: (Invitee, S)) -> Self {
        let (_, state) = value;
        let serde_state = From::from(state);
        Self::Invitee(serde_state)
    }
}

#[derive(Debug, Deserialize)]
pub enum InviterState {
    Initial(InviterInitial),
    Invited(InviterInvited),
    Requested(InviterRequested),
    Responded(RespondedState),
    Complete(CompleteState),
}

impl From<InviterInitial> for InviterState {
    fn from(value: InviterInitial) -> Self {
        Self::Initial(value)
    }
}

impl From<InviterInvited> for InviterState {
    fn from(value: InviterInvited) -> Self {
        Self::Invited(value)
    }
}

impl From<InviterRequested> for InviterState {
    fn from(value: InviterRequested) -> Self {
        Self::Requested(value)
    }
}

impl From<RespondedState> for InviterState {
    fn from(value: RespondedState) -> Self {
        Self::Responded(value)
    }
}

impl From<CompleteState> for InviterState {
    fn from(value: CompleteState) -> Self {
        Self::Complete(value)
    }
}

#[derive(Debug, Deserialize)]
pub enum InviteeState {
    Initial(InviteeInitial),
    Invited(InviteeInvited),
    Requested(InviteeRequested),
    Responded(RespondedState),
    Complete(CompleteState),
}

impl From<InviteeInitial> for InviteeState {
    fn from(value: InviteeInitial) -> Self {
        Self::Initial(value)
    }
}

impl From<InviteeInvited> for InviteeState {
    fn from(value: InviteeInvited) -> Self {
        Self::Invited(value)
    }
}

impl From<InviteeRequested> for InviteeState {
    fn from(value: InviteeRequested) -> Self {
        Self::Requested(value)
    }
}

impl From<RespondedState> for InviteeState {
    fn from(value: RespondedState) -> Self {
        Self::Responded(value)
    }
}

impl From<CompleteState> for InviteeState {
    fn from(value: CompleteState) -> Self {
        Self::Complete(value)
    }
}
