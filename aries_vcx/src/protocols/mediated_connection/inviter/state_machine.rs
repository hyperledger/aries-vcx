use std::clone::Clone;
use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use diddoc::aries::diddoc::AriesDidDoc;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use messages::decorators::thread::Thread;
use messages::decorators::timing::Timing;
use messages::msg_fields::protocols::connection::invitation::{
    Invitation, PairwiseInvitation, PairwiseInvitationContent, PwInvitationDecorators,
};
use messages::msg_fields::protocols::connection::problem_report::{
    ProblemReport, ProblemReportContent, ProblemReportDecorators,
};
use messages::msg_fields::protocols::connection::request::Request;
use messages::msg_fields::protocols::connection::response::{Response, ResponseContent, ResponseDecorators};
use messages::msg_fields::protocols::connection::Connection;
use messages::msg_fields::protocols::discover_features::disclose::Disclose;
use messages::msg_fields::protocols::discover_features::query::QueryContent;
use messages::msg_fields::protocols::discover_features::ProtocolDescriptor;
use messages::msg_fields::protocols::trust_ping::TrustPing;
use messages::AriesMessage;
use url::Url;
use uuid::Uuid;

use crate::common::signing::sign_connection_response;
use crate::errors::error::prelude::*;
use crate::handlers::util::{verify_thread_id, AnyInvitation};
use crate::protocols::mediated_connection::inviter::states::completed::CompletedState;
use crate::protocols::mediated_connection::inviter::states::initial::InitialState;
use crate::protocols::mediated_connection::inviter::states::invited::InvitedState;
use crate::protocols::mediated_connection::inviter::states::requested::RequestedState;
use crate::protocols::mediated_connection::inviter::states::responded::RespondedState;
use crate::protocols::mediated_connection::pairwise_info::PairwiseInfo;
use crate::protocols::SendClosureConnection;

#[derive(Clone, Serialize, Deserialize)]
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
    Completed(CompletedState),
}

#[derive(Debug, PartialEq, Eq)]
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
            thread_id: Uuid::new_v4().to_string(),
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

    pub fn their_did_doc(&self) -> Option<AriesDidDoc> {
        match self.state {
            InviterFullState::Initial(_) => None,
            InviterFullState::Invited(ref _state) => None,
            InviterFullState::Requested(ref state) => Some(state.did_doc.clone()),
            InviterFullState::Responded(ref state) => Some(state.did_doc.clone()),
            InviterFullState::Completed(ref state) => Some(state.did_doc.clone()),
        }
    }

    pub fn get_invitation(&self) -> Option<&AnyInvitation> {
        match self.state {
            InviterFullState::Invited(ref state) => Some(&state.invitation),
            _ => None,
        }
    }

    pub fn find_message_to_update_state(
        &self,
        messages: HashMap<String, AriesMessage>,
    ) -> Option<(String, AriesMessage)> {
        for (uid, message) in messages {
            if self.can_progress_state(&message) {
                return Some((uid, message));
            }
        }
        None
    }

    pub fn get_protocols(&self) -> Vec<ProtocolDescriptor> {
        let query = QueryContent::new("*".to_owned());
        query.lookup()
    }

    pub fn get_remote_protocols(&self) -> Option<Vec<ProtocolDescriptor>> {
        match self.state {
            InviterFullState::Completed(ref state) => state.protocols.clone(),
            _ => None,
        }
    }

    pub fn is_in_null_state(&self) -> bool {
        matches!(self.state, InviterFullState::Initial(_))
    }

    pub fn is_in_final_state(&self) -> bool {
        matches!(self.state, InviterFullState::Completed(_))
    }

    pub fn remote_did(&self) -> VcxResult<String> {
        self.their_did_doc()
            .map(|did_doc: AriesDidDoc| did_doc.id)
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Remote Connection DID is not set",
            ))
    }

    pub fn remote_vk(&self) -> VcxResult<String> {
        let did_did = self.their_did_doc().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::NotReady,
            "Counterparty diddoc is not available.",
        ))?;
        did_did
            .recipient_keys()?
            .get(0)
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Can't resolve recipient key from the counterparty diddoc.",
            ))
            .map(|s| s.to_string())
    }

    pub fn can_progress_state(&self, message: &AriesMessage) -> bool {
        match self.state {
            InviterFullState::Invited(_) => matches!(
                message,
                AriesMessage::Connection(Connection::Request(_))
                    | AriesMessage::Connection(Connection::ProblemReport(_))
            ),
            InviterFullState::Responded(_) => matches!(
                message,
                AriesMessage::Notification(_)
                    | AriesMessage::TrustPing(TrustPing::Ping(_))
                    | AriesMessage::Connection(Connection::ProblemReport(_))
            ),
            _ => false,
        }
    }

    pub fn create_invitation(self, routing_keys: Vec<String>, service_endpoint: Url) -> VcxResult<Self> {
        let state = match self.state {
            InviterFullState::Initial(state) => {
                let id = self.thread_id.clone();
                let content = PairwiseInvitationContent::new(
                    self.source_id.clone(),
                    vec![self.pairwise_info.pw_vk.clone()],
                    routing_keys,
                    service_endpoint,
                );

                let decorators = PwInvitationDecorators::default();

                let invite = PairwiseInvitation::with_decorators(id, content, decorators);

                let invitation = AnyInvitation::Con(Invitation::Pairwise(invite));

                InviterFullState::Invited((state, invitation).into())
            }
            _ => self.state.clone(),
        };
        Ok(Self { state, ..self })
    }

    pub async fn handle_connection_request(
        self,
        wallet: Arc<dyn BaseWallet>,
        request: Request,
        new_pairwise_info: &PairwiseInfo,
        new_routing_keys: Vec<String>,
        new_service_endpoint: Url,
        send_message: SendClosureConnection,
    ) -> VcxResult<Self> {
        if !matches!(self.state, InviterFullState::Initial(_)) {
            verify_thread_id(&self.get_thread_id(), &request.clone().into())?;
        };

        let state = match self.state {
            InviterFullState::Invited(_) | InviterFullState::Initial(_) => {
                if let Err(err) = request.content.connection.did_doc.validate() {
                    let mut content = ProblemReportContent::default();
                    content.explain = Some(err.to_string());

                    let mut decorators = ProblemReportDecorators::new(Thread::new(self.thread_id.clone()));
                    let mut timing = Timing::default();
                    timing.out_time = Some(Utc::now());
                    decorators.timing = Some(timing);

                    let problem_report =
                        ProblemReport::with_decorators(Uuid::new_v4().to_string(), content, decorators);

                    let sender_vk = self.pairwise_info().pw_vk.clone();
                    let did_doc = request.content.connection.did_doc.clone();

                    send_message(problem_report.clone().into(), sender_vk, did_doc)
                        .await
                        .ok();
                    return Ok(Self {
                        state: InviterFullState::Initial((problem_report).into()),
                        ..self
                    });
                };
                let signed_response = self
                    .build_response(
                        &wallet,
                        &request,
                        new_pairwise_info,
                        new_routing_keys,
                        new_service_endpoint,
                    )
                    .await?;
                InviterFullState::Requested((request, signed_response).into())
            }
            _ => self.state,
        };
        Ok(Self {
            pairwise_info: new_pairwise_info.to_owned(),
            state,
            ..self
        })
    }

    pub fn handle_problem_report(self, problem_report: ProblemReport) -> VcxResult<Self> {
        let state = match self.state {
            InviterFullState::Responded(_) => InviterFullState::Initial((problem_report).into()),
            InviterFullState::Invited(_) => InviterFullState::Initial((problem_report).into()),
            _ => self.state,
        };
        Ok(Self { state, ..self })
    }

    pub async fn handle_send_response(self, send_message: SendClosureConnection) -> VcxResult<Self> {
        let state = match self.state {
            InviterFullState::Requested(state) => {
                send_message(
                    state.signed_response.clone().into(),
                    self.pairwise_info.pw_vk.clone(),
                    state.did_doc.clone(),
                )
                .await?;
                InviterFullState::Responded(state.into())
            }
            _ => self.state,
        };
        Ok(Self { state, ..self })
    }

    pub fn handle_disclose(self, disclose: Disclose) -> VcxResult<Self> {
        let state = match self.state {
            InviterFullState::Completed(state) => {
                InviterFullState::Completed((state, disclose.content.protocols).into())
            }
            _ => self.state,
        };
        Ok(Self { state, ..self })
    }

    pub async fn handle_confirmation_message(self, msg: &AriesMessage) -> VcxResult<Self> {
        verify_thread_id(&self.get_thread_id(), msg)?;
        match self.state {
            InviterFullState::Responded(state) => Ok(Self {
                state: InviterFullState::Completed(state.into()),
                ..self
            }),
            _ => Ok(self),
        }
    }

    pub fn get_thread_id(&self) -> String {
        self.thread_id.clone()
    }

    async fn build_response(
        &self,
        wallet: &Arc<dyn BaseWallet>,
        request: &Request,
        new_pairwise_info: &PairwiseInfo,
        new_routing_keys: Vec<String>,
        new_service_endpoint: Url,
    ) -> VcxResult<Response> {
        match &self.state {
            InviterFullState::Invited(_) | InviterFullState::Initial(_) => {
                let new_recipient_keys = vec![new_pairwise_info.pw_vk.clone()];

                let id = Uuid::new_v4().to_string();

                let con_sig =
                    sign_connection_response(wallet, &self.pairwise_info.pw_vk, &request.content.connection).await?;

                let content = ResponseContent::new(con_sig);

                let thread_id = request
                    .decorators
                    .thread
                    .as_ref()
                    .map(|t| t.thid.as_str())
                    .unwrap_or(request.id.as_str());

                let mut decorators = ResponseDecorators::new(Thread::new(thread_id.to_owned()));
                let mut timing = Timing::default();
                timing.out_time = Some(Utc::now());
                decorators.timing = Some(timing);

                Ok(Response::with_decorators(id, content, decorators))
            }
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Building connection ack in current state is not allowed",
            )),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use messages::concepts::ack::test_utils::_ack;
    use messages::protocols::connection::problem_report::unit_tests::_problem_report;
    use messages::protocols::connection::request::unit_tests::_request;
    use messages::protocols::connection::response::test_utils::_signed_response;
    use messages::protocols::discovery::disclose::test_utils::_disclose;
    use messages::protocols::discovery::query::test_utils::_query;
    use messages::protocols::trust_ping::ping::unit_tests::_ping;

    use crate::test::source_id;
    use crate::utils::devsetup::SetupMocks;

    use super::*;

    pub mod inviter {
        use crate::common::test_utils::mock_profile;

        use super::*;

        fn _send_message() -> SendClosureConnection {
            Box::new(|_: A2AMessage, _: String, _: AriesDidDoc| Box::pin(async { VcxResult::Ok(()) }))
        }

        pub async fn inviter_sm() -> SmConnectionInviter {
            let pairwise_info = PairwiseInfo::create(&mock_profile().inject_wallet()).await.unwrap();
            SmConnectionInviter::new(&source_id(), pairwise_info)
        }

        impl SmConnectionInviter {
            fn to_inviter_invited_state(mut self) -> SmConnectionInviter {
                let routing_keys: Vec<String> = vec!["verkey123".into()];
                let service_endpoint = String::from("https://example.org/agent");
                self = self.create_invitation(routing_keys, service_endpoint).unwrap();
                self
            }

            async fn to_inviter_requested_state(mut self) -> SmConnectionInviter {
                self = self.to_inviter_invited_state();
                let new_pairwise_info = PairwiseInfo::create(&mock_profile().inject_wallet()).await.unwrap();
                let new_routing_keys: Vec<String> = vec!["verkey456".into()];
                let new_service_endpoint = String::from("https://example.org/agent");
                self = self
                    .handle_connection_request(
                        mock_profile().inject_wallet(),
                        _request(),
                        &new_pairwise_info,
                        new_routing_keys,
                        new_service_endpoint,
                        _send_message(),
                    )
                    .await
                    .unwrap();
                self = self.handle_send_response(_send_message()).await.unwrap();
                self
            }

            async fn to_inviter_responded_state(mut self) -> SmConnectionInviter {
                self = self.to_inviter_requested_state().await;
                self = self.handle_send_response(_send_message()).await.unwrap();
                self
            }

            // todo: try reuse to_inviter_responded_state
            async fn to_inviter_completed_state(mut self) -> SmConnectionInviter {
                self = self.to_inviter_responded_state().await;
                self = self
                    .handle_confirmation_message(&A2AMessage::Ack(_ack()))
                    .await
                    .unwrap();
                self
            }
        }

        mod build_messages {
            use messages::a2a::MessageId;

            use crate::utils::devsetup::was_in_past;

            use super::*;

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_build_connection_response_msg() {
                let _setup = SetupMocks::init();

                let mut inviter = inviter_sm().await;
                inviter = inviter.to_inviter_invited_state();
                let new_pairwise_info = PairwiseInfo {
                    pw_did: "AC3Gx1RoAz8iYVcfY47gjJ".to_string(),
                    pw_vk: "verkey456".to_string(),
                };
                let new_routing_keys: Vec<String> = vec!["AC3Gx1RoAz8iYVcfY47gjJ".into()];
                let new_service_endpoint = String::from("https://example.org/agent");
                let msg = inviter
                    .build_response(
                        &mock_profile().inject_wallet(),
                        &_request(),
                        &new_pairwise_info,
                        new_routing_keys,
                        new_service_endpoint,
                    )
                    .await
                    .unwrap();

                assert_eq!(msg.id, MessageId::default());
                assert!(was_in_past(
                    &msg.timing.unwrap().out_time.unwrap(),
                    chrono::Duration::milliseconds(100),
                )
                .unwrap());
            }
        }

        mod get_thread_id {
            use messages::concepts::ack::test_utils::_ack_random_thread;

            use super::*;

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn ack_fails_with_incorrect_thread_id() {
                let _setup = SetupMocks::init();

                let inviter = inviter_sm().await.to_inviter_responded_state().await;

                assert!(inviter
                    .handle_confirmation_message(&A2AMessage::Ack(_ack_random_thread()))
                    .await
                    .is_err())
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
                did_exchange_sm = did_exchange_sm
                    .create_invitation(routing_keys, service_endpoint)
                    .unwrap();

                assert_match!(InviterFullState::Invited(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_other_messages_from_null_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = inviter_sm().await;

                did_exchange_sm = did_exchange_sm
                    .handle_confirmation_message(&A2AMessage::Ack(_ack()))
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
                        mock_profile().inject_wallet(),
                        _request(),
                        &new_pairwise_info,
                        new_routing_keys,
                        new_service_endpoint,
                        _send_message(),
                    )
                    .await
                    .unwrap();
                did_exchange_sm = did_exchange_sm.handle_send_response(_send_message()).await.unwrap();
                assert_match!(InviterFullState::Responded(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_invalid_exchange_request_message_from_invited_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = inviter_sm().await.to_inviter_invited_state();

                let mut request = _request();
                request.connection.did_doc = AriesDidDoc::default();

                let new_pairwise_info = PairwiseInfo {
                    pw_did: "AC3Gx1RoAz8iYVcfY47gjJ".to_string(),
                    pw_vk: "verkey456".to_string(),
                };
                let new_routing_keys: Vec<String> = vec!["AC3Gx1RoAz8iYVcfY47gjJ".into()];
                let new_service_endpoint = String::from("https://example.org/agent");
                did_exchange_sm = did_exchange_sm
                    .handle_connection_request(
                        mock_profile().inject_wallet(),
                        request,
                        &new_pairwise_info,
                        new_routing_keys,
                        new_service_endpoint,
                        _send_message(),
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
                did_exchange_sm = did_exchange_sm
                    .create_invitation(routing_keys, service_endpoint)
                    .unwrap();
                assert_match!(InviterFullState::Invited(_), did_exchange_sm.state);

                did_exchange_sm = did_exchange_sm
                    .handle_confirmation_message(&A2AMessage::Ack(_ack()))
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
                    .handle_confirmation_message(&A2AMessage::Ack(_ack()))
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
                    .handle_confirmation_message(&A2AMessage::Ping(_ping()))
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
                did_exchange_sm = did_exchange_sm
                    .create_invitation(routing_keys, service_endpoint)
                    .unwrap();

                assert_match!(InviterFullState::Responded(_), did_exchange_sm.state);
            }

            #[tokio::test]
            #[cfg(feature = "general_test")]
            async fn test_did_exchange_handle_messages_from_completed_state() {
                let _setup = SetupIndyMocks::init();

                let mut did_exchange_sm = inviter_sm().await.to_inviter_completed_state().await;

                // Ping
                did_exchange_sm = did_exchange_sm
                    .handle_confirmation_message(&A2AMessage::Ping(_ping()))
                    .await
                    .unwrap();
                assert_match!(InviterFullState::Completed(_), did_exchange_sm.state);

                // Ack
                did_exchange_sm = did_exchange_sm
                    .handle_confirmation_message(&A2AMessage::Ack(_ack()))
                    .await
                    .unwrap();
                assert_match!(InviterFullState::Completed(_), did_exchange_sm.state);

                // Disclose
                assert!(did_exchange_sm.get_remote_protocols().is_none());

                did_exchange_sm = did_exchange_sm.handle_disclose(_disclose()).unwrap();
                assert_match!(InviterFullState::Completed(_), did_exchange_sm.state);

                assert!(did_exchange_sm.get_remote_protocols().is_some());

                // Problem Report
                did_exchange_sm = did_exchange_sm.handle_problem_report(_problem_report()).unwrap();
                assert_match!(InviterFullState::Completed(_), did_exchange_sm.state);
            }
        }

        mod find_message_to_handle {
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
