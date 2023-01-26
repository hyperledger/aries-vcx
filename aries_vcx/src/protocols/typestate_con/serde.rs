use super::{
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SerdeCon {
    source_id: String,
    pairwise_info: PairwiseInfo,
    state: SerdeState,
}

impl SerdeCon {
    fn new(source_id: String, pairwise_info: PairwiseInfo, state: SerdeState) -> Self {
        Self {
            source_id,
            pairwise_info,
            state,
        }
    }
}

impl<I, S> From<Connection<I, S>> for SerdeCon
where
    SerdeState: From<(I, S)>,
{
    fn from(value: Connection<I, S>) -> Self {
        let state = From::from((value.initiation_type, value.state));
        Self::new(value.source_id, value.pairwise_info, state)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SerdeState {
    Inviter(SerdeInviterState),
    Invitee(SerdeInviteeState),
}

impl<S> From<(Inviter, S)> for SerdeState
where
    SerdeInviterState: From<S>,
{
    fn from(value: (Inviter, S)) -> Self {
        let (_, state) = value;
        let serde_state = From::from(state);
        Self::Inviter(serde_state)
    }
}

impl<S> From<(Invitee, S)> for SerdeState
where
    SerdeInviteeState: From<S>,
{
    fn from(value: (Invitee, S)) -> Self {
        let (_, state) = value;
        let serde_state = From::from(state);
        Self::Invitee(serde_state)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SerdeInviterState {
    Initial(InviterInitial),
    Invited(InviterInvited),
    Requested(InviterRequested),
    Responded(RespondedState),
    Complete(CompleteState),
}

impl From<InviterInitial> for SerdeInviterState {
    fn from(value: InviterInitial) -> Self {
        Self::Initial(value)
    }
}

impl From<InviterInvited> for SerdeInviterState {
    fn from(value: InviterInvited) -> Self {
        Self::Invited(value)
    }
}

impl From<InviterRequested> for SerdeInviterState {
    fn from(value: InviterRequested) -> Self {
        Self::Requested(value)
    }
}

impl From<RespondedState> for SerdeInviterState {
    fn from(value: RespondedState) -> Self {
        Self::Responded(value)
    }
}

impl From<CompleteState> for SerdeInviterState {
    fn from(value: CompleteState) -> Self {
        Self::Complete(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SerdeInviteeState {
    Initial(InviteeInitial),
    Invited(InviteeInvited),
    Requested(InviteeRequested),
    Responded(RespondedState),
    Complete(CompleteState),
}

impl From<InviteeInitial> for SerdeInviteeState {
    fn from(value: InviteeInitial) -> Self {
        Self::Initial(value)
    }
}

impl From<InviteeInvited> for SerdeInviteeState {
    fn from(value: InviteeInvited) -> Self {
        Self::Invited(value)
    }
}

impl From<InviteeRequested> for SerdeInviteeState {
    fn from(value: InviteeRequested) -> Self {
        Self::Requested(value)
    }
}

impl From<RespondedState> for SerdeInviteeState {
    fn from(value: RespondedState) -> Self {
        Self::Responded(value)
    }
}

impl From<CompleteState> for SerdeInviteeState {
    fn from(value: CompleteState) -> Self {
        Self::Complete(value)
    }
}
