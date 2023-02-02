use serde::Serialize;

use crate::protocols::connection::{
    common::states::{complete::Complete, responded::Responded},
    initiation_type::{Invitee, Inviter},
    invitee::states::{invited::Invited as InviteeInvited, requested::Requested as InviteeRequested},
    inviter::states::{invited::Invited as InviterInvited, requested::Requested as InviterRequested},
    pairwise_info::PairwiseInfo,
    Connection,
};

use super::common::states::initial::Initial;

/// Macro used for boilerplace implementation of the
/// [`From`] trait from a concrete connection state to the equivalent reference state
/// used for serialization.
macro_rules! from_concrete_to_serializable {
    ($from:ident, $var:ident, $to:ident) => {
        impl<'a> From<&'a $from> for $to<'a> {
            fn from(value: &'a $from) -> Self {
                Self::$var(value)
            }
        }
    };

    ($init_type:ident, $state:ident, $var:ident, $to:ident) => {
        impl<'a, S> From<(&'a $init_type, &'a S)> for $to<'a>
        where
            $state<'a>: From<&'a S>,
            S: 'a,
        {
            fn from(value: (&'a $init_type, &'a S)) -> Self {
                let (_, state) = value;
                let serde_state = From::from(state);
                Self::$var(serde_state)
            }
        }
    };
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

#[derive(Debug, Serialize)]
pub enum RefState<'a> {
    Inviter(RefInviterState<'a>),
    Invitee(RefInviteeState<'a>),
}

#[derive(Debug, Serialize)]
pub enum RefInviterState<'a> {
    Initial(&'a Initial),
    Invited(&'a InviterInvited),
    Requested(&'a InviterRequested),
    Responded(&'a Responded),
    Complete(&'a Complete),
}

#[derive(Debug, Serialize)]
pub enum RefInviteeState<'a> {
    Initial(&'a Initial),
    Invited(&'a InviteeInvited),
    Requested(&'a InviteeRequested),
    Responded(&'a Responded),
    Complete(&'a Complete),
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

from_concrete_to_serializable!(Inviter, RefInviterState, Inviter, RefState);
from_concrete_to_serializable!(Invitee, RefInviteeState, Invitee, RefState);

from_concrete_to_serializable!(Initial, Initial, RefInviterState);
from_concrete_to_serializable!(InviterInvited, Invited, RefInviterState);
from_concrete_to_serializable!(InviterRequested, Requested, RefInviterState);
from_concrete_to_serializable!(Responded, Responded, RefInviterState);
from_concrete_to_serializable!(Complete, Complete, RefInviterState);

from_concrete_to_serializable!(Initial, Initial, RefInviteeState);
from_concrete_to_serializable!(InviteeInvited, Invited, RefInviteeState);
from_concrete_to_serializable!(InviteeRequested, Requested, RefInviteeState);
from_concrete_to_serializable!(Responded, Responded, RefInviteeState);
from_concrete_to_serializable!(Complete, Complete, RefInviteeState);

impl<'a> SerializableConnection<'a> {
    fn new(source_id: &'a str, pairwise_info: &'a PairwiseInfo, state: RefState<'a>) -> Self {
        Self {
            source_id,
            pairwise_info,
            state,
        }
    }
}

/// Manual implementation of [`Serialize`] for [`Connection`],
/// as we'll first convert into [`SerializableConnection`] for a [`Connection`] reference
/// and serialize that.
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
