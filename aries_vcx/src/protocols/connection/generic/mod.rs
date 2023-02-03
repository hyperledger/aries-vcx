mod conversions;
mod thin_state;

use std::sync::Arc;

use messages::{a2a::A2AMessage, diddoc::aries::diddoc::AriesDidDoc, protocols::connection::invite::Invitation};

pub use self::thin_state::{State, ThinState};

use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    plugins::wallet::base_wallet::BaseWallet,
    protocols::connection::{
        common::states::{complete::Complete, responded::Responded},
        invitee::states::{invited::Invited as InviteeInvited, requested::Requested as InviteeRequested},
        inviter::states::{invited::Invited as InviterInvited, requested::Requested as InviterRequested},
        pairwise_info::PairwiseInfo,
        trait_bounds::{TheirDidDoc, ThreadId},
    },
    transport::Transport,
};

use super::{basic_send_message, common::states::initial::Initial};

/// A type that can encapsulate a [`Connection`] of any state.
/// While mainly used for deserialization, it exposes some methods for retrieving
/// connection information.
///
/// However, using methods directly from [`Connection`], if possible, comes with certain
/// benefits such as being able to obtain an [`AriesDidDoc`] directly (if the state contains it)
/// and not an [`Option<AriesDidDoc>`] (which is what [`GenericConnection`] provides).
///
/// [`GenericConnection`] implements [`From`] for all [`Connection`] states and
/// [`Connection`] implements [`TryFrom`] from [`GenericConnection`], with the conversion failing
/// if the [`GenericConnection`] is in a different state than the requested one.
/// This is also the mechanism used for direct deserialization of a [`Connection`].
///
/// Because a [`TryFrom`] conversion is fallible and consumes the [`GenericConnection`], a thin [`State`]
/// can be retrieved through [`GenericConnection::state`] method at runtime. In that case, a more dynamic conversion
/// could be done this way:
///
/// ``` ignore
/// // Assume the `con` variable stores a `GenericConnection`:
///
/// let initial_connections = Vec::new();
/// let completed_connections = Vec::new();
///
/// // Unwrapping after the match is sound
/// // because we can guarantee the conversion will work
/// match con.state() {
///     State::Invitee(Stage::Initial) => initial_connections.push(con.try_into().unwrap()),
///     State::Invitee(Stage::Complete) => completed_connections.push(con.try_into().unwrap())
/// }
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenericConnection {
    pub(super) source_id: String,
    pub(super) pairwise_info: PairwiseInfo,
    pub(super) state: GenericState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GenericState {
    Inviter(InviterState),
    Invitee(InviteeState),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InviterState {
    Initial(Initial),
    Invited(InviterInvited),
    Requested(InviterRequested),
    Responded(Responded),
    Complete(Complete),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InviteeState {
    Initial(Initial),
    Invited(InviteeInvited),
    Requested(InviteeRequested),
    Responded(Responded),
    Complete(Complete),
}

impl GenericConnection {
    /// Returns the underlying [`super::Connection`]'s state as a thin [`State`].
    /// Used for pattern matching when there's no hint as to what connection type
    /// is expected from or stored into the [`GenericConnection`].
    pub fn state(&self) -> State {
        (&self.state).into()
    }

    pub fn thread_id(&self) -> Option<&str> {
        match &self.state {
            GenericState::Invitee(InviteeState::Initial(_)) => None,
            GenericState::Invitee(InviteeState::Invited(s)) => Some(s.thread_id()),
            GenericState::Invitee(InviteeState::Requested(s)) => Some(s.thread_id()),
            GenericState::Invitee(InviteeState::Responded(s)) => Some(s.thread_id()),
            GenericState::Invitee(InviteeState::Complete(s)) => Some(s.thread_id()),
            GenericState::Inviter(InviterState::Initial(_)) => None,
            GenericState::Inviter(InviterState::Invited(s)) => Some(s.thread_id()),
            GenericState::Inviter(InviterState::Requested(s)) => Some(s.thread_id()),
            GenericState::Inviter(InviterState::Responded(s)) => Some(s.thread_id()),
            GenericState::Inviter(InviterState::Complete(s)) => Some(s.thread_id()),
        }
    }

    pub fn pairwise_info(&self) -> &PairwiseInfo {
        &self.pairwise_info
    }

    pub fn their_did_doc(&self) -> Option<&AriesDidDoc> {
        match &self.state {
            GenericState::Invitee(InviteeState::Initial(_)) => None,
            GenericState::Invitee(InviteeState::Invited(s)) => Some(s.their_did_doc()),
            GenericState::Invitee(InviteeState::Requested(s)) => Some(s.their_did_doc()),
            GenericState::Invitee(InviteeState::Responded(s)) => Some(s.their_did_doc()),
            GenericState::Invitee(InviteeState::Complete(s)) => Some(s.their_did_doc()),
            GenericState::Inviter(InviterState::Initial(_)) => None,
            GenericState::Inviter(InviterState::Invited(_)) => None,
            GenericState::Inviter(InviterState::Requested(s)) => Some(s.their_did_doc()),
            GenericState::Inviter(InviterState::Responded(s)) => Some(s.their_did_doc()),
            GenericState::Inviter(InviterState::Complete(s)) => Some(s.their_did_doc()),
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
            GenericState::Inviter(InviterState::Invited(s)) => Some(&s.invitation),
            GenericState::Invitee(InviteeState::Invited(s)) => Some(&s.invitation),
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

        basic_send_message(wallet, message, sender_verkey, did_doc, transport).await
    }
}

/// Compile-time assurance that the deserialization type
/// of the [`Connection`], if modified, will be modified along the serialization type.
#[cfg(test)]
mod tests {
    use crate::protocols::connection::serializable::*;

    use super::*;

    impl<'a> From<RefInviteeState<'a>> for InviteeState {
        fn from(value: RefInviteeState<'a>) -> Self {
            match value {
                RefInviteeState::Initial(s) => Self::Initial(s.to_owned()),
                RefInviteeState::Invited(s) => Self::Invited(s.to_owned()),
                RefInviteeState::Requested(s) => Self::Requested(s.to_owned()),
                RefInviteeState::Responded(s) => Self::Responded(s.to_owned()),
                RefInviteeState::Complete(s) => Self::Complete(s.to_owned()),
            }
        }
    }

    impl<'a> From<RefInviterState<'a>> for InviterState {
        fn from(value: RefInviterState<'a>) -> Self {
            match value {
                RefInviterState::Initial(s) => Self::Initial(s.to_owned()),
                RefInviterState::Invited(s) => Self::Invited(s.to_owned()),
                RefInviterState::Requested(s) => Self::Requested(s.to_owned()),
                RefInviterState::Responded(s) => Self::Responded(s.to_owned()),
                RefInviterState::Complete(s) => Self::Complete(s.to_owned()),
            }
        }
    }

    impl<'a> From<RefState<'a>> for GenericState {
        fn from(value: RefState<'a>) -> Self {
            match value {
                RefState::Invitee(s) => Self::Invitee(s.into()),
                RefState::Inviter(s) => Self::Inviter(s.into()),
            }
        }
    }

    impl<'a> From<SerializableConnection<'a>> for GenericConnection {
        fn from(value: SerializableConnection<'a>) -> Self {
            let SerializableConnection {
                source_id,
                pairwise_info,
                state,
            } = value;

            Self {
                source_id: source_id.to_owned(),
                pairwise_info: pairwise_info.to_owned(),
                state: state.into(),
            }
        }
    }

    impl<'a> From<&'a InviteeState> for RefInviteeState<'a> {
        fn from(value: &'a InviteeState) -> Self {
            match value {
                InviteeState::Initial(s) => Self::Initial(s),
                InviteeState::Invited(s) => Self::Invited(s),
                InviteeState::Requested(s) => Self::Requested(s),
                InviteeState::Responded(s) => Self::Responded(s),
                InviteeState::Complete(s) => Self::Complete(s),
            }
        }
    }

    impl<'a> From<&'a InviterState> for RefInviterState<'a> {
        fn from(value: &'a InviterState) -> Self {
            match value {
                InviterState::Initial(s) => Self::Initial(s),
                InviterState::Invited(s) => Self::Invited(s),
                InviterState::Requested(s) => Self::Requested(s),
                InviterState::Responded(s) => Self::Responded(s),
                InviterState::Complete(s) => Self::Complete(s),
            }
        }
    }

    impl<'a> From<&'a GenericState> for RefState<'a> {
        fn from(value: &'a GenericState) -> Self {
            match value {
                GenericState::Invitee(s) => Self::Invitee(s.into()),
                GenericState::Inviter(s) => Self::Inviter(s.into()),
            }
        }
    }

    impl<'a> From<&'a GenericConnection> for SerializableConnection<'a> {
        fn from(value: &'a GenericConnection) -> Self {
            let GenericConnection {
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
