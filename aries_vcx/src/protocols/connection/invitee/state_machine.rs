use std::clone::Clone;
use std::collections::HashMap;
use std::future::Future;

use indy_sys::{WalletHandle, PoolHandle};

use crate::did_doc::DidDoc;
use crate::error::prelude::*;
use crate::handlers::util::verify_thread_id;
use crate::messages::a2a::protocol_registry::ProtocolRegistry;
use crate::messages::a2a::A2AMessage;
use crate::messages::ack::Ack;
use crate::messages::connection::invite::Invitation;
use crate::messages::connection::problem_report::{ProblemCode, ProblemReport};
use crate::messages::connection::request::Request;
use crate::messages::connection::response::{Response, SignedResponse};
use crate::messages::discovery::disclose::{Disclose, ProtocolDescriptor};
use crate::protocols::connection::invitee::states::complete::CompleteState;
use crate::protocols::connection::invitee::states::initial::InitialState;
use crate::protocols::connection::invitee::states::invited::InvitedState;
use crate::protocols::connection::invitee::states::requested::RequestedState;
use crate::protocols::connection::invitee::states::responded::RespondedState;
use crate::protocols::connection::pairwise_info::PairwiseInfo;

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
        self.source_id == other.source_id && self.pairwise_info == other.pairwise_info && self.state == other.state
    }
}

impl From<InviteeFullState> for InviteeState {
    fn from(state: InviteeFullState) -> InviteeState {
        match state {
            InviteeFullState::Initial(_) => InviteeState::Initial,
            InviteeFullState::Invited(_) => InviteeState::Invited,
            InviteeFullState::Requested(_) => InviteeState::Requested,
            InviteeFullState::Responded(_) => InviteeState::Responded,
            InviteeFullState::Completed(_) => InviteeState::Completed,
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

    pub fn is_in_null_state(&self) -> bool {
        match self.state {
            InviteeFullState::Initial(_) => true,
            _ => false,
        }
    }

    pub fn is_in_final_state(&self) -> bool {
        match self.state {
            InviteeFullState::Completed(_) => true,
            _ => false,
        }
    }

    pub fn their_did_doc(&self, pool_handle: PoolHandle) -> Option<DidDoc> {
        match self.state {
            InviteeFullState::Initial(_) => None,
            InviteeFullState::Invited(ref state) => state.invitation.into_did_doc(pool_handle).ok(),
            InviteeFullState::Requested(ref state) => Some(state.did_doc.clone()),
            InviteeFullState::Responded(ref state) => Some(state.did_doc.clone()),
            InviteeFullState::Completed(ref state) => Some(state.did_doc.clone()),
        }
    }

    pub fn bootstrap_did_doc(&self, pool_handle: PoolHandle) -> Option<DidDoc> {
        match self.state {
            InviteeFullState::Initial(_) => None,
            InviteeFullState::Invited(ref state) => state.invitation.into_did_doc(pool_handle).ok(),
            InviteeFullState::Requested(ref state) => Some(state.did_doc.clone()),
            InviteeFullState::Responded(ref state) => Some(state.did_doc.clone()),
            InviteeFullState::Completed(ref state) => Some(state.bootstrap_did_doc.clone()),
        }
    }

    pub fn get_invitation(&self) -> Option<&Invitation> {
        match self.state {
            InviteeFullState::Invited(ref state) => Some(&state.invitation),
            _ => None,
        }
    }

    pub fn find_message_to_update_state(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        for (uid, message) in messages {
            if self.can_progress_state(&message) {
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
            _ => None,
        }
    }

    pub fn remote_did(&self, pool_handle: PoolHandle) -> VcxResult<String> {
        self.their_did_doc(pool_handle)
            .map(|did_doc: DidDoc| did_doc.id)
            .ok_or(VcxError::from_msg(
                VcxErrorKind::NotReady,
                "Remote Connection DID is not set",
            ))
    }

    pub fn remote_vk(&self, pool_handle: PoolHandle) -> VcxResult<String> {
        self.their_did_doc(pool_handle)
            .and_then(|did_doc| did_doc.recipient_keys().get(0).cloned())
            .ok_or(VcxError::from_msg(
                VcxErrorKind::NotReady,
                "Remote Connection Verkey is not set",
            ))
    }

    pub fn can_progress_state(&self, message: &A2AMessage) -> bool {
        match self.state {
            InviteeFullState::Requested(_) => match message {
                A2AMessage::ConnectionResponse(_) | A2AMessage::ConnectionProblemReport(_) => true,
                _ => false,
            },
            _ => false,
        }
    }

    // todo: should only build message, logic of thread_id determination should be separate
    fn build_connection_request_msg(
        &self,
        routing_keys: Vec<String>,
        service_endpoint: String,
    ) -> VcxResult<(Request, String)> {
        match &self.state {
            InviteeFullState::Invited(state) => {
                let recipient_keys = vec![self.pairwise_info.pw_vk.clone()];
                let request = Request::create()
                    .set_label(self.source_id.to_string())
                    .set_did(self.pairwise_info.pw_did.to_string())
                    .set_service_endpoint(service_endpoint.to_string())
                    .set_keys(recipient_keys, routing_keys)
                    .set_out_time();
                let (request, thread_id) = match &state.invitation {
                    Invitation::Public(_) => (
                        request
                            .clone()
                            .set_parent_thread_id(&self.thread_id)
                            .set_thread_id_matching_id(),
                        request.id.0.clone(),
                    ),
                    Invitation::Pairwise(_) => (request.set_thread_id(&self.thread_id), self.get_thread_id()),
                    Invitation::OutOfBand(invite) => (
                        request
                            .clone()
                            .set_parent_thread_id(&invite.id.0)
                            .set_thread_id_matching_id(),
                        request.id.0.clone(),
                    ),
                };
                Ok((request, thread_id))
            }
            _ => Err(VcxError::from_msg(
                VcxErrorKind::NotReady,
                "Building connection request in current state is not allowed",
            )),
        }
    }

    fn build_connection_ack_msg(&self) -> VcxResult<Ack> {
        match &self.state {
            InviteeFullState::Responded(_) => Ok(Ack::create().set_out_time().set_thread_id(&self.thread_id)),
            _ => Err(VcxError::from_msg(
                VcxErrorKind::NotReady,
                "Building connection ack in current state is not allowed",
            )),
        }
    }

    // todo: extract response validation to different function
    async fn _send_ack<F, T>(
        &self,
        wallet_handle: WalletHandle,
        did_doc: &DidDoc,
        request: &Request,
        response: &SignedResponse,
        pairwise_info: &PairwiseInfo,
        send_message: F,
    ) -> VcxResult<Response>
    where
        F: Fn(WalletHandle, String, DidDoc, A2AMessage) -> T,
        T: Future<Output = VcxResult<()>>,
    {
        let remote_vk: String = did_doc.recipient_keys().get(0).cloned().ok_or(VcxError::from_msg(
            VcxErrorKind::InvalidState,
            "Cannot handle response: remote verkey not found",
        ))?;

        let response = response.clone().decode(&remote_vk).await?;

        if !response.from_thread(&request.get_thread_id()) {
            return Err(VcxError::from_msg(
                VcxErrorKind::InvalidJson,
                format!(
                    "Cannot handle response: thread id does not match: {:?}",
                    response.thread
                ),
            ));
        }

        let message = self.build_connection_ack_msg()?.to_a2a_message();

        send_message(
            wallet_handle,
            pairwise_info.pw_vk.clone(),
            response.connection.did_doc.clone(),
            message,
        )
        .await?;
        Ok(response)
    }

    pub fn handle_invitation(self, invitation: Invitation) -> VcxResult<Self> {
        let Self { state, .. } = self;
        let state = match state {
            InviteeFullState::Initial(state) => InviteeFullState::Invited((state, invitation.clone()).into()),
            s => {
                return Err(VcxError::from_msg(
                    VcxErrorKind::InvalidState,
                    format!("Cannot handle inviation: not in Initial state, current state: {:?}", s),
                ));
            }
        };
        Ok(Self {
            state,
            thread_id: invitation.get_id()?,
            ..self
        })
    }

    pub async fn send_connection_request<F, T>(
        self,
        wallet_handle: WalletHandle,
        pool_handle: PoolHandle,
        routing_keys: Vec<String>,
        service_endpoint: String,
        send_message: F,
    ) -> VcxResult<Self>
    where
        F: Fn(WalletHandle, String, DidDoc, A2AMessage) -> T,
        T: Future<Output = VcxResult<()>>,
    {
        let (state, thread_id) = match self.state {
            InviteeFullState::Invited(ref state) => {
                let ddo = state.invitation.into_did_doc(pool_handle)?;
                let (request, thread_id) = self.build_connection_request_msg(routing_keys, service_endpoint)?;
                send_message(
                    wallet_handle,
                    self.pairwise_info.pw_vk.clone(),
                    ddo,
                    request.to_a2a_message(),
                )
                .await?;
                (InviteeFullState::Requested((state.clone(), request, pool_handle).into()), thread_id)
            }
            _ => (self.state.clone(), self.get_thread_id()),
        };
        Ok(Self {
            state,
            thread_id,
            ..self
        })
    }

    pub fn handle_connection_response(self, response: SignedResponse) -> VcxResult<Self> {
        verify_thread_id(&self.get_thread_id(), &A2AMessage::ConnectionResponse(response.clone()))?;
        let state = match self.state {
            InviteeFullState::Requested(state) => InviteeFullState::Responded((state, response).into()),
            _ => self.state.clone(),
        };
        Ok(Self { state, ..self })
    }

    pub fn handle_disclose(self, disclose: Disclose) -> VcxResult<Self> {
        let state = match self.state {
            InviteeFullState::Completed(state) => InviteeFullState::Completed((state, disclose.protocols).into()),
            _ => self.state,
        };
        Ok(Self { state, ..self })
    }

    // todo: send ack is validaiting connection response, should be moved to handle_connection_response
    pub async fn handle_send_ack<F, T>(self, wallet_handle: WalletHandle, send_message: &F) -> VcxResult<Self>
    where
        F: Fn(WalletHandle, String, DidDoc, A2AMessage) -> T,
        T: Future<Output = VcxResult<()>>,
    {
        let state = match self.state {
            InviteeFullState::Responded(ref state) => match self
                ._send_ack(
                    wallet_handle,
                    &state.did_doc,
                    &state.request,
                    &state.response,
                    &self.pairwise_info,
                    send_message,
                )
                .await
            {
                Ok(response) => InviteeFullState::Completed((state.clone(), response).into()),
                Err(err) => {
                    let problem_report = ProblemReport::create()
                        .set_problem_code(ProblemCode::ResponseProcessingError)
                        .set_explain(err.to_string())
                        .set_thread_id(&self.thread_id)
                        .set_out_time();
                    send_message(
                        wallet_handle,
                        self.pairwise_info.pw_vk.clone(),
                        state.did_doc.clone(),
                        problem_report.to_a2a_message(),
                    )
                    .await
                    .ok();
                    InviteeFullState::Initial((state.clone(), problem_report).into())
                }
            },
            _ => self.state.clone(),
        };
        Ok(Self { state, ..self })
    }

    pub fn handle_problem_report(self, _problem_report: ProblemReport) -> VcxResult<Self> {
        let state = match self.state {
            InviteeFullState::Requested(_state) => InviteeFullState::Initial(InitialState::new(None)),
            InviteeFullState::Invited(_state) => InviteeFullState::Initial(InitialState::new(None)),
            _ => self.state.clone(),
        };
        Ok(Self { state, ..self })
    }

    pub fn get_thread_id(&self) -> String {
        self.thread_id.clone()
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use crate::messages::ack::test_utils::_ack;
    use crate::messages::connection::invite::test_utils::_pairwise_invitation;
    use crate::messages::connection::problem_report::unit_tests::_problem_report;
    use crate::messages::connection::request::unit_tests::_request;
    use crate::messages::connection::response::test_utils::_signed_response;
    use crate::messages::discovery::disclose::test_utils::_disclose;
    
    use crate::messages::trust_ping::ping::unit_tests::_ping;
    
    use crate::test::source_id;
    use crate::utils::devsetup::SetupMocks;

    use super::*;

    fn _dummy_wallet_handle() -> WalletHandle {
        WalletHandle(0)
    }

    pub mod invitee {
        use indy_sys::WalletHandle;

        use crate::did_doc::test_utils::_service_endpoint;
        use crate::messages::connection::response::{Response, SignedResponse};

        use super::*;

        fn _dummy_pool_handle() -> PoolHandle {
            0
        }

        async fn _send_message(
            _wallet_handle: WalletHandle,
            _pv_wk: String,
            _did_doc: DidDoc,
            _a2a_message: A2AMessage,
        ) -> VcxResult<()> {
            VcxResult::Ok(())
        }

        pub async fn invitee_sm() -> SmConnectionInvitee {
            let pairwise_info = PairwiseInfo::create(_dummy_wallet_handle()).await.unwrap();
            SmConnectionInvitee::new(&source_id(), pairwise_info)
        }

        impl SmConnectionInvitee {
            pub fn to_invitee_invited_state(mut self) -> SmConnectionInvitee {
                self = self
                    .handle_invitation(Invitation::Pairwise(_pairwise_invitation()))
                    .unwrap();
                self
            }

            pub async fn to_invitee_requested_state(mut self) -> SmConnectionInvitee {
                self = self.to_invitee_invited_state();
                let routing_keys: Vec<String> = vec!["verkey123".into()];
                let service_endpoint = String::from("https://example.org/agent");
                self = self
                    .send_connection_request(_dummy_wallet_handle(), _dummy_pool_handle(), routing_keys, service_endpoint, _send_message)
                    .await
                    .unwrap();
                self
            }

            pub async fn to_invitee_completed_state(mut self) -> SmConnectionInvitee {
                let key = "GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL".to_string();
                self = self.to_invitee_requested_state().await;
                self = self
                    .handle_connection_response(_response(WalletHandle(0), &key, &_request().id.0).await)
                    .unwrap();
                self = self
                    .handle_send_ack(_dummy_wallet_handle(), &_send_message)
                    .await
                    .unwrap();
                self
            }
        }

        async fn _response(wallet_handle: WalletHandle, recipient_key: &str, thread_id: &str) -> SignedResponse {
            Response::default()
                .set_service_endpoint(_service_endpoint())
                .set_keys(vec![recipient_key.to_string()], vec![])
                .set_thread_id(thread_id)
                .encode(wallet_handle, &recipient_key)
                .await
                .unwrap()
        }

        async fn _response_1(wallet_handle: WalletHandle, key: &str) -> SignedResponse {
            Response::default()
                .set_service_endpoint(_service_endpoint())
                .set_keys(vec![key.to_string()], vec![])
                .set_thread_id("testid_1")
                .encode(wallet_handle, &key)
                .await
                .unwrap()
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

        mod build_messages {
            use super::*;
            use crate::messages::a2a::MessageId;
            use crate::messages::ack::AckStatus;
            use crate::utils::devsetup::was_in_past;

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_build_connection_request_msg() {
                let _setup = SetupMocks::init();

                let mut invitee = invitee_sm().await;

                let msg_invitation = _pairwise_invitation();
                invitee = invitee
                    .handle_invitation(Invitation::Pairwise(msg_invitation.clone()))
                    .unwrap();
                let routing_keys: Vec<String> = vec!["ABCD000000QYfNL9XkaJdrQejfztN4XqdsiV4ct30000".to_string()];
                let service_endpoint = String::from("https://example.org");
                let (msg, _) = invitee
                    .build_connection_request_msg(routing_keys.clone(), service_endpoint.clone())
                    .unwrap();

                assert_eq!(msg.connection.did_doc.routing_keys(), routing_keys);
                assert_eq!(
                    msg.connection.did_doc.recipient_keys(),
                    vec![invitee.pairwise_info.pw_vk.clone()]
                );
                assert_eq!(msg.connection.did_doc.get_endpoint(), service_endpoint.to_string());
                assert_eq!(msg.id, MessageId::default());
                assert!(was_in_past(
                    &msg.timing.unwrap().out_time.unwrap(),
                    chrono::Duration::milliseconds(100)
                )
                .unwrap());
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_build_connection_ack_msg() {
                let _setup = SetupMocks::init();

                let mut invitee = invitee_sm().await;
                invitee = invitee.to_invitee_requested_state().await;
                let msg_request = &_request();
                let recipient_key = "GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL".to_string();
                invitee = invitee
                    .handle_connection_response(_response(WalletHandle(0), &recipient_key, &msg_request.id.0).await)
                    .unwrap();

                let msg = invitee.build_connection_ack_msg().unwrap();

                assert_eq!(msg.id, MessageId::default());
                assert_eq!(msg.thread.thid.unwrap(), msg_request.id.0);
                assert_eq!(msg.status, AckStatus::Ok);
                assert!(was_in_past(
                    &msg.timing.unwrap().out_time.unwrap(),
                    chrono::Duration::milliseconds(100)
                )
                .unwrap());
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

                invitee = invitee
                    .handle_invitation(Invitation::Pairwise(_pairwise_invitation()))
                    .unwrap();
                let routing_keys: Vec<String> = vec!["verkey123".into()];
                let service_endpoint = String::from("https://example.org/agent");
                invitee = invitee
                    .send_connection_request(_dummy_wallet_handle(), _dummy_pool_handle(), routing_keys, service_endpoint, _send_message)
                    .await
                    .unwrap();
                assert_match!(InviteeState::Requested, invitee.get_state());
                assert!(invitee
                    .handle_connection_response(_response_1(WalletHandle(0), &key).await)
                    .is_err());
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

                did_exchange_sm = did_exchange_sm
                    .handle_invitation(Invitation::Pairwise(_pairwise_invitation()))
                    .unwrap();

                assert_match!(InviteeFullState::Invited(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_wont_sent_connection_request_in_null_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = invitee_sm().await;

                let routing_keys: Vec<String> = vec!["verkey123".into()];
                let service_endpoint = String::from("https://example.org/agent");
                did_exchange_sm = did_exchange_sm
                    .send_connection_request(_dummy_wallet_handle(), _dummy_pool_handle(), routing_keys, service_endpoint, _send_message)
                    .await
                    .unwrap();
                assert_match!(InviteeFullState::Initial(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_wont_accept_connection_response_in_null_state() {
                let _setup = SetupIndyMocks::init();

                let did_exchange_sm = invitee_sm().await;

                let key = "GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL";
                assert!(did_exchange_sm
                    .handle_connection_response(_response(WalletHandle(0), key, &_request().id.0).await)
                    .is_err());
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_connect_message_from_invited_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = invitee_sm().await.to_invitee_invited_state();

                let routing_keys: Vec<String> = vec!["verkey123".into()];
                let service_endpoint = String::from("https://example.org/agent");
                did_exchange_sm = did_exchange_sm
                    .send_connection_request(_dummy_wallet_handle(), _dummy_pool_handle(), routing_keys, service_endpoint, _send_message)
                    .await
                    .unwrap();

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

                did_exchange_sm = did_exchange_sm
                    .handle_connection_response(_response(WalletHandle(0), &key, &_request().id.0).await)
                    .unwrap();
                did_exchange_sm = did_exchange_sm
                    .handle_send_ack(_dummy_wallet_handle(), &_send_message)
                    .await
                    .unwrap();

                assert_match!(InviteeFullState::Completed(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_other_messages_from_invited_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = invitee_sm().await.to_invitee_invited_state();

                did_exchange_sm = did_exchange_sm.handle_disclose(_disclose()).unwrap();
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
                did_exchange_sm = did_exchange_sm
                    .handle_send_ack(_dummy_wallet_handle(), &_send_message)
                    .await
                    .unwrap();

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

                did_exchange_sm = did_exchange_sm.handle_disclose(_disclose()).unwrap();
                assert_match!(InviteeFullState::Requested(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_messages_from_completed_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = invitee_sm().await.to_invitee_completed_state().await;

                // Disclose
                assert!(did_exchange_sm.get_remote_protocols().is_none());

                did_exchange_sm = did_exchange_sm.handle_disclose(_disclose()).unwrap();
                assert_match!(InviteeFullState::Completed(_), did_exchange_sm.state);

                assert!(did_exchange_sm.get_remote_protocols().is_some());

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

                    assert!(connection.find_message_to_update_state(messages).is_none());
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

                    let (uid, message) = connection.find_message_to_update_state(messages).unwrap();
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

                    let (uid, message) = connection.find_message_to_update_state(messages).unwrap();
                    assert_eq!("key_3", uid);
                    assert_match!(A2AMessage::ConnectionProblemReport(_), message);
                }

                // No messages
                {
                    let messages = map!(
                        "key_1".to_string() => A2AMessage::Ping(_ping()),
                        "key_2".to_string() => A2AMessage::Ack(_ack())
                    );

                    assert!(connection.find_message_to_update_state(messages).is_none());
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
                assert_eq!(
                    InviteeState::Invited,
                    invitee_sm().await.to_invitee_invited_state().get_state()
                );
                assert_eq!(
                    InviteeState::Requested,
                    invitee_sm().await.to_invitee_requested_state().await.get_state()
                );
            }
        }
    }
}
