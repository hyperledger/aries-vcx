use std::clone::Clone;
use std::collections::HashMap;
use std::future::Future;

use crate::error::prelude::*;
use crate::messages::a2a::A2AMessage;
use crate::messages::a2a::protocol_registry::ProtocolRegistry;
use crate::messages::ack::Ack;
use crate::messages::connection::did_doc::DidDoc;
use crate::messages::connection::invite::Invitation;
use crate::messages::connection::problem_report::{ProblemCode, ProblemReport};
use crate::messages::connection::request::Request;
use crate::messages::connection::response::{Response, SignedResponse};
use crate::messages::discovery::disclose::{Disclose, ProtocolDescriptor};
use crate::messages::discovery::query::Query;
use crate::messages::trust_ping::ping::Ping;
use crate::messages::trust_ping::ping_response::PingResponse;
use crate::messages::out_of_band::handshake_reuse::OutOfBandHandshakeReuse;
use crate::handlers::out_of_band::OutOfBandInvitation;
use crate::protocols::connection::invitee::states::complete::CompleteState;
use crate::protocols::connection::invitee::states::initial::InitialState;
use crate::protocols::connection::invitee::states::invited::InvitedState;
use crate::protocols::connection::invitee::states::requested::RequestedState;
use crate::protocols::connection::invitee::states::responded::RespondedState;
use crate::protocols::connection::pairwise_info::PairwiseInfo;
use crate::protocols::connection::util::verify_thread_id;

#[derive(Clone)]
pub struct SmConnectionInvitee {
    source_id: String,
    thread_id: String,
    pairwise_info: PairwiseInfo,
    state: InviteeFullState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InviteeFullState {
    Initial(InitialState),
    Invited(InvitedState),
    Requested(RequestedState),
    Responded(RespondedState),
    Completed(CompleteState),
}

#[derive(Debug, PartialEq)]
pub enum InviteeState {
    Initial,
    Invited,
    Requested,
    Responded,
    Completed,
}

impl PartialEq for SmConnectionInvitee {
    fn eq(&self, other: &Self) -> bool {
        self.source_id == other.source_id &&
            self.pairwise_info == other.pairwise_info &&
            self.state == other.state
    }
}

impl From<InviteeFullState> for InviteeState {
    fn from(state: InviteeFullState) -> InviteeState {
        match state {
            InviteeFullState::Initial(_) => InviteeState::Initial,
            InviteeFullState::Invited(_) => InviteeState::Invited,
            InviteeFullState::Requested(_) => InviteeState::Requested,
            InviteeFullState::Responded(_) => InviteeState::Responded,
            InviteeFullState::Completed(_) => InviteeState::Completed
        }
    }
}

impl SmConnectionInvitee {
    pub fn new(source_id: &str, pairwise_info: PairwiseInfo) -> Self {
        SmConnectionInvitee {
            source_id: source_id.to_string(),
            thread_id: String::new(),
            state: InviteeFullState::Initial(InitialState::new(None)),
            pairwise_info,
        }
    }

    pub fn is_in_null_state(&self) -> bool {
        return InviteeState::from(self.state.clone()) == InviteeState::Initial;
    }

    pub fn from(source_id: String, thread_id: String, pairwise_info: PairwiseInfo, state: InviteeFullState) -> Self {
        SmConnectionInvitee {
            source_id,
            thread_id,
            pairwise_info,
            state,
        }
    }

    pub fn pairwise_info(&self) -> &PairwiseInfo {
        &self.pairwise_info
    }

    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    pub fn get_state(&self) -> InviteeState {
        InviteeState::from(self.state.clone())
    }

    pub fn state_object(&self) -> &InviteeFullState {
        &self.state
    }

    pub fn needs_message(&self) -> bool {
        match self.state {
            InviteeFullState::Responded(_) => false,
            _ => true
        }
    }

    pub fn their_did_doc(&self) -> Option<DidDoc> {
        match self.state {
            InviteeFullState::Initial(_) => None,
            InviteeFullState::Invited(ref state) => Some(DidDoc::from(state.invitation.clone())),
            InviteeFullState::Requested(ref state) => Some(state.did_doc.clone()),
            InviteeFullState::Responded(ref state) => Some(state.did_doc.clone()),
            InviteeFullState::Completed(ref state) => Some(state.did_doc.clone()),
        }
    }

    pub fn bootstrap_did_doc(&self) -> Option<DidDoc> {
        match self.state {
            InviteeFullState::Initial(_) => None,
            InviteeFullState::Invited(ref state) => Some(DidDoc::from(state.invitation.clone())),
            InviteeFullState::Requested(ref state) => Some(state.did_doc.clone()),
            InviteeFullState::Responded(ref state) => Some(state.did_doc.clone()),
            InviteeFullState::Completed(ref state) => Some(state.bootstrap_did_doc.clone()),
        }
    }

    pub fn get_invitation(&self) -> Option<&Invitation> {
        match self.state {
            InviteeFullState::Invited(ref state) => Some(&state.invitation),
            _ => None
        }
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        for (uid, message) in messages {
            if self.can_handle_message(&message) {
                return Some((uid, message));
            }
        }
        None
    }

    pub fn get_protocols(&self) -> Vec<ProtocolDescriptor> {
        ProtocolRegistry::init().protocols()
    }

    pub fn get_remote_protocols(&self) -> Option<Vec<ProtocolDescriptor>> {
        match self.state {
            InviteeFullState::Completed(ref state) => state.protocols.clone(),
            _ => None
        }
    }

    pub fn remote_did(&self) -> VcxResult<String> {
        self.their_did_doc()
            .map(|did_doc: DidDoc| did_doc.id.clone())
            .ok_or(VcxError::from_msg(VcxErrorKind::NotReady, "Remote Connection DID is not set"))
    }

    pub fn remote_vk(&self) -> VcxResult<String> {
        self.their_did_doc()
            .and_then(|did_doc| did_doc.recipient_keys().get(0).cloned())
            .ok_or(VcxError::from_msg(VcxErrorKind::NotReady, "Remote Connection Verkey is not set"))
    }

    pub fn can_handle_message(&self, message: &A2AMessage) -> bool {
        match self.state {
            InviteeFullState::Requested(_) => {
                match message {
                    A2AMessage::ConnectionResponse(_) => {
                        debug!("Invitee received ConnectionResponse message");
                        true
                    }
                    A2AMessage::ConnectionProblemReport(_) => {
                        debug!("Invitee received ProblemReport message");
                        true
                    }
                    _ => {
                        debug!("Invitee received unexpected message: {:?}", message);
                        false
                    }
                }
            }
            InviteeFullState::Completed(_) => {
                match message {
                    A2AMessage::Ping(_) => {
                        debug!("Ping message received");
                        true
                    }
                    A2AMessage::PingResponse(_) => {
                        debug!("PingResponse message received");
                        true
                    }
                    A2AMessage::Query(_) => {
                        debug!("Query message received");
                        true
                    }
                    A2AMessage::Disclose(_) => {
                        debug!("Disclose message received");
                        true
                    }
                    A2AMessage::OutOfBandHandshakeReuse(_) => {
                        debug!("OutOfBandHandshakeReuse message received");
                        true
                    }
                    A2AMessage::OutOfBandHandshakeReuseAccepted(_) => {
                        debug!("OutOfBandHandshakeReuseAccepted message received");
                        true
                    }
                    _ => {
                        debug!("Unexpected message received in Completed state: {:?}", message);
                        false
                    }
                }
            }
            _ => {
                debug!("Unexpected message received: message: {:?}", message);
                false
            }
        }
    }

    async fn _send_ack<F, T>(&self,
                             did_doc: &DidDoc,
                             request: &Request,
                             response: &SignedResponse,
                             pairwise_info: &PairwiseInfo,
                             send_message: F) -> VcxResult<Response>
        where
            F: Fn(String, DidDoc, A2AMessage) -> T,
            T: Future<Output=VcxResult<()>>

    {
        let remote_vk: String = did_doc.recipient_keys().get(0).cloned()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "Cannot handle response: remote verkey not found"))?;

        let response = response.clone().decode(&remote_vk).await?;

        if !response.from_thread(&request.get_thread_id()) {
            return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot handle response: thread id does not match: {:?}", response.thread)));
        }

        let message = Ack::create()
            .set_thread_id(&self.thread_id)
            .to_a2a_message();

        send_message(pairwise_info.pw_vk.clone(), response.connection.did_doc.clone(), message).await?;
        Ok(response)
    }

    pub fn handle_invitation(self, invitation: Invitation) -> VcxResult<Self> {
        let Self { state, .. } = self;
        let state = match state {
            InviteeFullState::Initial(state) => InviteeFullState::Invited((state, invitation.clone()).into()),
            s @ _ => { return Err(VcxError::from_msg(VcxErrorKind::InvalidState, format!("Cannot handle inviation: not in Initial state, current state: {:?}", s))); }
        };
        Ok(Self { state, thread_id: invitation.get_id()?, ..self })
    }

    pub async fn handle_connect<F, T>(self, routing_keys: Vec<String>, service_endpoint: String, send_message: F) -> VcxResult<Self>
        where
            F: Fn(String, DidDoc, A2AMessage) -> T,
            T: Future<Output=VcxResult<()>>
    {
        let (state, thread_id) = match self.state {
            InviteeFullState::Invited(ref state) => {
                let recipient_keys = vec!(self.pairwise_info.pw_vk.clone());
                let request = Request::create()
                    .set_label(self.source_id.to_string())
                    .set_did(self.pairwise_info.pw_did.to_string())
                    .set_service_endpoint(service_endpoint)
                    .set_keys(recipient_keys, routing_keys);
                let (request, thread_id) = match &state.invitation {
                    Invitation::Public(_) => (
                        request
                            .clone()
                            .set_parent_thread_id(&self.thread_id)
                            .set_thread_id_matching_id(),
                        request.id.0.clone()
                    ),
                    Invitation::Pairwise(_) => (
                        request
                            .set_thread_id(&self.thread_id),
                        self.get_thread_id()
                    ),
                    Invitation::OutOfBand(invite) => (
                        request
                            .clone()
                            .set_parent_thread_id(&invite.id.0)
                            .set_thread_id_matching_id(),
                        request.id.0.clone()
                    )
                };
                let ddo = DidDoc::from(state.invitation.clone());
                send_message(self.pairwise_info.pw_vk.clone(), ddo, request.to_a2a_message()).await?;
                (InviteeFullState::Requested((state.clone(), request).into()), thread_id)
            }
            _ => (self.state.clone(), self.get_thread_id())
        };
        Ok(Self { state, thread_id, ..self })
    }

    pub fn handle_connection_response(self, response: SignedResponse) -> VcxResult<Self> {
        verify_thread_id(&self.get_thread_id(), &A2AMessage::ConnectionResponse(response.clone()))?;
        let state = match self.state {
            InviteeFullState::Requested(state) => {
                InviteeFullState::Responded((state, response).into())
            }
            _ => self.state.clone()
        };
        Ok(Self { state, ..self })
    }

    pub async fn handle_ping<F, T>(self, ping: Ping, send_message: F) -> VcxResult<Self>
        where
            F: Fn(String, DidDoc, A2AMessage) -> T,
            T: Future<Output=VcxResult<()>>
    {
        verify_thread_id(&self.get_thread_id(), &A2AMessage::Ping(ping.clone()))?;
        let state = match self.state {
            InviteeFullState::Completed(state) => {
                state.handle_ping(&ping, &self.pairwise_info.pw_vk, send_message).await?;
                InviteeFullState::Completed(state)
            }
            _ => self.state.clone()
        };
        Ok(Self { state, ..self })
    }

    pub async fn handle_send_ping<F, T>(self, comment: Option<String>, send_message: F) -> VcxResult<Self>
        where
            F: Fn(String, DidDoc, A2AMessage) -> T,
            T: Future<Output=VcxResult<()>>
    {
        let state = match self.state {
            InviteeFullState::Completed(state) => {
                state.handle_send_ping(comment, &self.pairwise_info.pw_vk, send_message).await?;
                InviteeFullState::Completed(state)
            }
            _ => self.state.clone()
        };
        Ok(Self { state, ..self })
    }

    pub fn handle_ping_response(self, ping_response: PingResponse) -> VcxResult<Self> {
        verify_thread_id(&self.get_thread_id(), &A2AMessage::PingResponse(ping_response))?;
        Ok(self)
    }

    pub async fn handle_send_handshake_reuse<F, T>(self, oob: OutOfBandInvitation, send_message: F) -> VcxResult<Self>
    where
        F: Fn(String, DidDoc, A2AMessage) -> T,
        T: Future<Output=VcxResult<()>>
    {
        let state = match self.state {
            InviteeFullState::Completed(state) => {
                state.handle_send_handshake_reuse(&oob.id.0, &self.pairwise_info.pw_vk, send_message).await?;
                InviteeFullState::Completed(state)
            }
            s @ _ => { return Err(VcxError::from_msg(VcxErrorKind::InvalidState, format!("Handshake reuse can be sent only in the Completed state, current state: {:?}", s))); }
        };
        Ok(Self { state, ..self })
    }

    pub async fn handle_handshake_reuse<F, T>(self,
                            reuse_msg: OutOfBandHandshakeReuse,
                            send_message: F
    ) -> VcxResult<Self>
    where
        F: Fn(String, DidDoc, A2AMessage) -> T,
        T: Future<Output=VcxResult<()>>
    {
        let state = match self.state {
            InviteeFullState::Completed(state) => {
                state.handle_send_handshake_reuse_accepted(reuse_msg, &self.pairwise_info.pw_vk, send_message).await?;
                InviteeFullState::Completed(state)
            }
            s @ _ => { return Err(VcxError::from_msg(VcxErrorKind::InvalidState, format!("Handshake reuse can be accepted only from the Completed state, current state: {:?}", s))); }
        };
        Ok(Self { state, ..self })
    }

    pub async fn handle_discover_features<F, T>(self, query_: Option<String>, comment: Option<String>, send_message: F) -> VcxResult<Self>
        where
            F: Fn(String, DidDoc, A2AMessage) -> T,
            T: Future<Output=VcxResult<()>>
    {
        let state = match self.state {
            InviteeFullState::Completed(state) => {
                state.handle_discover_features(query_, comment, &self.pairwise_info.pw_vk, send_message).await?;
                InviteeFullState::Completed(state)
            }
            _ => self.state.clone()
        };
        Ok(Self { state, ..self })
    }

    pub async fn handle_discovery_query<F, T>(self, query: Query, send_message: F) -> VcxResult<Self>
        where
            F: Fn(String, DidDoc, A2AMessage) -> T,
            T: Future<Output=VcxResult<()>>
    {
        let state = match self.state {
            InviteeFullState::Completed(state) => {
                state.handle_discovery_query(query, &self.pairwise_info.pw_vk, send_message).await?;
                InviteeFullState::Completed(state)
            }
            _ => self.state.clone()
        };
        Ok(Self { state, ..self })
    }

    pub fn handle_disclose(self, disclose: Disclose) -> VcxResult<Self> {
        let Self { state, .. } = self;
        let state = match state {
            InviteeFullState::Completed(state) => {
                InviteeFullState::Completed((state.clone(), disclose.protocols).into())
            }
            _ => state.clone()
        };
        Ok(Self { state, ..self })
    }

    pub async fn handle_send_ack<F, T>(self, send_message: &F) -> VcxResult<Self>
        where
            F: Fn(String, DidDoc, A2AMessage) -> T,
            T: Future<Output=VcxResult<()>>
    {
        let state = match self.state {
            InviteeFullState::Responded(ref state) => {
                match self._send_ack(&state.did_doc, &state.request, &state.response, &self.pairwise_info, send_message).await {
                    Ok(response) => InviteeFullState::Completed((state.clone(), response).into()),
                    Err(err) => {
                        let problem_report = ProblemReport::create()
                            .set_problem_code(ProblemCode::ResponseProcessingError)
                            .set_explain(err.to_string())
                            .set_thread_id(&self.thread_id);
                        send_message(self.pairwise_info.pw_vk.clone(), state.did_doc.clone(), problem_report.to_a2a_message()).await.ok();
                        InviteeFullState::Initial((state.clone(), problem_report).into())
                    }
                }
            }
            _ => self.state.clone()
        };
        Ok(Self { state, ..self })
    }

    pub fn handle_problem_report(self, _problem_report: ProblemReport) -> VcxResult<Self> {
        let state = match self.state {
            InviteeFullState::Requested(_state) => {
                InviteeFullState::Initial(InitialState::new(None))
            }
            InviteeFullState::Invited(_state) => {
                InviteeFullState::Initial(InitialState::new(None))
            }
            _ => self.state.clone()
        };
        Ok(Self { state, ..self })
    }

    pub fn handle_ack(self, _ack: Ack) -> VcxResult<Self> {
        Ok(self)
    }

    pub fn get_thread_id(&self) -> String {
        self.thread_id.clone()
    }
}

#[cfg(test)]
pub mod test {
    use crate::messages::ack::test_utils::_ack;
    use crate::messages::connection::invite::test_utils::_pairwise_invitation;
    use crate::messages::connection::problem_report::tests::_problem_report;
    use crate::messages::connection::request::tests::_request;
    use crate::messages::connection::response::test_utils::_signed_response;
    use crate::messages::discovery::disclose::tests::_disclose;
    use crate::messages::discovery::query::tests::_query;
    use crate::messages::trust_ping::ping::tests::_ping;
    use crate::messages::trust_ping::ping_response::tests::_ping_response;
    use crate::test::source_id;
    use crate::utils::devsetup::SetupMocks;

    use super::*;

    pub mod invitee {
        use indy_sys::WalletHandle;
        use crate::global::wallet::get_main_wallet_handle;
        use crate::messages::connection::did_doc::test_utils::_service_endpoint;
        use crate::messages::connection::response::{Response, SignedResponse};

        use super::*;

        async fn _send_message(_pv_wk: String, _did_doc: DidDoc, _a2a_message: A2AMessage) -> VcxResult<()> {
            VcxResult::Ok(())
        }

        pub async fn invitee_sm() -> SmConnectionInvitee {
            let pairwise_info = PairwiseInfo::create().await.unwrap();
            SmConnectionInvitee::new(&source_id(), pairwise_info)
        }

        impl SmConnectionInvitee {
            pub fn to_invitee_invited_state(mut self) -> SmConnectionInvitee {
                self = self.handle_invitation(Invitation::Pairwise(_pairwise_invitation())).unwrap();
                self
            }

            pub async fn to_invitee_requested_state(mut self) -> SmConnectionInvitee {
                self = self.handle_invitation(Invitation::Pairwise(_pairwise_invitation())).unwrap();
                let routing_keys: Vec<String> = vec!("verkey123".into());
                let service_endpoint = String::from("https://example.org/agent");
                self = self.handle_connect(routing_keys, service_endpoint, _send_message).await.unwrap();
                self
            }

            pub async fn to_invitee_completed_state(mut self) -> SmConnectionInvitee {
                let key = "GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL".to_string();

                self = self.handle_invitation(Invitation::Pairwise(_pairwise_invitation())).unwrap();

                let routing_keys: Vec<String> = vec!(key.clone());
                let service_endpoint = String::from("https://example.org/agent");
                self = self.handle_connect(routing_keys, service_endpoint, _send_message).await.unwrap();
                self = self.handle_connection_response(_response(WalletHandle(0), &key).await).unwrap();
                self = self.handle_send_ack(&_send_message).await.unwrap();
                self = self.handle_ack(_ack()).unwrap();
                self
            }
        }

        async fn _response(wallet_handle: WalletHandle, key: &str) -> SignedResponse {
            Response::default()
                .set_service_endpoint(_service_endpoint())
                .set_keys(vec![key.to_string()], vec![])
                .set_thread_id(&_request().id.0)
                .encode(wallet_handle, &key).await.unwrap()
        }

        async fn _response_1(wallet_handle: WalletHandle, key: &str) -> SignedResponse {
            Response::default()
                .set_service_endpoint(_service_endpoint())
                .set_keys(vec![key.to_string()], vec![])
                .set_thread_id("testid_1")
                .encode(wallet_handle, &key).await.unwrap()
        }

        mod new {
            use super::*;

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_invitee_new() {
                let _setup = SetupMocks::init();

                let invitee_sm = invitee_sm().await;

                assert_match!(InviteeFullState::Initial(_), invitee_sm.state);
                assert_eq!(source_id(), invitee_sm.source_id());
            }
        }

        mod get_thread_id {
            use super::*;

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn handle_response_fails_with_incorrect_thread_id() {
                let _setup = SetupMocks::init();
                let key = "GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL".to_string();
                let mut invitee = invitee_sm().await;

                invitee = invitee.handle_invitation(Invitation::Pairwise(_pairwise_invitation())).unwrap();

                let routing_keys: Vec<String> = vec!("verkey123".into());
                let service_endpoint = String::from("https://example.org/agent");
                invitee = invitee.handle_connect(routing_keys, service_endpoint, _send_message).await.unwrap();
                assert_match!(InviteeState::Requested, invitee.get_state());
                invitee = invitee.handle_connection_response(_response_1(WalletHandle(0), &key).await).unwrap();
                assert_match!(InviteeState::Responded, invitee.get_state());
                invitee = invitee.handle_send_ack(&_send_message).await.unwrap();
                assert_match!(InviteeState::Initial, invitee.get_state());
            }
        }

        mod step {
            use crate::utils::devsetup::SetupIndyMocks;

            use super::*;

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_init() {
                let _setup = SetupIndyMocks::init();

                let did_exchange_sm = invitee_sm().await;

                assert_match!(InviteeFullState::Initial(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_invite_message_from_null_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = invitee_sm().await;

                did_exchange_sm = did_exchange_sm.handle_invitation(Invitation::Pairwise(_pairwise_invitation())).unwrap();

                assert_match!(InviteeFullState::Invited(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_other_message_from_null_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = invitee_sm().await;

                let routing_keys: Vec<String> = vec!("verkey123".into());
                let service_endpoint = String::from("https://example.org/agent");
                did_exchange_sm = did_exchange_sm.handle_connect(routing_keys, service_endpoint, _send_message).await.unwrap();
                assert_match!(InviteeFullState::Initial(_), did_exchange_sm.state);

                did_exchange_sm = did_exchange_sm.handle_ack(_ack()).unwrap();
                assert_match!(InviteeFullState::Initial(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_connect_message_from_invited_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = invitee_sm().await.to_invitee_invited_state();

                let routing_keys: Vec<String> = vec!("verkey123".into());
                let service_endpoint = String::from("https://example.org/agent");
                did_exchange_sm = did_exchange_sm.handle_connect(routing_keys, service_endpoint, _send_message).await.unwrap();

                assert_match!(InviteeFullState::Requested(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_problem_report_message_from_invited_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = invitee_sm().await.to_invitee_invited_state();

                did_exchange_sm = did_exchange_sm.handle_problem_report(_problem_report()).unwrap();

                assert_match!(InviteeFullState::Initial(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_response_message_from_requested_state() {
                let _setup = SetupIndyMocks::init();

                let key = "GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL";

                let mut did_exchange_sm = invitee_sm().await.to_invitee_requested_state().await;

                did_exchange_sm = did_exchange_sm.handle_connection_response(_response(WalletHandle(0), &key).await).unwrap();
                did_exchange_sm = did_exchange_sm.handle_send_ack(&_send_message).await.unwrap();

                assert_match!(InviteeFullState::Completed(_), did_exchange_sm.state);
            }


            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_other_messages_from_invited_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = invitee_sm().await.to_invitee_invited_state();

                did_exchange_sm = did_exchange_sm.handle_ack(_ack()).unwrap();
                assert_match!(InviteeFullState::Invited(_), did_exchange_sm.state);

                did_exchange_sm = did_exchange_sm.handle_discovery_query(_query(), _send_message).await.unwrap();
                assert_match!(InviteeFullState::Invited(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_invalid_response_message_from_requested_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = invitee_sm().await.to_invitee_requested_state().await;

                let mut signed_response = _signed_response();
                signed_response.connection_sig.signature = String::from("other");

                did_exchange_sm = did_exchange_sm.handle_connection_response(signed_response).unwrap();
                did_exchange_sm = did_exchange_sm.handle_send_ack(&_send_message).await.unwrap();

                assert_match!(InviteeFullState::Initial(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_problem_report_message_from_requested_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = invitee_sm().await.to_invitee_requested_state().await;

                did_exchange_sm = did_exchange_sm.handle_problem_report(_problem_report()).unwrap();

                assert_match!(InviteeFullState::Initial(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_other_messages_from_requested_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = invitee_sm().await.to_invitee_requested_state().await;

                did_exchange_sm = did_exchange_sm.handle_ack(_ack()).unwrap();
                assert_match!(InviteeFullState::Requested(_), did_exchange_sm.state);

                did_exchange_sm = did_exchange_sm.handle_ping(_ping(), _send_message).await.unwrap();
                assert_match!(InviteeFullState::Requested(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_messages_from_completed_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = invitee_sm().await.to_invitee_completed_state().await;

                // Send Ping
                did_exchange_sm = did_exchange_sm.handle_send_ping(None, _send_message).await.unwrap();
                assert_match!(InviteeFullState::Completed(_), did_exchange_sm.state);

                // Ping
                did_exchange_sm = did_exchange_sm.handle_ping(_ping(), _send_message).await.unwrap();
                assert_match!(InviteeFullState::Completed(_), did_exchange_sm.state);

                // Ping Response
                did_exchange_sm = did_exchange_sm.handle_ping_response(_ping_response()).unwrap();
                assert_match!(InviteeFullState::Completed(_), did_exchange_sm.state);

                // Discovery Features
                did_exchange_sm = did_exchange_sm.handle_discover_features(None, None, _send_message).await.unwrap();
                assert_match!(InviteeFullState::Completed(_), did_exchange_sm.state);

                // Query
                did_exchange_sm = did_exchange_sm.handle_discovery_query(_query(), _send_message).await.unwrap();
                assert_match!(InviteeFullState::Completed(_), did_exchange_sm.state);

                // Disclose
                assert!(did_exchange_sm.get_remote_protocols().is_none());

                did_exchange_sm = did_exchange_sm.handle_disclose(_disclose()).unwrap();
                assert_match!(InviteeFullState::Completed(_), did_exchange_sm.state);

                assert!(did_exchange_sm.get_remote_protocols().is_some());

                // ignore
                // Ack
                did_exchange_sm = did_exchange_sm.handle_ack(_ack()).unwrap();
                assert_match!(InviteeFullState::Completed(_), did_exchange_sm.state);

                // Problem Report
                did_exchange_sm = did_exchange_sm.handle_problem_report(_problem_report()).unwrap();
                assert_match!(InviteeFullState::Completed(_), did_exchange_sm.state);
            }
        }

        mod find_message_to_handle {
            use crate::utils::devsetup::SetupIndyMocks;

            use super::*;

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_find_message_to_handle_from_invited_state() {
                let _setup = SetupIndyMocks::init();

                let connection = invitee_sm().await.to_invitee_invited_state();

                // No messages
                {
                    let messages = map!(
                        "key_1".to_string() => A2AMessage::ConnectionRequest(_request()),
                        "key_2".to_string() => A2AMessage::ConnectionResponse(_signed_response()),
                        "key_3".to_string() => A2AMessage::ConnectionProblemReport(_problem_report()),
                        "key_4".to_string() => A2AMessage::Ping(_ping()),
                        "key_5".to_string() => A2AMessage::Ack(_ack())
                    );

                    assert!(connection.find_message_to_handle(messages).is_none());
                }
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_find_message_to_handle_from_requested_state() {
                let _setup = SetupIndyMocks::init();

                let connection = invitee_sm().await.to_invitee_requested_state().await;

                // Connection Response
                {
                    let messages = map!(
                        "key_1".to_string() => A2AMessage::Ping(_ping()),
                        "key_2".to_string() => A2AMessage::ConnectionRequest(_request()),
                        "key_3".to_string() => A2AMessage::ConnectionResponse(_signed_response())
                    );

                    let (uid, message) = connection.find_message_to_handle(messages).unwrap();
                    assert_eq!("key_3", uid);
                    assert_match!(A2AMessage::ConnectionResponse(_), message);
                }

                // Connection Problem Report
                {
                    let messages = map!(
                        "key_1".to_string() => A2AMessage::Ping(_ping()),
                        "key_2".to_string() => A2AMessage::Ack(_ack()),
                        "key_3".to_string() => A2AMessage::ConnectionProblemReport(_problem_report())
                    );

                    let (uid, message) = connection.find_message_to_handle(messages).unwrap();
                    assert_eq!("key_3", uid);
                    assert_match!(A2AMessage::ConnectionProblemReport(_), message);
                }

                // No messages
                {
                    let messages = map!(
                        "key_1".to_string() => A2AMessage::Ping(_ping()),
                        "key_2".to_string() => A2AMessage::Ack(_ack())
                    );

                    assert!(connection.find_message_to_handle(messages).is_none());
                }
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_find_message_to_handle_from_completed_state() {
                let _setup = SetupIndyMocks::init();

                let connection = invitee_sm().await.to_invitee_completed_state().await;

                // Ping
                {
                    let messages = map!(
                        "key_1".to_string() => A2AMessage::ConnectionRequest(_request()),
                        "key_2".to_string() => A2AMessage::ConnectionResponse(_signed_response()),
                        "key_3".to_string() => A2AMessage::ConnectionProblemReport(_problem_report()),
                        "key_4".to_string() => A2AMessage::Ping(_ping()),
                        "key_5".to_string() => A2AMessage::Ack(_ack())
                    );

                    let (uid, message) = connection.find_message_to_handle(messages).unwrap();
                    assert_eq!("key_4", uid);
                    assert_match!(A2AMessage::Ping(_), message);
                }

                // Ping Response
                {
                    let messages = map!(
                        "key_1".to_string() => A2AMessage::ConnectionRequest(_request()),
                        "key_2".to_string() => A2AMessage::ConnectionResponse(_signed_response()),
                        "key_3".to_string() => A2AMessage::ConnectionProblemReport(_problem_report()),
                        "key_4".to_string() => A2AMessage::PingResponse(_ping_response()),
                        "key_5".to_string() => A2AMessage::Ack(_ack())
                    );

                    let (uid, message) = connection.find_message_to_handle(messages).unwrap();
                    assert_eq!("key_4", uid);
                    assert_match!(A2AMessage::PingResponse(_), message);
                }

                // Query
                {
                    let messages = map!(
                        "key_1".to_string() => A2AMessage::ConnectionRequest(_request()),
                        "key_2".to_string() => A2AMessage::ConnectionResponse(_signed_response()),
                        "key_3".to_string() => A2AMessage::Query(_query())
                    );

                    let (uid, message) = connection.find_message_to_handle(messages).unwrap();
                    assert_eq!("key_3", uid);
                    assert_match!(A2AMessage::Query(_), message);
                }

                // Disclose
                {
                    let messages = map!(
                        "key_1".to_string() => A2AMessage::ConnectionRequest(_request()),
                        "key_2".to_string() => A2AMessage::ConnectionResponse(_signed_response()),
                        "key_3".to_string() => A2AMessage::Disclose(_disclose())
                    );

                    let (uid, message) = connection.find_message_to_handle(messages).unwrap();
                    assert_eq!("key_3", uid);
                    assert_match!(A2AMessage::Disclose(_), message);
                }
            }
        }

        mod get_state {
            use super::*;

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_get_state() {
                let _setup = SetupMocks::init();

                assert_eq!(InviteeState::Initial, invitee_sm().await.get_state());
                assert_eq!(InviteeState::Invited, invitee_sm().await.to_invitee_invited_state().get_state());
                assert_eq!(InviteeState::Requested, invitee_sm().await.to_invitee_requested_state().await.get_state());
            }
        }
    }
}
