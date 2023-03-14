use std::{clone::Clone, collections::HashMap, sync::Arc};

use messages::{
    a2a::{protocol_registry::ProtocolRegistry, A2AMessage},
    concepts::ack::Ack,
    diddoc::aries::diddoc::AriesDidDoc,
    protocols::{
        connection::{
            invite::Invitation,
            problem_report::{ProblemCode, ProblemReport},
            request::Request,
            response::SignedResponse,
        },
        discovery::disclose::{Disclose, ProtocolDescriptor},
    },
};

use crate::{
    common::signing::decode_signed_connection_response,
    errors::error::prelude::*,
    handlers::util::verify_thread_id,
    plugins::wallet::base_wallet::BaseWallet,
    protocols::{
        mediated_connection::{
            invitee::states::{
                completed::CompletedState, initial::InitialState, invited::InvitedState, requested::RequestedState,
                responded::RespondedState,
            },
            pairwise_info::PairwiseInfo,
        },
        SendClosureConnection,
    },
};

#[derive(Clone, Serialize, Deserialize)]
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
    Completed(CompletedState),
}

#[derive(Debug, PartialEq, Eq)]
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
    pub fn new(source_id: &str, pairwise_info: PairwiseInfo, did_doc: AriesDidDoc) -> Self {
        SmConnectionInvitee {
            source_id: source_id.to_string(),
            thread_id: String::new(),
            state: InviteeFullState::Initial(InitialState::new(None, Some(did_doc))),
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
        matches!(self.state, InviteeFullState::Initial(_))
    }

    pub fn is_in_final_state(&self) -> bool {
        matches!(self.state, InviteeFullState::Completed(_))
    }

    pub fn their_did_doc(&self) -> Option<AriesDidDoc> {
        match self.state {
            InviteeFullState::Initial(ref state) => state.did_doc.clone(),
            InviteeFullState::Invited(ref state) => Some(state.did_doc.clone()),
            InviteeFullState::Requested(ref state) => Some(state.did_doc.clone()),
            InviteeFullState::Responded(ref state) => Some(state.did_doc.clone()),
            InviteeFullState::Completed(ref state) => Some(state.did_doc.clone()),
        }
    }

    pub fn bootstrap_did_doc(&self) -> Option<AriesDidDoc> {
        match self.state {
            InviteeFullState::Initial(ref state) => state.did_doc.clone(),
            InviteeFullState::Invited(ref state) => Some(state.did_doc.clone()),
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

    pub fn can_progress_state(&self, message: &A2AMessage) -> bool {
        match self.state {
            InviteeFullState::Requested(_) => matches!(
                message,
                A2AMessage::ConnectionResponse(_) | A2AMessage::ConnectionProblemReport(_)
            ),
            _ => false,
        }
    }

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
                    .set_service_endpoint(service_endpoint)
                    .set_keys(recipient_keys, routing_keys)
                    .set_out_time();
                let request_id = request.id.0.clone();
                let (request, thread_id) = match &state.invitation {
                    Invitation::Public(_) => (
                        request
                            .set_parent_thread_id(&self.thread_id)
                            .set_thread_id_matching_id(),
                        request_id,
                    ),
                    Invitation::Pairwise(_) => (request.set_thread_id(&self.thread_id), self.get_thread_id()),
                    Invitation::OutOfBand(invite) => (
                        request.set_parent_thread_id(&invite.id.0).set_thread_id_matching_id(),
                        request_id,
                    ),
                };
                Ok((request, thread_id))
            }
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Building connection request in current state is not allowed",
            )),
        }
    }

    fn build_connection_ack_msg(&self) -> VcxResult<Ack> {
        match &self.state {
            InviteeFullState::Responded(_) => Ok(Ack::create().set_out_time().set_thread_id(&self.thread_id)),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Building connection ack in current state is not allowed",
            )),
        }
    }

    pub fn handle_invitation(self, invitation: Invitation) -> VcxResult<Self> {
        let Self { state, .. } = self;
        let thread_id = invitation.get_id().to_owned();
        let state = match state {
            InviteeFullState::Initial(state) => InviteeFullState::Invited(
                (
                    state.clone(),
                    invitation,
                    state.did_doc.ok_or(AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidState,
                        "Expected none None state.did_doc result given current state",
                    ))?,
                )
                    .into(),
            ),
            s => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    format!("Cannot handle inviation: not in Initial state, current state: {:?}", s),
                ));
            }
        };
        Ok(Self {
            state,
            thread_id,
            ..self
        })
    }

    pub async fn send_connection_request(
        self,
        routing_keys: Vec<String>,
        service_endpoint: String,
        send_message: SendClosureConnection,
    ) -> VcxResult<Self> {
        let (state, thread_id) = match self.state {
            InviteeFullState::Invited(ref state) => {
                let ddo = self.their_did_doc().ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Missing did doc",
                ))?;
                let (request, thread_id) = self.build_connection_request_msg(routing_keys, service_endpoint)?;
                send_message(request.to_a2a_message(), self.pairwise_info.pw_vk.clone(), ddo.clone()).await?;
                (
                    InviteeFullState::Requested((state.clone(), request, ddo).into()),
                    thread_id,
                )
            }
            _ => (self.state.clone(), self.get_thread_id()),
        };
        Ok(Self {
            state,
            thread_id,
            ..self
        })
    }

    pub async fn handle_connection_response(
        self,
        wallet: &Arc<dyn BaseWallet>,
        response: SignedResponse,
        send_message: SendClosureConnection,
    ) -> VcxResult<Self> {
        verify_thread_id(&self.get_thread_id(), &A2AMessage::ConnectionResponse(response.clone()))?;
        let state = match self.state {
            InviteeFullState::Requested(state) => {
                let remote_vk: String =
                    state
                        .did_doc
                        .recipient_keys()?
                        .get(0)
                        .cloned()
                        .ok_or(AriesVcxError::from_msg(
                            AriesVcxErrorKind::InvalidState,
                            "Cannot handle response: remote verkey not found",
                        ))?;

                match decode_signed_connection_response(wallet, response.clone(), &remote_vk).await {
                    Ok(response) => {
                        if !response.from_thread(&state.request.get_thread_id()) {
                            return Err(AriesVcxError::from_msg(
                                AriesVcxErrorKind::InvalidJson,
                                format!(
                                    "Cannot handle response: thread id does not match: {:?}",
                                    response.thread
                                ),
                            ));
                        }
                        InviteeFullState::Responded((state, response).into())
                    }
                    Err(err) => {
                        let problem_report = ProblemReport::create()
                            .set_problem_code(ProblemCode::ResponseProcessingError)
                            .set_explain(err.to_string())
                            .set_thread_id(&self.thread_id)
                            .set_out_time();
                        send_message(
                            problem_report.to_a2a_message(),
                            self.pairwise_info.pw_vk.clone(),
                            state.did_doc.clone(),
                        )
                        .await
                        .ok();
                        InviteeFullState::Initial((state.clone(), problem_report).into())
                    }
                }
            }
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

    pub async fn handle_send_ack(self, send_message: SendClosureConnection) -> VcxResult<Self> {
        let state = match self.state {
            InviteeFullState::Responded(ref state) => {
                let sender_vk = self.pairwise_info().pw_vk.clone();
                let did_doc = state.response.connection.did_doc.clone();
                send_message(self.build_connection_ack_msg()?.to_a2a_message(), sender_vk, did_doc).await?;
                InviteeFullState::Completed((state.clone()).into())
            }
            _ => self.state.clone(),
        };
        Ok(Self { state, ..self })
    }

    pub fn handle_problem_report(self, _problem_report: ProblemReport) -> VcxResult<Self> {
        let state = match self.state {
            InviteeFullState::Requested(_state) => InviteeFullState::Initial(InitialState::new(None, None)),
            InviteeFullState::Invited(_state) => InviteeFullState::Initial(InitialState::new(None, None)),
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
    use messages::{
        concepts::ack::test_utils::_ack,
        protocols::{
            connection::{
                invite::test_utils::_pairwise_invitation, problem_report::unit_tests::_problem_report,
                request::unit_tests::_request, response::test_utils::_signed_response,
            },
            discovery::disclose::test_utils::_disclose,
            trust_ping::ping::unit_tests::_ping,
        },
    };

    use super::*;
    use crate::{test::source_id, utils::devsetup::SetupMocks};

    pub mod invitee {

        use messages::{
            diddoc::aries::diddoc::test_utils::{_did_doc_inlined_recipient_keys, _service_endpoint},
            protocols::connection::response::{Response, SignedResponse},
        };

        use super::*;
        use crate::common::{signing::sign_connection_response, test_utils::mock_profile};

        fn _send_message() -> SendClosureConnection {
            Box::new(|_: A2AMessage, _: String, _: AriesDidDoc| Box::pin(async { Ok(()) }))
        }

        pub async fn invitee_sm() -> SmConnectionInvitee {
            let pairwise_info = PairwiseInfo::create(&mock_profile().inject_wallet()).await.unwrap();
            SmConnectionInvitee::new(&source_id(), pairwise_info, _did_doc_inlined_recipient_keys())
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
                    .send_connection_request(routing_keys, service_endpoint, _send_message())
                    .await
                    .unwrap();
                self
            }

            pub async fn to_invitee_completed_state(mut self) -> SmConnectionInvitee {
                let key = "GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL".to_string();
                self = self.to_invitee_requested_state().await;
                self = self
                    .handle_connection_response(
                        &mock_profile().inject_wallet(),
                        _response(&mock_profile().inject_wallet(), &key, &_request().id.0).await,
                        _send_message(),
                    )
                    .await
                    .unwrap();
                self = self.handle_send_ack(_send_message()).await.unwrap();
                self
            }
        }

        async fn _response(wallet: &Arc<dyn BaseWallet>, key: &str, thread_id: &str) -> SignedResponse {
            sign_connection_response(
                wallet,
                key,
                Response::default()
                    .set_service_endpoint(_service_endpoint())
                    .set_keys(vec![key.to_string()], vec![])
                    .set_thread_id(thread_id),
            )
            .await
            .unwrap()
        }

        async fn _response_1(wallet: &Arc<dyn BaseWallet>, key: &str) -> SignedResponse {
            sign_connection_response(
                wallet,
                key,
                Response::default()
                    .set_service_endpoint(_service_endpoint())
                    .set_keys(vec![key.to_string()], vec![])
                    .set_thread_id("testid_1"),
            )
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
            use messages::{a2a::MessageId, concepts::ack::AckStatus};

            use super::*;
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
                    msg.connection.did_doc.recipient_keys().unwrap(),
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
                    .handle_connection_response(
                        &mock_profile().inject_wallet(),
                        _response(&mock_profile().inject_wallet(), &recipient_key, &msg_request.id.0).await,
                        _send_message(),
                    )
                    .await
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
                    .send_connection_request(routing_keys, service_endpoint, _send_message())
                    .await
                    .unwrap();
                assert_match!(InviteeState::Requested, invitee.get_state());
                assert!(invitee
                    .handle_connection_response(
                        &mock_profile().inject_wallet(),
                        _response_1(&mock_profile().inject_wallet(), &key).await,
                        _send_message()
                    )
                    .await
                    .is_err());
            }
        }

        mod step {
            use super::*;
            use crate::utils::devsetup::SetupIndyMocks;

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
                    .send_connection_request(routing_keys, service_endpoint, _send_message())
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
                    .handle_connection_response(
                        &mock_profile().inject_wallet(),
                        _response(&mock_profile().inject_wallet(), key, &_request().id.0).await,
                        _send_message()
                    )
                    .await
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
                    .send_connection_request(routing_keys, service_endpoint, _send_message())
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
                    .handle_connection_response(
                        &mock_profile().inject_wallet(),
                        _response(&mock_profile().inject_wallet(), &key, &_request().id.0).await,
                        _send_message(),
                    )
                    .await
                    .unwrap();
                did_exchange_sm = did_exchange_sm.handle_send_ack(_send_message()).await.unwrap();

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

                did_exchange_sm = did_exchange_sm
                    .handle_connection_response(&mock_profile().inject_wallet(), signed_response, _send_message())
                    .await
                    .unwrap();
                did_exchange_sm = did_exchange_sm.handle_send_ack(_send_message()).await.unwrap();

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
                assert_match!(InviteeFullState::Completed(_), did_exchange_sm.state);

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
            use super::*;
            use crate::utils::devsetup::SetupIndyMocks;

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
