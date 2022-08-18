use std::clone::Clone;
use std::collections::HashMap;
use std::future::Future;

use indy_sys::WalletHandle;

use crate::did_doc::DidDoc;
use crate::error::prelude::*;

use crate::handlers::trust_ping::util::handle_ping;
use crate::handlers::util::verify_thread_id;
use crate::messages::a2a::protocol_registry::ProtocolRegistry;
use crate::messages::a2a::{A2AMessage, MessageId};
use crate::messages::ack::Ack;
use crate::messages::connection::invite::{Invitation, PairwiseInvitation};
use crate::messages::connection::problem_report::{ProblemCode, ProblemReport};
use crate::messages::connection::request::Request;
use crate::messages::connection::response::{Response, SignedResponse};
use crate::messages::discovery::disclose::{Disclose, ProtocolDescriptor};

use crate::messages::trust_ping::ping::Ping;
use crate::protocols::connection::inviter::states::complete::CompleteState;
use crate::protocols::connection::inviter::states::initial::InitialState;
use crate::protocols::connection::inviter::states::invited::InvitedState;
use crate::protocols::connection::inviter::states::requested::RequestedState;
use crate::protocols::connection::inviter::states::responded::RespondedState;
use crate::protocols::connection::pairwise_info::PairwiseInfo;

#[derive(Clone)]
pub struct SmConnectionInviter {
    pub source_id: String,
    thread_id: String,
    pub pairwise_info: PairwiseInfo,
    pub state: InviterFullState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InviterFullState {
    Initial(InitialState),
    Invited(InvitedState),
    Requested(RequestedState),
    Responded(RespondedState),
    Completed(CompleteState),
}

#[derive(Debug, PartialEq)]
pub enum InviterState {
    Initial,
    Invited,
    Requested,
    Responded,
    Completed,
}

impl PartialEq for SmConnectionInviter {
    fn eq(&self, other: &Self) -> bool {
        self.source_id == other.source_id && self.pairwise_info == other.pairwise_info && self.state == other.state
    }
}

impl From<InviterFullState> for InviterState {
    fn from(state: InviterFullState) -> InviterState {
        match state {
            InviterFullState::Initial(_) => InviterState::Initial,
            InviterFullState::Invited(_) => InviterState::Invited,
            InviterFullState::Requested(_) => InviterState::Requested,
            InviterFullState::Responded(_) => InviterState::Responded,
            InviterFullState::Completed(_) => InviterState::Completed,
        }
    }
}

impl SmConnectionInviter {
    pub fn new(source_id: &str, pairwise_info: PairwiseInfo) -> Self {
        Self {
            source_id: source_id.to_string(),
            thread_id: MessageId::new().0,
            state: InviterFullState::Initial(InitialState::new(None)),
            pairwise_info,
        }
    }

    pub fn from(source_id: String, thread_id: String, pairwise_info: PairwiseInfo, state: InviterFullState) -> Self {
        Self {
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

    pub fn get_state(&self) -> InviterState {
        InviterState::from(self.state.clone())
    }

    pub fn state_object(&self) -> &InviterFullState {
        &self.state
    }

    pub fn their_did_doc(&self) -> Option<DidDoc> {
        match self.state {
            InviterFullState::Initial(_) => None,
            InviterFullState::Invited(ref _state) => None,
            InviterFullState::Requested(ref state) => Some(state.did_doc.clone()),
            InviterFullState::Responded(ref state) => Some(state.did_doc.clone()),
            InviterFullState::Completed(ref state) => Some(state.did_doc.clone()),
        }
    }

    pub fn get_invitation(&self) -> Option<&Invitation> {
        match self.state {
            InviterFullState::Invited(ref state) => Some(&state.invitation),
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
            InviterFullState::Completed(ref state) => state.protocols.clone(),
            _ => None,
        }
    }

    pub fn is_in_null_state(&self) -> bool {
        match self.state {
            InviterFullState::Initial(_) => true,
            _ => false,
        }
    }

    pub fn is_in_final_state(&self) -> bool {
        match self.state {
            InviterFullState::Completed(_) => true,
            _ => false,
        }
    }

    pub fn remote_did(&self) -> VcxResult<String> {
        self.their_did_doc()
            .map(|did_doc: DidDoc| did_doc.id)
            .ok_or(VcxError::from_msg(
                VcxErrorKind::NotReady,
                "Remote Connection DID is not set",
            ))
    }

    pub fn remote_vk(&self) -> VcxResult<String> {
        self.their_did_doc()
            .and_then(|did_doc| did_doc.recipient_keys().get(0).cloned())
            .ok_or(VcxError::from_msg(
                VcxErrorKind::NotReady,
                "Remote Connection Verkey is not set",
            ))
    }

    pub fn can_progress_state(&self, message: &A2AMessage) -> bool {
        match self.state {
            InviterFullState::Invited(_) => match message {
                A2AMessage::ConnectionRequest(_) | A2AMessage::ConnectionProblemReport(_) => true,
                _ => false,
            },
            InviterFullState::Responded(_) => match message {
                A2AMessage::Ack(_) | A2AMessage::Ping(_) | A2AMessage::ConnectionProblemReport(_) => true,
                _ => false,
            },
            _ => false,
        }
    }

    async fn _send_response<F, T>(
        wallet_handle: WalletHandle,
        state: &RequestedState,
        new_pw_vk: String,
        send_message: F,
    ) -> VcxResult<()>
    where
        F: Fn(WalletHandle, String, DidDoc, A2AMessage) -> T,
        T: Future<Output = VcxResult<()>>,
    {
        send_message(
            wallet_handle,
            new_pw_vk,
            state.did_doc.clone(),
            state.signed_response.to_a2a_message(),
        )
        .await
    }

    pub fn handle_connect(self, routing_keys: Vec<String>, service_endpoint: String) -> VcxResult<Self> {
        let state = match self.state {
            InviterFullState::Initial(state) => {
                let invite: PairwiseInvitation = PairwiseInvitation::create()
                    .set_id(&self.thread_id)
                    .set_label(&self.source_id)
                    .set_recipient_keys(vec![self.pairwise_info.pw_vk.clone()])
                    .set_routing_keys(routing_keys)
                    .set_service_endpoint(service_endpoint);

                InviterFullState::Invited((state, Invitation::Pairwise(invite)).into())
            }
            _ => self.state.clone(),
        };
        Ok(Self { state, ..self })
    }

    pub async fn handle_connection_request<F, T>(
        self,
        wallet_handle: WalletHandle,
        request: Request,
        new_pairwise_info: &PairwiseInfo,
        new_routing_keys: Vec<String>,
        new_service_endpoint: String,
        send_message: F,
    ) -> VcxResult<Self>
    where
        F: Fn(WalletHandle, String, DidDoc, A2AMessage) -> T,
        T: Future<Output = VcxResult<()>>,
    {
        let bootstrap_pairwise_info = self.pairwise_info.clone();
        let thread_id = request.get_thread_id();
        if !matches!(self.state, InviterFullState::Initial(_)) {
            verify_thread_id(&self.get_thread_id(), &A2AMessage::ConnectionRequest(request.clone()))?;
        };
        let state = match self.state {
            InviterFullState::Invited(_) | InviterFullState::Initial(_) => match &self
                .build_response(
                    wallet_handle,
                    &request,
                    &bootstrap_pairwise_info,
                    new_pairwise_info,
                    new_routing_keys,
                    new_service_endpoint,
                )
                .await
            {
                Ok(signed_response) => InviterFullState::Requested((request, signed_response.clone()).into()),
                Err(err) => {
                    let problem_report = ProblemReport::create()
                        .set_problem_code(ProblemCode::RequestProcessingError)
                        .set_explain(err.to_string())
                        .set_thread_id(&thread_id);

                    send_message(
                        wallet_handle,
                        bootstrap_pairwise_info.pw_vk,
                        request.connection.did_doc,
                        problem_report.to_a2a_message(),
                    )
                    .await
                    .ok();
                    InviterFullState::Initial((problem_report).into())
                }
            },
            _ => self.state,
        };
        Ok(Self {
            pairwise_info: new_pairwise_info.to_owned(),
            thread_id,
            state,
            ..self
        })
    }

    pub async fn handle_ping<F, T>(self, wallet_handle: WalletHandle, ping: Ping, send_message: F) -> VcxResult<Self>
    where
        F: Fn(WalletHandle, String, DidDoc, A2AMessage) -> T,
        T: Future<Output = VcxResult<()>>,
    {
        verify_thread_id(&self.get_thread_id(), &ping.to_a2a_message())?;
        let Self {
            state, pairwise_info, ..
        } = self;
        let state = match state {
            InviterFullState::Responded(state) => {
                handle_ping(wallet_handle, &ping, &pairwise_info.pw_vk, &state.did_doc, send_message).await?;
                InviterFullState::Completed((state, ping).into())
            }
            _ => state,
        };
        Ok(Self {
            state,
            pairwise_info,
            ..self
        })
    }

    pub fn handle_disclose(self, disclose: Disclose) -> VcxResult<Self> {
        let state = match self.state {
            InviterFullState::Completed(state) => InviterFullState::Completed((state, disclose.protocols).into()),
            _ => self.state,
        };
        Ok(Self { state, ..self })
    }

    pub fn handle_problem_report(self, problem_report: ProblemReport) -> VcxResult<Self> {
        let state = match self.state {
            InviterFullState::Responded(_) => InviterFullState::Initial((problem_report).into()),
            InviterFullState::Invited(_) => InviterFullState::Initial((problem_report).into()),
            _ => self.state,
        };
        Ok(Self { state, ..self })
    }

    pub async fn handle_send_response<F, T>(self, wallet_handle: WalletHandle, send_message: &F) -> VcxResult<Self>
    where
        F: Fn(WalletHandle, String, DidDoc, A2AMessage) -> T,
        T: Future<Output = VcxResult<()>>,
    {
        let state = match self.state {
            InviterFullState::Requested(state) => {
                match Self::_send_response(wallet_handle, &state, self.pairwise_info.pw_vk.clone(), send_message).await
                {
                    Ok(_) => InviterFullState::Responded(state.into()),
                    Err(err) => {
                        // todo: we should distinguish errors - probably should not send problem report
                        //       if we just lost internet connectivity
                        let problem_report = ProblemReport::create()
                            .set_problem_code(ProblemCode::RequestProcessingError)
                            .set_explain(err.to_string())
                            .set_thread_id(&self.thread_id);

                        send_message(
                            wallet_handle,
                            self.pairwise_info.pw_vk.clone(),
                            state.did_doc.clone(),
                            problem_report.to_a2a_message(),
                        )
                        .await
                        .ok();
                        InviterFullState::Initial((state, problem_report).into())
                    }
                }
            }
            _ => self.state,
        };
        Ok(Self { state, ..self })
    }

    pub async fn handle_ack<F, T>(self, wallet_handle: WalletHandle, msg_ack: Ack, send_message: F) -> VcxResult<Self>
    where
        F: Fn(WalletHandle, String, DidDoc, A2AMessage) -> T,
        T: Future<Output = VcxResult<()>>,
    {
        let Self {
            state, pairwise_info, ..
        } = self.clone();
        let state = match state {
            InviterFullState::Responded(state) => {
                if !msg_ack.from_thread(&self.get_thread_id()) {
                    let problem_report = ProblemReport::create()
                        .set_problem_code(ProblemCode::RequestProcessingError)
                        .set_explain(format!(
                            "Cannot handle ack: thread id does not match: {:?}",
                            msg_ack.thread
                        ))
                        .set_thread_id(&self.get_thread_id()); // TODO: Maybe set sender's thread id?

                    send_message(
                        wallet_handle,
                        pairwise_info.pw_vk.clone(),
                        state.did_doc.clone(),
                        problem_report.to_a2a_message(),
                    )
                    .await
                    .ok();
                    InviterFullState::Initial((state, problem_report).into())
                } else {
                    InviterFullState::Completed((state, msg_ack).into())
                }
            }
            _ => state,
        };
        Ok(Self { state, ..self })
    }

    pub fn get_thread_id(&self) -> String {
        self.thread_id.clone()
    }

    async fn build_response(
        &self,
        wallet_handle: WalletHandle,
        request: &Request,
        bootstrap_pairwise_info: &PairwiseInfo,
        new_pairwise_info: &PairwiseInfo,
        new_routing_keys: Vec<String>,
        new_service_endpoint: String,
    ) -> VcxResult<SignedResponse> {
        request.connection.did_doc.validate()?;
        let new_recipient_keys = vec![new_pairwise_info.pw_vk.clone()];
        Response::create()
            .set_did(new_pairwise_info.pw_did.to_string())
            .set_service_endpoint(new_service_endpoint)
            .set_keys(new_recipient_keys, new_routing_keys)
            .ask_for_ack()
            .set_thread_id(&request.get_thread_id())
            .encode(wallet_handle, &bootstrap_pairwise_info.pw_vk)
            .await
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use crate::messages::ack::test_utils::{_ack, _ack_1};
    use crate::messages::connection::problem_report::unit_tests::_problem_report;
    use crate::messages::connection::request::unit_tests::_request;
    use crate::messages::connection::response::test_utils::_signed_response;
    use crate::messages::discovery::disclose::test_utils::_disclose;
    use crate::messages::discovery::query::test_utils::_query;
    use crate::messages::trust_ping::ping::unit_tests::_ping;
    use crate::messages::trust_ping::ping_response::unit_tests::_ping_response;
    use crate::test::source_id;
    use crate::utils::devsetup::SetupMocks;

    use super::*;

    fn _dummy_wallet_handle() -> WalletHandle {
        WalletHandle(0)
    }

    pub mod inviter {
        use super::*;

        async fn _send_message(
            _wallet_handle: WalletHandle,
            _pv_wk: String,
            _did_doc: DidDoc,
            _a2a_message: A2AMessage,
        ) -> VcxResult<()> {
            VcxResult::Ok(())
        }

        pub async fn inviter_sm() -> SmConnectionInviter {
            let pairwise_info = PairwiseInfo::create(_dummy_wallet_handle()).await.unwrap();
            SmConnectionInviter::new(&source_id(), pairwise_info)
        }

        impl SmConnectionInviter {
            fn to_inviter_invited_state(mut self) -> SmConnectionInviter {
                let routing_keys: Vec<String> = vec!["verkey123".into()];
                let service_endpoint = String::from("https://example.org/agent");
                self = self.handle_connect(routing_keys, service_endpoint).unwrap();
                self
            }

            async fn to_inviter_responded_state(mut self) -> SmConnectionInviter {
                let routing_keys: Vec<String> = vec!["verkey123".into()];
                let service_endpoint = String::from("https://example.org/agent");
                self = self.handle_connect(routing_keys, service_endpoint).unwrap();

                let new_pairwise_info = PairwiseInfo::create(_dummy_wallet_handle()).await.unwrap();
                let new_routing_keys: Vec<String> = vec!["verkey456".into()];
                let new_service_endpoint = String::from("https://example.org/agent");
                self = self
                    .handle_connection_request(
                        _dummy_wallet_handle(),
                        _request(),
                        &new_pairwise_info,
                        new_routing_keys,
                        new_service_endpoint,
                        _send_message,
                    )
                    .await
                    .unwrap();
                self = self
                    .handle_send_response(_dummy_wallet_handle(), &_send_message)
                    .await
                    .unwrap();
                self
            }

            async fn to_inviter_completed_state(mut self) -> SmConnectionInviter {
                let routing_keys: Vec<String> = vec!["verkey123".into()];
                let service_endpoint = String::from("https://example.org/agent");
                self = self.handle_connect(routing_keys, service_endpoint).unwrap();

                let new_pairwise_info = PairwiseInfo {
                    pw_did: "AC3Gx1RoAz8iYVcfY47gjJ".to_string(),
                    pw_vk: "verkey456".to_string(),
                };
                let new_routing_keys: Vec<String> = vec!["AC3Gx1RoAz8iYVcfY47gjJ".into()];
                let new_service_endpoint = String::from("https://example.org/agent");
                self = self
                    .handle_connection_request(
                        _dummy_wallet_handle(),
                        _request(),
                        &new_pairwise_info,
                        new_routing_keys,
                        new_service_endpoint,
                        _send_message,
                    )
                    .await
                    .unwrap();
                self = self
                    .handle_send_response(_dummy_wallet_handle(), &_send_message)
                    .await
                    .unwrap();
                self = self
                    .handle_ack(_dummy_wallet_handle(), _ack(), _send_message)
                    .await
                    .unwrap();
                self
            }
        }

        mod get_thread_id {
            use super::*;

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn ack_fails_with_incorrect_thread_id() {
                let _setup = SetupMocks::init();
                let routing_keys: Vec<String> = vec!["verkey123".into()];
                let service_endpoint = String::from("https://example.org/agent");
                let mut inviter = inviter_sm().await;
                inviter = inviter.handle_connect(routing_keys, service_endpoint).unwrap();

                let new_pairwise_info = PairwiseInfo {
                    pw_did: "AC3Gx1RoAz8iYVcfY47gjJ".to_string(),
                    pw_vk: "verkey456".to_string(),
                };
                let new_routing_keys: Vec<String> = vec!["AC3Gx1RoAz8iYVcfY47gjJ".into()];
                let new_service_endpoint = String::from("https://example.org/agent");
                inviter = inviter
                    .handle_connection_request(
                        _dummy_wallet_handle(),
                        _request(),
                        &new_pairwise_info,
                        new_routing_keys,
                        new_service_endpoint,
                        _send_message,
                    )
                    .await
                    .unwrap();
                inviter = inviter
                    .handle_send_response(_dummy_wallet_handle(), &_send_message)
                    .await
                    .unwrap();
                inviter = inviter
                    .handle_ack(_dummy_wallet_handle(), _ack_1(), _send_message)
                    .await
                    .unwrap();
                assert_match!(InviterState::Initial, inviter.get_state());
            }
        }

        mod new {
            use super::*;

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_inviter_new() {
                let _setup = SetupMocks::init();

                let inviter_sm = inviter_sm().await;

                assert_match!(InviterFullState::Initial(_), inviter_sm.state);
                assert_eq!(source_id(), inviter_sm.source_id());
            }
        }

        mod step {
            use crate::utils::devsetup::SetupIndyMocks;

            use super::*;

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_init() {
                let _setup = SetupIndyMocks::init();

                let did_exchange_sm = inviter_sm().await;
                assert_match!(InviterFullState::Initial(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_connect_message_from_null_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = inviter_sm().await;

                let routing_keys: Vec<String> = vec!["verkey123".into()];
                let service_endpoint = String::from("https://example.org/agent");
                did_exchange_sm = did_exchange_sm.handle_connect(routing_keys, service_endpoint).unwrap();

                assert_match!(InviterFullState::Invited(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_other_messages_from_null_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = inviter_sm().await;

                did_exchange_sm = did_exchange_sm
                    .handle_ack(_dummy_wallet_handle(), _ack(), _send_message)
                    .await
                    .unwrap();
                assert_match!(InviterFullState::Initial(_), did_exchange_sm.state);

                did_exchange_sm = did_exchange_sm.handle_problem_report(_problem_report()).unwrap();
                assert_match!(InviterFullState::Initial(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_exchange_request_message_from_invited_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = inviter_sm().await.to_inviter_invited_state();

                let new_pairwise_info = PairwiseInfo {
                    pw_did: "AC3Gx1RoAz8iYVcfY47gjJ".to_string(),
                    pw_vk: "verkey456".to_string(),
                };
                let new_routing_keys: Vec<String> = vec!["AC3Gx1RoAz8iYVcfY47gjJ".into()];
                let new_service_endpoint = String::from("https://example.org/agent");
                did_exchange_sm = did_exchange_sm
                    .handle_connection_request(
                        _dummy_wallet_handle(),
                        _request(),
                        &new_pairwise_info,
                        new_routing_keys,
                        new_service_endpoint,
                        _send_message,
                    )
                    .await
                    .unwrap();
                did_exchange_sm = did_exchange_sm
                    .handle_send_response(_dummy_wallet_handle(), &_send_message)
                    .await
                    .unwrap();
                assert_match!(InviterFullState::Responded(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_invalid_exchange_request_message_from_invited_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = inviter_sm().await.to_inviter_invited_state();

                let mut request = _request();
                request.connection.did_doc = DidDoc::default();

                let new_pairwise_info = PairwiseInfo {
                    pw_did: "AC3Gx1RoAz8iYVcfY47gjJ".to_string(),
                    pw_vk: "verkey456".to_string(),
                };
                let new_routing_keys: Vec<String> = vec!["AC3Gx1RoAz8iYVcfY47gjJ".into()];
                let new_service_endpoint = String::from("https://example.org/agent");
                did_exchange_sm = did_exchange_sm
                    .handle_connection_request(
                        _dummy_wallet_handle(),
                        request,
                        &new_pairwise_info,
                        new_routing_keys,
                        new_service_endpoint,
                        _send_message,
                    )
                    .await
                    .unwrap();

                assert_match!(InviterFullState::Initial(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_problem_report_message_from_invited_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = inviter_sm().await.to_inviter_invited_state();

                did_exchange_sm = did_exchange_sm.handle_problem_report(_problem_report()).unwrap();

                assert_match!(InviterFullState::Initial(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_other_message_from_null_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = inviter_sm().await.to_inviter_invited_state();

                let routing_keys: Vec<String> = vec!["verkey123".into()];
                let service_endpoint = String::from("https://example.org/agent");
                did_exchange_sm = did_exchange_sm.handle_connect(routing_keys, service_endpoint).unwrap();
                assert_match!(InviterFullState::Invited(_), did_exchange_sm.state);

                did_exchange_sm = did_exchange_sm
                    .handle_ack(_dummy_wallet_handle(), _ack(), _send_message)
                    .await
                    .unwrap();
                assert_match!(InviterFullState::Invited(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_ack_message_from_responded_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = inviter_sm().await.to_inviter_responded_state().await;

                did_exchange_sm = did_exchange_sm
                    .handle_ack(_dummy_wallet_handle(), _ack(), _send_message)
                    .await
                    .unwrap();

                assert_match!(InviterFullState::Completed(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_ping_message_from_responded_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = inviter_sm().await.to_inviter_responded_state().await;

                did_exchange_sm = did_exchange_sm
                    .handle_ping(_dummy_wallet_handle(), _ping(), _send_message)
                    .await
                    .unwrap();

                assert_match!(InviterFullState::Completed(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_problem_report_message_from_responded_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = inviter_sm().await.to_inviter_responded_state().await;

                did_exchange_sm = did_exchange_sm.handle_problem_report(_problem_report()).unwrap();

                assert_match!(InviterFullState::Initial(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_other_messages_from_responded_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = inviter_sm().await.to_inviter_responded_state().await;

                let routing_keys: Vec<String> = vec!["verkey123".into()];
                let service_endpoint = String::from("https://example.org/agent");
                did_exchange_sm = did_exchange_sm.handle_connect(routing_keys, service_endpoint).unwrap();

                assert_match!(InviterFullState::Responded(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_messages_from_completed_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = inviter_sm().await.to_inviter_completed_state().await;

                // Ping
                did_exchange_sm = did_exchange_sm
                    .handle_ping(_dummy_wallet_handle(), _ping(), _send_message)
                    .await
                    .unwrap();
                assert_match!(InviterFullState::Completed(_), did_exchange_sm.state);

                // Disclose
                assert!(did_exchange_sm.get_remote_protocols().is_none());

                did_exchange_sm = did_exchange_sm.handle_disclose(_disclose()).unwrap();
                assert_match!(InviterFullState::Completed(_), did_exchange_sm.state);

                assert!(did_exchange_sm.get_remote_protocols().is_some());

                // ignore
                // Ack
                did_exchange_sm = did_exchange_sm
                    .handle_ack(_dummy_wallet_handle(), _ack(), _send_message)
                    .await
                    .unwrap();
                assert_match!(InviterFullState::Completed(_), did_exchange_sm.state);

                // Problem Report
                did_exchange_sm = did_exchange_sm.handle_problem_report(_problem_report()).unwrap();
                assert_match!(InviterFullState::Completed(_), did_exchange_sm.state);
            }
        }

        mod find_message_to_handle {
            use crate::messages::discovery::query::test_utils::_query_string;
            use crate::utils::devsetup::SetupIndyMocks;

            use super::*;

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_find_message_to_handle_from_null_state() {
                let _setup = SetupIndyMocks::init();

                let connection = inviter_sm().await;

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
            async fn test_find_message_to_handle_from_invited_state() {
                let _setup = SetupIndyMocks::init();

                let connection = inviter_sm().await.to_inviter_invited_state();

                // Connection Request
                {
                    let messages = map!(
                        "key_1".to_string() => A2AMessage::Ping(_ping()),
                        "key_2".to_string() => A2AMessage::ConnectionRequest(_request()),
                        "key_3".to_string() => A2AMessage::ConnectionResponse(_signed_response())
                    );

                    let (uid, message) = connection.find_message_to_update_state(messages).unwrap();
                    assert_eq!("key_2", uid);
                    assert_match!(A2AMessage::ConnectionRequest(_), message);
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

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_find_message_to_handle_from_responded_state() {
                let _setup = SetupIndyMocks::init();

                let connection = inviter_sm().await.to_inviter_responded_state().await;

                // Ping
                {
                    let messages = map!(
                        "key_1".to_string() => A2AMessage::Ping(_ping()),
                        "key_2".to_string() => A2AMessage::ConnectionRequest(_request()),
                        "key_3".to_string() => A2AMessage::ConnectionResponse(_signed_response())
                    );

                    let (uid, message) = connection.find_message_to_update_state(messages).unwrap();
                    assert_eq!("key_1", uid);
                    assert_match!(A2AMessage::Ping(_), message);
                }

                // Ack
                {
                    let messages = map!(
                        "key_1".to_string() => A2AMessage::ConnectionRequest(_request()),
                        "key_2".to_string() => A2AMessage::Ack(_ack()),
                        "key_3".to_string() => A2AMessage::ConnectionResponse(_signed_response())
                    );

                    let (uid, message) = connection.find_message_to_update_state(messages).unwrap();
                    assert_eq!("key_2", uid);
                    assert_match!(A2AMessage::Ack(_), message);
                }

                // Connection Problem Report
                {
                    let messages = map!(
                        "key_1".to_string() => A2AMessage::ConnectionRequest(_request()),
                        "key_2".to_string() => A2AMessage::ConnectionProblemReport(_problem_report())
                    );

                    let (uid, message) = connection.find_message_to_update_state(messages).unwrap();
                    assert_eq!("key_2", uid);
                    assert_match!(A2AMessage::ConnectionProblemReport(_), message);
                }

                // No messages
                {
                    let messages = map!(
                        "key_1".to_string() => A2AMessage::ConnectionRequest(_request()),
                        "key_2".to_string() => A2AMessage::ConnectionResponse(_signed_response())
                    );

                    assert!(connection.find_message_to_update_state(messages).is_none());
                }
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_should_not_find_processable_message_in_complete_state() {
                let _setup = SetupIndyMocks::init();
                let connection = inviter_sm().await.to_inviter_completed_state().await;
                {
                    let messages = map!(
                        "key_1".to_string() => A2AMessage::ConnectionRequest(_request()),
                        "key_2".to_string() => A2AMessage::ConnectionResponse(_signed_response()),
                        "key_3".to_string() => A2AMessage::Query(_query()),
                        "key_3".to_string() => A2AMessage::Ping(_ping()),
                        "key_3".to_string() => A2AMessage::Ack(_ack()),
                        "key_3".to_string() => A2AMessage::Disclose(_disclose())
                    );

                    assert!(connection.find_message_to_update_state(messages).is_none())
                }
            }
        }

        mod get_state {
            use super::*;

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_get_state() {
                let _setup = SetupMocks::init();

                assert_eq!(InviterState::Initial, inviter_sm().await.get_state());
                assert_eq!(
                    InviterState::Invited,
                    inviter_sm().await.to_inviter_invited_state().get_state()
                );
                assert_eq!(
                    InviterState::Responded,
                    inviter_sm().await.to_inviter_responded_state().await.get_state()
                );
                assert_eq!(
                    InviterState::Completed,
                    inviter_sm().await.to_inviter_completed_state().await.get_state()
                );
            }
        }
    }
}
