mod conversions;
mod thin_state;

use std::sync::Arc;

use messages::{a2a::A2AMessage, diddoc::aries::diddoc::AriesDidDoc, protocols::connection::invite::Invitation};

pub use self::thin_state::{State, ThinState};
use super::{trait_bounds::BootstrapDidDoc, wrap_and_send_msg};
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    plugins::wallet::base_wallet::BaseWallet,
    protocols::connection::{
        invitee::states::{
            completed::Completed as InviteeCompleted, initial::Initial as InviteeInitial,
            invited::Invited as InviteeInvited, requested::Requested as InviteeRequested,
            responded::Responded as InviteeResponded,
        },
        inviter::states::{
            completed::Completed as InviterCompleted, initial::Initial as InviterInitial,
            invited::Invited as InviterInvited, requested::Requested as InviterRequested,
            responded::Responded as InviterResponded,
        },
        pairwise_info::PairwiseInfo,
        trait_bounds::{TheirDidDoc, ThreadId},
    },
    transport::Transport,
};

/// A type that can encapsulate a [`super::Connection`] of any state.
/// While mainly used for deserialization, it exposes some methods for retrieving
/// connection information.
///
/// However, using methods directly from [`super::Connection`], if possible, comes with certain
/// benefits such as being able to obtain an [`AriesDidDoc`] directly (if the state contains it)
/// and not an [`Option<AriesDidDoc>`] (which is what [`GenericConnection`] provides).
///
/// [`GenericConnection`] implements [`From`] for all [`super::Connection`] states and
/// [`super::Connection`] implements [`TryFrom`] from [`GenericConnection`], with the conversion
/// failing if the [`GenericConnection`] is in a different state than the requested one.
/// This is also the mechanism used for direct deserialization of a [`super::Connection`].
///
/// Because a [`TryFrom`] conversion is fallible and consumes the [`GenericConnection`], a
/// [`ThinState`] can be retrieved through [`GenericConnection::state`] method at runtime. In that
/// case, a more dynamic conversion could be done this way:
///
/// ```
/// # use aries_vcx::protocols::connection::invitee::states::{complete::Complete, initial::Initial};
/// # use aries_vcx::protocols::connection::initiation_type::Invitee;
/// # use aries_vcx::protocols::mediated_connection::pairwise_info::PairwiseInfo;
/// # use aries_vcx::protocols::connection::{GenericConnection, ThinState, State, Connection};
/// #
/// # let con_inviter = Connection::new_invitee(String::new(), PairwiseInfo::default());
///
/// // We get a GenericConnection somehow
/// let con: GenericConnection = con_inviter.into();
///
/// let mut initial_connections: Vec<Connection<Invitee, Initial>> = Vec::new();
/// let mut completed_connections: Vec<Connection<Invitee, Complete>> = Vec::new();
///
/// // Unwrapping after the match is sound
/// // because we can guarantee the conversion will work
/// match con.state() {
///     ThinState::Invitee(State::Initial) => initial_connections.push(con.try_into().unwrap()),
///     ThinState::Invitee(State::Complete) => completed_connections.push(con.try_into().unwrap()),
///     _ => todo!()
/// }
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenericConnection {
    source_id: String,
    pairwise_info: PairwiseInfo,
    state: GenericState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GenericState {
    Inviter(InviterState),
    Invitee(InviteeState),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InviterState {
    Initial(InviterInitial),
    Invited(InviterInvited),
    Requested(InviterRequested),
    Responded(InviterResponded),
    Completed(InviterCompleted),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InviteeState {
    Initial(InviteeInitial),
    Invited(InviteeInvited),
    Requested(InviteeRequested),
    Responded(InviteeResponded),
    Completed(InviteeCompleted),
}

impl GenericConnection {
    /// Returns the underlying [`super::Connection`]'s state as a [`ThinState`].
    /// Used for pattern matching when there's no hint as to what connection type
    /// is expected from or stored into the [`GenericConnection`].
    pub fn state(&self) -> ThinState {
        (&self.state).into()
    }

    pub fn thread_id(&self) -> Option<&str> {
        match &self.state {
            GenericState::Invitee(InviteeState::Initial(_)) => None,
            GenericState::Invitee(InviteeState::Invited(s)) => Some(s.thread_id()),
            GenericState::Invitee(InviteeState::Requested(s)) => Some(s.thread_id()),
            GenericState::Invitee(InviteeState::Responded(s)) => Some(s.thread_id()),
            GenericState::Invitee(InviteeState::Completed(s)) => Some(s.thread_id()),
            GenericState::Inviter(InviterState::Initial(_)) => None,
            GenericState::Inviter(InviterState::Invited(s)) => Some(s.thread_id()),
            GenericState::Inviter(InviterState::Requested(s)) => Some(s.thread_id()),
            GenericState::Inviter(InviterState::Responded(s)) => Some(s.thread_id()),
            GenericState::Inviter(InviterState::Completed(s)) => Some(s.thread_id()),
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
            GenericState::Invitee(InviteeState::Completed(s)) => Some(s.their_did_doc()),
            GenericState::Inviter(InviterState::Initial(_)) => None,
            GenericState::Inviter(InviterState::Invited(_)) => None,
            GenericState::Inviter(InviterState::Requested(s)) => Some(s.their_did_doc()),
            GenericState::Inviter(InviterState::Responded(s)) => Some(s.their_did_doc()),
            GenericState::Inviter(InviterState::Completed(s)) => Some(s.their_did_doc()),
        }
    }

    pub fn bootstrap_did_doc(&self) -> Option<&AriesDidDoc> {
        match &self.state {
            GenericState::Inviter(_) => None,
            GenericState::Invitee(InviteeState::Initial(_)) => None,
            GenericState::Invitee(InviteeState::Invited(s)) => Some(s.bootstrap_did_doc()),
            GenericState::Invitee(InviteeState::Requested(s)) => Some(s.bootstrap_did_doc()),
            GenericState::Invitee(InviteeState::Responded(s)) => Some(s.bootstrap_did_doc()),
            GenericState::Invitee(InviteeState::Completed(s)) => Some(s.bootstrap_did_doc()),
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

        wrap_and_send_msg(wallet, message, sender_verkey, did_doc, transport).await
    }
}

/// Compile-time assurance that the [`GenericConnection`] and the hidden serialization type
/// of the [`crate::protocols::connection::Connection`], if modified, will be modified together.
#[cfg(test)]
mod connection_serde_tests {
    #![allow(clippy::unwrap_used)]

    use std::sync::Arc;

    use async_trait::async_trait;
    use messages::protocols::connection::{invite::PairwiseInvitation, request::Request, response::Response};

    use super::*;
    use crate::{
        common::signing::sign_connection_response,
        core::profile::profile::Profile,
        protocols::connection::{invitee::InviteeConnection, inviter::InviterConnection, serializable::*, Connection},
        utils::mockdata::profile::mock_profile::MockProfile,
    };

    impl<'a> From<RefInviteeState<'a>> for InviteeState {
        fn from(value: RefInviteeState<'a>) -> Self {
            match value {
                RefInviteeState::Initial(s) => Self::Initial(s.to_owned()),
                RefInviteeState::Invited(s) => Self::Invited(s.to_owned()),
                RefInviteeState::Requested(s) => Self::Requested(s.to_owned()),
                RefInviteeState::Responded(s) => Self::Responded(s.to_owned()),
                RefInviteeState::Completed(s) => Self::Completed(s.to_owned()),
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
                RefInviterState::Completed(s) => Self::Completed(s.to_owned()),
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
                InviteeState::Completed(s) => Self::Completed(s),
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
                InviterState::Completed(s) => Self::Completed(s),
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

    struct MockTransport;

    #[async_trait]
    impl Transport for MockTransport {
        async fn send_message(&self, _msg: Vec<u8>, _service_endpoint: &str) -> VcxResult<()> {
            Ok(())
        }
    }

    fn serde_test<I, S>(con: Connection<I, S>)
    where
        I: Clone,
        S: Clone,
        for<'a> SerializableConnection<'a>: From<&'a Connection<I, S>>,
        GenericConnection: From<Connection<I, S>>,
        Connection<I, S>: TryFrom<GenericConnection, Error = AriesVcxError>,
        (I, S): TryFrom<GenericState, Error = AriesVcxError>,
    {
        // Clone and convert to generic
        let gen_con = GenericConnection::from(con.clone());

        // Serialize concrete and generic connections, then compare.
        let con_string = serde_json::to_string(&con).unwrap();
        let gen_con_string = serde_json::to_string(&gen_con).unwrap();
        assert_eq!(con_string, gen_con_string);

        // Deliberately reversing the strings that were serialized.
        // The states are identical, so the cross-deserialization should work.
        let con: Connection<I, S> = serde_json::from_str(&gen_con_string).unwrap();
        let gen_con: GenericConnection = serde_json::from_str(&con_string).unwrap();

        // Serialize and compare again.
        let con_string = serde_json::to_string(&con).unwrap();
        let gen_con_string = serde_json::to_string(&gen_con).unwrap();
        assert_eq!(con_string, gen_con_string);
    }

    const SOURCE_ID: &str = "connection_serde_tests";
    const PW_KEY: &str = "7Z9ZajGKvb6BMsZ9TBEqxMHktxGdts3FvAbKSJT5XgzK";
    const SERVICE_ENDPOINT: &str = "https://localhost:8080";

    fn make_mock_profile() -> Arc<dyn Profile> {
        Arc::new(MockProfile)
    }

    async fn make_initial_parts() -> (String, PairwiseInfo) {
        let source_id = SOURCE_ID.to_owned();
        let pairwise_info = PairwiseInfo::create(&make_mock_profile().inject_wallet())
            .await
            .unwrap();

        (source_id, pairwise_info)
    }

    async fn make_invitee_initial() -> InviteeConnection<InviteeInitial> {
        let (source_id, pairwise_info) = make_initial_parts().await;
        Connection::new_invitee(source_id, pairwise_info)
    }

    async fn make_invitee_invited() -> InviteeConnection<InviteeInvited> {
        let profile = make_mock_profile();
        let pw_invite = PairwiseInvitation::default().set_recipient_keys(vec![PW_KEY.to_owned()]);
        let invitation = Invitation::Pairwise(pw_invite);

        make_invitee_initial()
            .await
            .accept_invitation(&profile, invitation)
            .await
            .unwrap()
    }

    async fn make_invitee_requested() -> InviteeConnection<InviteeRequested> {
        let wallet = make_mock_profile().inject_wallet();
        let service_endpoint = SERVICE_ENDPOINT.to_owned();
        let routing_keys = vec![];

        make_invitee_invited()
            .await
            .send_request(&wallet, service_endpoint, routing_keys, &MockTransport)
            .await
            .unwrap()
    }

    async fn make_invitee_responded() -> InviteeConnection<InviteeResponded> {
        let wallet = make_mock_profile().inject_wallet();
        let con = make_invitee_requested().await;
        let response = Response::create()
            .set_keys(vec![PW_KEY.to_owned()], vec![])
            .ask_for_ack()
            .set_thread_id(con.thread_id())
            .set_out_time();

        let response = sign_connection_response(&wallet, PW_KEY, response).await.unwrap();

        con.handle_response(&wallet, response, &MockTransport).await.unwrap()
    }

    async fn make_invitee_completed() -> InviteeConnection<InviteeCompleted> {
        let wallet = make_mock_profile().inject_wallet();

        make_invitee_responded()
            .await
            .send_ack(&wallet, &MockTransport)
            .await
            .unwrap()
    }

    async fn make_inviter_initial() -> InviterConnection<InviterInitial> {
        let (source_id, pairwise_info) = make_initial_parts().await;
        Connection::new_inviter(source_id, pairwise_info)
    }

    async fn make_inviter_invited() -> InviterConnection<InviterInvited> {
        make_inviter_initial().await.into_invited(&String::default())
    }

    async fn make_inviter_requested() -> InviterConnection<InviterRequested> {
        let wallet = make_mock_profile().inject_wallet();
        let con = make_inviter_invited().await;
        let new_service_endpoint = SERVICE_ENDPOINT.to_owned();
        let new_routing_keys = vec![];
        let request = Request::create()
            .set_service_endpoint(new_service_endpoint.clone())
            .set_label(SOURCE_ID.to_owned())
            .set_did(PW_KEY.to_owned())
            .set_keys(vec![PW_KEY.to_owned()], vec![])
            .set_thread_id(con.thread_id())
            .set_out_time();

        con.handle_request(&wallet, request, new_service_endpoint, new_routing_keys, &MockTransport)
            .await
            .unwrap()
    }

    async fn make_inviter_responded() -> InviterConnection<InviterResponded> {
        let wallet = make_mock_profile().inject_wallet();

        make_inviter_requested()
            .await
            .send_response(&wallet, &MockTransport)
            .await
            .unwrap()
    }

    async fn make_inviter_completed() -> InviterConnection<InviterCompleted> {
        let msg = Request::create().to_a2a_message();

        make_inviter_responded().await.acknowledge_connection(&msg).unwrap()
    }

    macro_rules! generate_test {
        ($name:ident, $func:ident) => {
            #[tokio::test]
            async fn $name() {
                let con = $func().await;
                serde_test(con);
            }
        };
    }

    generate_test!(invitee_connection_initial, make_invitee_initial);
    generate_test!(invitee_connection_invited, make_invitee_invited);
    generate_test!(invitee_connection_requested, make_invitee_requested);
    generate_test!(invitee_connection_responded, make_invitee_responded);
    generate_test!(invitee_connection_complete, make_invitee_completed);

    generate_test!(inviter_connection_initial, make_inviter_initial);
    generate_test!(inviter_connection_invited, make_inviter_invited);
    generate_test!(inviter_connection_requested, make_inviter_requested);
    generate_test!(inviter_connection_responded, make_inviter_responded);
    generate_test!(inviter_connection_complete, make_inviter_completed);
}
