mod conversions;
mod thin_state;

use aries_vcx_wallet::wallet::base_wallet::BaseWallet;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use messages::AriesMessage;

pub use self::thin_state::{State, ThinState};
use super::trait_bounds::BootstrapDidDoc;
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    handlers::util::AnyInvitation,
    protocols::connection::{
        invitee::states::{
            completed::Completed as InviteeCompleted, initial::Initial as InviteeInitial,
            invited::Invited as InviteeInvited, requested::Requested as InviteeRequested,
        },
        inviter::states::{
            completed::Completed as InviterCompleted, initial::Initial as InviterInitial,
            invited::Invited as InviterInvited, requested::Requested as InviterRequested,
        },
        pairwise_info::PairwiseInfo,
        trait_bounds::{TheirDidDoc, ThreadId},
    },
    transport::Transport,
    utils::encryption_envelope::EncryptionEnvelope,
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
    Completed(InviterCompleted),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InviteeState {
    Initial(InviteeInitial),
    Invited(InviteeInvited),
    Requested(InviteeRequested),
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
            GenericState::Invitee(InviteeState::Completed(s)) => Some(s.thread_id()),
            GenericState::Inviter(InviterState::Initial(_)) => None,
            GenericState::Inviter(InviterState::Invited(s)) => Some(s.thread_id()),
            GenericState::Inviter(InviterState::Requested(s)) => Some(s.thread_id()),
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
            GenericState::Invitee(InviteeState::Completed(s)) => Some(s.their_did_doc()),
            GenericState::Inviter(InviterState::Initial(_)) => None,
            GenericState::Inviter(InviterState::Invited(_)) => None,
            GenericState::Inviter(InviterState::Requested(s)) => Some(s.their_did_doc()),
            GenericState::Inviter(InviterState::Completed(s)) => Some(s.their_did_doc()),
        }
    }

    pub fn bootstrap_did_doc(&self) -> Option<&AriesDidDoc> {
        match &self.state {
            GenericState::Inviter(_) => None,
            GenericState::Invitee(InviteeState::Initial(_)) => None,
            GenericState::Invitee(InviteeState::Invited(s)) => Some(s.bootstrap_did_doc()),
            GenericState::Invitee(InviteeState::Requested(s)) => Some(s.bootstrap_did_doc()),
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

    pub fn invitation(&self) -> Option<&AnyInvitation> {
        match &self.state {
            GenericState::Inviter(InviterState::Invited(s)) => Some(&s.invitation),
            GenericState::Invitee(InviteeState::Invited(s)) => Some(&s.invitation),
            _ => None,
        }
    }

    pub async fn encrypt_message(
        &self,
        wallet: &impl BaseWallet,
        message: &AriesMessage,
    ) -> VcxResult<EncryptionEnvelope> {
        let sender_verkey = &self.pairwise_info().pw_vk;
        let did_doc = self.their_did_doc().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::NotReady,
            "No DidDoc present",
        ))?;
        EncryptionEnvelope::create_from_legacy(
            wallet,
            json!(message).to_string().as_bytes(),
            Some(sender_verkey),
            did_doc,
        )
        .await
    }

    pub async fn send_message<T>(
        &self,
        wallet: &impl BaseWallet,
        message: &AriesMessage,
        transport: &T,
    ) -> VcxResult<()>
    where
        T: Transport,
    {
        let did_doc = self.their_did_doc().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::NotReady,
            "No DidDoc present",
        ))?;

        let msg = self.encrypt_message(wallet, message).await?.0;
        let service_endpoint = did_doc.get_endpoint().ok_or_else(|| {
            AriesVcxError::from_msg(AriesVcxErrorKind::InvalidUrl, "No URL in DID Doc")
        })?;
        transport.send_message(msg, &service_endpoint).await
    }
}

/// Compile-time assurance that the [`GenericConnection`] and the hidden serialization type
/// of the [`crate::protocols::connection::Connection`], if modified, will be modified together.
#[cfg(test)]
mod connection_serde_tests {
    use async_trait::async_trait;
    use chrono::Utc;
    use messages::{
        decorators::{thread::Thread, timing::Timing},
        msg_fields::protocols::{
            connection::{
                invitation::{Invitation, InvitationContent},
                request::{Request, RequestContent, RequestDecorators},
                response::{Response, ResponseContent, ResponseDecorators},
                ConnectionData,
            },
            notification::ack::{Ack, AckContent, AckDecorators, AckStatus},
        },
    };
    use test_utils::{mock_wallet::MockWallet, mockdata::mock_ledger::MockLedger};
    use url::Url;
    use uuid::Uuid;

    use super::*;
    use crate::{
        common::signing::sign_connection_response,
        handlers::util::AnyInvitation,
        protocols::connection::{
            invitee::InviteeConnection, inviter::InviterConnection, serializable::*, Connection,
        },
    };

    impl<'a> From<RefInviteeState<'a>> for InviteeState {
        fn from(value: RefInviteeState<'a>) -> Self {
            match value {
                RefInviteeState::Initial(s) => Self::Initial(s.to_owned()),
                RefInviteeState::Invited(s) => Self::Invited(s.to_owned()),
                RefInviteeState::Requested(s) => Self::Requested(s.to_owned()),
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
        async fn send_message(&self, _msg: Vec<u8>, _service_endpoint: &Url) -> VcxResult<()> {
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

    async fn make_initial_parts() -> (String, PairwiseInfo) {
        let source_id = SOURCE_ID.to_owned();
        let wallet = MockWallet;
        let pairwise_info = PairwiseInfo::create(&wallet).await.unwrap();

        (source_id, pairwise_info)
    }

    async fn make_invitee_initial() -> InviteeConnection<InviteeInitial> {
        let (source_id, pairwise_info) = make_initial_parts().await;
        Connection::new_invitee(source_id, pairwise_info)
    }

    async fn make_invitee_invited() -> InviteeConnection<InviteeInvited> {
        let indy_ledger = MockLedger;
        let content = InvitationContent::builder_pairwise()
            .label(String::new())
            .recipient_keys(vec![PW_KEY.to_owned()])
            .service_endpoint(SERVICE_ENDPOINT.parse().unwrap())
            .build();

        let pw_invite = Invitation::builder()
            .id(Uuid::new_v4().to_string())
            .content(content)
            .build();

        let invitation = AnyInvitation::Con(pw_invite);

        make_invitee_initial()
            .await
            .accept_invitation(&indy_ledger, invitation)
            .await
            .unwrap()
    }

    async fn make_invitee_requested() -> InviteeConnection<InviteeRequested> {
        let service_endpoint = SERVICE_ENDPOINT.parse().unwrap();
        let routing_keys = vec![];

        make_invitee_invited()
            .await
            .prepare_request(service_endpoint, routing_keys)
            .await
            .unwrap()
    }

    async fn make_invitee_completed() -> InviteeConnection<InviteeCompleted> {
        let wallet = MockWallet;
        let con = make_invitee_requested().await;
        let mut con_data = ConnectionData::new(PW_KEY.to_owned(), AriesDidDoc::default());
        PW_KEY.clone_into(&mut con_data.did_doc.id);
        con_data.did_doc.set_recipient_keys(vec![PW_KEY.to_owned()]);
        con_data.did_doc.set_routing_keys(Vec::new());

        let sig_data = sign_connection_response(&wallet, PW_KEY, &con_data)
            .await
            .unwrap();

        let content = ResponseContent::builder().connection_sig(sig_data).build();
        let decorators = ResponseDecorators::builder()
            .thread(Thread::builder().thid(con.thread_id().to_owned()).build())
            .timing(Timing::builder().out_time(Utc::now()).build())
            .build();

        let response = Response::builder()
            .id(Uuid::new_v4().to_string())
            .content(content)
            .decorators(decorators)
            .build();

        let con = con.handle_response(&wallet, response).await.unwrap();

        con.send_message(&wallet, &con.get_ack().into(), &MockTransport)
            .await
            .unwrap();
        con
    }

    async fn make_inviter_initial() -> InviterConnection<InviterInitial> {
        let (source_id, pairwise_info) = make_initial_parts().await;
        Connection::new_inviter(source_id, pairwise_info)
    }

    async fn make_inviter_invited() -> InviterConnection<InviterInvited> {
        make_inviter_initial()
            .await
            .into_invited(&String::default())
    }

    async fn make_inviter_requested() -> InviterConnection<InviterRequested> {
        let wallet = MockWallet;
        let con = make_inviter_invited().await;
        let new_service_endpoint = SERVICE_ENDPOINT
            .to_owned()
            .parse()
            .expect("url should be valid");
        let new_routing_keys = vec![];

        let mut con_data = ConnectionData::new(PW_KEY.to_owned(), AriesDidDoc::default());
        PW_KEY.clone_into(&mut con_data.did_doc.id);
        con_data.did_doc.set_recipient_keys(vec![PW_KEY.to_owned()]);
        con_data.did_doc.set_routing_keys(Vec::new());

        let content = RequestContent::builder()
            .label(PW_KEY.to_owned())
            .connection(con_data)
            .build();

        let decorators = RequestDecorators::builder()
            .thread(Thread::builder().thid(con.thread_id().to_owned()).build())
            .timing(Timing::builder().out_time(Utc::now()).build())
            .build();

        let request = Request::builder()
            .id(Uuid::new_v4().to_string())
            .content(content)
            .decorators(decorators)
            .build();

        con.handle_request(&wallet, request, new_service_endpoint, new_routing_keys)
            .await
            .unwrap()
    }

    async fn make_inviter_completed() -> InviterConnection<InviterCompleted> {
        let con = make_inviter_requested().await;

        let content = AckContent::builder().status(AckStatus::Ok).build();
        let decorators = AckDecorators::builder()
            .thread(Thread::builder().thid(con.thread_id().to_owned()).build())
            .build();

        let msg = Ack::builder()
            .id(Uuid::new_v4().to_string())
            .content(content)
            .decorators(decorators)
            .build();

        con.acknowledge_connection(&msg).unwrap()
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
    generate_test!(invitee_connection_complete, make_invitee_completed);

    generate_test!(inviter_connection_initial, make_inviter_initial);
    generate_test!(inviter_connection_invited, make_inviter_invited);
    generate_test!(inviter_connection_requested, make_inviter_requested);
    generate_test!(inviter_connection_complete, make_inviter_completed);
}
