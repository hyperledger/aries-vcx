use std::collections::HashMap;

use api::VcxStateType;
use error::prelude::*;
use v3::handlers::connection::agent_info::AgentInfo;
use v3::handlers::connection::messages::DidExchangeMessages;
use v3::handlers::connection::state_machine::{ActorDidExchangeState, DidExchangeSM, DidExchangeState};
use v3::handlers::connection::states::null::NullState;
use v3::messages::a2a::A2AMessage;
use v3::messages::a2a::protocol_registry::ProtocolRegistry;
use v3::messages::ack::Ack;
use v3::messages::connection::did_doc::DidDoc;
use v3::messages::connection::invite::Invitation;
use v3::messages::connection::problem_report::{ProblemCode, ProblemReport};
use v3::messages::connection::request::Request;
use v3::messages::connection::response::{Response, SignedResponse};
use v3::messages::discovery::disclose::{Disclose, ProtocolDescriptor};
use v3::messages::discovery::query::Query;
use v3::messages::trust_ping::ping::Ping;
use v3::messages::trust_ping::ping_response::PingResponse;

impl DidExchangeSM {
    pub fn _build_inviter(source_id: &str) -> Self {
        DidExchangeSM {
            source_id: source_id.to_string(),
            state: ActorDidExchangeState::Inviter(DidExchangeState::Null(NullState {})),
            agent_info: AgentInfo::default(),
        }
    }

    pub fn inviter_can_handle_message(&self, inviter_state: &DidExchangeState, message: &A2AMessage) -> bool {
        match inviter_state {
            DidExchangeState::Invited(_) => {
                match message {
                    A2AMessage::ConnectionRequest(_) => {
                        debug!("Inviter received ConnectionRequest message");
                        true
                    }
                    A2AMessage::ConnectionProblemReport(_) => {
                        debug!("Inviter received ProblemReport message");
                        true
                    }
                    _ => {
                        debug!("Inviter received unexpected message: {:?}", message);
                        false
                    }
                }
            }
            DidExchangeState::Responded(_) => {
                match message {
                    A2AMessage::Ack(_) => {
                        debug!("Ack message received");
                        true
                    }
                    A2AMessage::Ping(_) => {
                        debug!("Ping message received");
                        true
                    }
                    A2AMessage::PingResponse(_) => {
                        debug!("PingResponse message received");
                        true
                    }
                    A2AMessage::ConnectionProblemReport(_) => {
                        debug!("ProblemReport message received");
                        true
                    }
                    _ => {
                        debug!("Unexpected message received in Responded state: {:?}", message);
                        false
                    }
                }
            }
            DidExchangeState::Completed(_) => {
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

    pub fn inviter_step(inviter_state: DidExchangeState, message: DidExchangeMessages, source_id: &str, mut agent_info: AgentInfo) -> VcxResult<(ActorDidExchangeState, AgentInfo)> {
        let new_state = match inviter_state {
            DidExchangeState::Null(state) => {
                match message {
                    DidExchangeMessages::Connect() => {
                        agent_info = agent_info.create_agent()?;

                        let invite: Invitation = Invitation::create()
                            .set_label(source_id.to_string())
                            .set_service_endpoint(agent_info.agency_endpoint()?)
                            .set_recipient_keys(agent_info.recipient_keys())
                            .set_routing_keys(agent_info.routing_keys()?);

                        ActorDidExchangeState::Inviter(DidExchangeState::Invited((state, invite).into()))
                    }
                    _ => {
                        ActorDidExchangeState::Inviter(DidExchangeState::Null(state))
                    }
                }
            }
            DidExchangeState::Invited(state) => {
                match message {
                    DidExchangeMessages::ExchangeRequestReceived(request) => {
                        match state.handle_connection_request(&request, &agent_info) {
                            Ok((response, new_agent_info)) => {
                                let prev_agent_info = agent_info.clone();
                                agent_info = new_agent_info;
                                ActorDidExchangeState::Inviter(DidExchangeState::Responded((state, request, response, prev_agent_info).into()))
                            }
                            Err(err) => {
                                let problem_report = ProblemReport::create()
                                    .set_problem_code(ProblemCode::RequestProcessingError)
                                    .set_explain(err.to_string())
                                    .set_thread_id(&request.id.0);

                                agent_info.send_message(&problem_report.to_a2a_message(), &request.connection.did_doc).ok(); // IS is possible?
                                ActorDidExchangeState::Inviter(DidExchangeState::Null((state, problem_report).into()))
                            }
                        }
                    }
                    DidExchangeMessages::ProblemReportReceived(problem_report) => {
                        ActorDidExchangeState::Inviter(DidExchangeState::Null((state, problem_report).into()))
                    }
                    _ => {
                        ActorDidExchangeState::Inviter(DidExchangeState::Invited(state))
                    }
                }
            }
            DidExchangeState::Requested(state) => {
                ActorDidExchangeState::Inviter(DidExchangeState::Requested(state))
            }
            DidExchangeState::Responded(state) => {
                match message {
                    DidExchangeMessages::AckReceived(ack) => {
                        ActorDidExchangeState::Inviter(DidExchangeState::Completed((state, ack).into()))
                    }
                    DidExchangeMessages::PingReceived(ping) => {
                        state.handle_ping(&ping, &agent_info)?;
                        ActorDidExchangeState::Inviter(DidExchangeState::Completed((state, ping).into()))
                    }
                    DidExchangeMessages::ProblemReportReceived(problem_report) => {
                        ActorDidExchangeState::Inviter(DidExchangeState::Null((state, problem_report).into()))
                    }
                    DidExchangeMessages::SendPing(comment) => {
                        let ping =
                            Ping::create()
                                .request_response()
                                .set_comment(comment);

                        agent_info.send_message(&ping.to_a2a_message(), &state.did_doc).ok();
                        ActorDidExchangeState::Inviter(DidExchangeState::Responded(state))
                    }
                    DidExchangeMessages::PingResponseReceived(ping_response) => {
                        ActorDidExchangeState::Inviter(DidExchangeState::Completed((state, ping_response).into()))
                    }
                    _ => {
                        ActorDidExchangeState::Inviter(DidExchangeState::Responded(state))
                    }
                }
            }
            DidExchangeState::Completed(state) => {
                ActorDidExchangeState::Inviter(state.handle_message(message, &agent_info)?)
            }
        };
        Ok((new_state, agent_info))
    }
}


#[cfg(test)]
pub mod test {
    use utils::devsetup::SetupAriesMocks;
    use v3::messages::ack::tests::_ack;
    use v3::messages::connection::invite::tests::_invitation;
    use v3::messages::connection::problem_report::tests::_problem_report;
    use v3::messages::connection::request::tests::_request;
    use v3::messages::connection::response::tests::_signed_response;
    use v3::messages::discovery::disclose::tests::_disclose;
    use v3::messages::discovery::query::tests::_query;
    use v3::messages::trust_ping::ping::tests::_ping;
    use v3::messages::trust_ping::ping_response::tests::_ping_response;
    use v3::test::setup::AgencyModeSetup;
    use v3::test::source_id;

    use super::*;

    pub mod inviter {
        use v3::handlers::connection::state_machine::Actor;

        use super::*;

        pub fn inviter_sm() -> DidExchangeSM {
            DidExchangeSM::new(Actor::Inviter, &source_id())
        }

        impl DidExchangeSM {
            fn to_inviter_invited_state(mut self) -> DidExchangeSM {
                self = self.step(DidExchangeMessages::Connect()).unwrap();
                self
            }

            fn to_inviter_responded_state(mut self) -> DidExchangeSM {
                self = self.step(DidExchangeMessages::Connect()).unwrap();
                self = self.step(DidExchangeMessages::ExchangeRequestReceived(_request())).unwrap();
                self
            }

            fn to_inviter_completed_state(mut self) -> DidExchangeSM {
                self = self.step(DidExchangeMessages::Connect()).unwrap();
                self = self.step(DidExchangeMessages::ExchangeRequestReceived(_request())).unwrap();
                self = self.step(DidExchangeMessages::AckReceived(_ack())).unwrap();
                self
            }
        }

        mod new {
            use super::*;

            #[test]
            #[cfg(feature = "general_test")]
            fn test_inviter_new() {
                let _setup = SetupAriesMocks::init();

                let inviter_sm = inviter_sm();

                assert_match!(ActorDidExchangeState::Inviter(DidExchangeState::Null(_)), inviter_sm.state);
                assert_eq!(source_id(), inviter_sm.source_id());
            }
        }

        mod step {
            use super::*;

            #[test]
            #[cfg(feature = "general_test")]
            fn test_did_exchange_init() {
                let _setup = AgencyModeSetup::init();

                let did_exchange_sm = inviter_sm();
                assert_match!(ActorDidExchangeState::Inviter(DidExchangeState::Null(_)), did_exchange_sm.state);
            }

            #[test]
            #[cfg(feature = "general_test")]
            fn test_did_exchange_handle_connect_message_from_null_state() {
                let _setup = AgencyModeSetup::init();

                let mut did_exchange_sm = inviter_sm();

                did_exchange_sm = did_exchange_sm.step(DidExchangeMessages::Connect()).unwrap();

                assert_match!(ActorDidExchangeState::Inviter(DidExchangeState::Invited(_)), did_exchange_sm.state);
            }

            #[test]
            #[cfg(feature = "general_test")]
            fn test_did_exchange_handle_other_messages_from_null_state() {
                let _setup = AgencyModeSetup::init();

                let mut did_exchange_sm = inviter_sm();

                did_exchange_sm = did_exchange_sm.step(DidExchangeMessages::AckReceived(_ack())).unwrap();
                assert_match!(ActorDidExchangeState::Inviter(DidExchangeState::Null(_)), did_exchange_sm.state);

                did_exchange_sm = did_exchange_sm.step(DidExchangeMessages::ProblemReportReceived(_problem_report())).unwrap();
                assert_match!(ActorDidExchangeState::Inviter(DidExchangeState::Null(_)), did_exchange_sm.state);
            }

            #[test]
            #[cfg(feature = "general_test")]
            fn test_did_exchange_handle_exchange_request_message_from_invited_state() {
                let _setup = AgencyModeSetup::init();

                let mut did_exchange_sm = inviter_sm().to_inviter_invited_state();

                did_exchange_sm = did_exchange_sm.step(DidExchangeMessages::ExchangeRequestReceived(_request())).unwrap();
                assert_match!(ActorDidExchangeState::Inviter(DidExchangeState::Responded(_)), did_exchange_sm.state);
            }

            #[test]
            #[cfg(feature = "general_test")]
            fn test_did_exchange_handle_invalid_exchange_request_message_from_invited_state() {
                let _setup = AgencyModeSetup::init();

                let mut did_exchange_sm = inviter_sm().to_inviter_invited_state();

                let mut request = _request();
                request.connection.did_doc = DidDoc::default();

                did_exchange_sm = did_exchange_sm.step(DidExchangeMessages::ExchangeRequestReceived(request)).unwrap();

                assert_match!(ActorDidExchangeState::Inviter(DidExchangeState::Null(_)), did_exchange_sm.state);
            }

            #[test]
            #[cfg(feature = "general_test")]
            fn test_did_exchange_handle_problem_report_message_from_invited_state() {
                let _setup = AgencyModeSetup::init();

                let mut did_exchange_sm = inviter_sm().to_inviter_invited_state();

                did_exchange_sm = did_exchange_sm.step(DidExchangeMessages::ProblemReportReceived(_problem_report())).unwrap();

                assert_match!(ActorDidExchangeState::Inviter(DidExchangeState::Null(_)), did_exchange_sm.state);
            }

            #[test]
            #[cfg(feature = "general_test")]
            fn test_did_exchange_handle_other_messages_from_invited_state() {
                let _setup = AgencyModeSetup::init();

                let mut did_exchange_sm = inviter_sm().to_inviter_invited_state();

                did_exchange_sm = did_exchange_sm.step(DidExchangeMessages::Connect()).unwrap();
                assert_match!(ActorDidExchangeState::Inviter(DidExchangeState::Invited(_)), did_exchange_sm.state);

                did_exchange_sm = did_exchange_sm.step(DidExchangeMessages::AckReceived(_ack())).unwrap();
                assert_match!(ActorDidExchangeState::Inviter(DidExchangeState::Invited(_)), did_exchange_sm.state);
            }

            #[test]
            #[cfg(feature = "general_test")]
            fn test_did_exchange_handle_ack_message_from_responded_state() {
                let _setup = AgencyModeSetup::init();

                let mut did_exchange_sm = inviter_sm().to_inviter_responded_state();

                did_exchange_sm = did_exchange_sm.step(DidExchangeMessages::AckReceived(_ack())).unwrap();


                assert_match!(ActorDidExchangeState::Inviter(DidExchangeState::Completed(_)), did_exchange_sm.state);
            }

            #[test]
            #[cfg(feature = "general_test")]
            fn test_did_exchange_handle_ping_message_from_responded_state() {
                let _setup = AgencyModeSetup::init();

                let mut did_exchange_sm = inviter_sm().to_inviter_responded_state();

                did_exchange_sm = did_exchange_sm.step(DidExchangeMessages::PingReceived(_ping())).unwrap();

                assert_match!(ActorDidExchangeState::Inviter(DidExchangeState::Completed(_)), did_exchange_sm.state);
            }

            #[test]
            #[cfg(feature = "general_test")]
            fn test_did_exchange_handle_problem_report_message_from_responded_state() {
                let _setup = AgencyModeSetup::init();

                let mut did_exchange_sm = inviter_sm().to_inviter_responded_state();

                did_exchange_sm = did_exchange_sm.step(DidExchangeMessages::ProblemReportReceived(_problem_report())).unwrap();

                assert_match!(ActorDidExchangeState::Inviter(DidExchangeState::Null(_)), did_exchange_sm.state);
            }

            #[test]
            #[cfg(feature = "general_test")]
            fn test_did_exchange_handle_other_messages_from_responded_state() {
                let _setup = AgencyModeSetup::init();

                let mut did_exchange_sm = inviter_sm().to_inviter_responded_state();

                did_exchange_sm = did_exchange_sm.step(DidExchangeMessages::Connect()).unwrap();

                assert_match!(ActorDidExchangeState::Inviter(DidExchangeState::Responded(_)), did_exchange_sm.state);
            }

            #[test]
            #[cfg(feature = "general_test")]
            fn test_did_exchange_handle_messages_from_completed_state() {
                let _setup = AgencyModeSetup::init();

                let mut did_exchange_sm = inviter_sm().to_inviter_completed_state();

                // Send Ping
                did_exchange_sm = did_exchange_sm.step(DidExchangeMessages::SendPing(None)).unwrap();
                assert_match!(ActorDidExchangeState::Inviter(DidExchangeState::Completed(_)), did_exchange_sm.state);

                // Ping
                did_exchange_sm = did_exchange_sm.step(DidExchangeMessages::PingReceived(_ping())).unwrap();
                assert_match!(ActorDidExchangeState::Inviter(DidExchangeState::Completed(_)), did_exchange_sm.state);

                // Ping Response
                did_exchange_sm = did_exchange_sm.step(DidExchangeMessages::PingResponseReceived(_ping_response())).unwrap();
                assert_match!(ActorDidExchangeState::Inviter(DidExchangeState::Completed(_)), did_exchange_sm.state);

                // Discovery Features
                did_exchange_sm = did_exchange_sm.step(DidExchangeMessages::DiscoverFeatures((None, None))).unwrap();
                assert_match!(ActorDidExchangeState::Inviter(DidExchangeState::Completed(_)), did_exchange_sm.state);

                // Query
                did_exchange_sm = did_exchange_sm.step(DidExchangeMessages::QueryReceived(_query())).unwrap();
                assert_match!(ActorDidExchangeState::Inviter(DidExchangeState::Completed(_)), did_exchange_sm.state);

                // Disclose
                assert!(did_exchange_sm.get_remote_protocols().is_none());

                did_exchange_sm = did_exchange_sm.step(DidExchangeMessages::DiscloseReceived(_disclose())).unwrap();
                assert_match!(ActorDidExchangeState::Inviter(DidExchangeState::Completed(_)), did_exchange_sm.state);

                assert!(did_exchange_sm.get_remote_protocols().is_some());

                // ignore
                // Ack
                did_exchange_sm = did_exchange_sm.step(DidExchangeMessages::AckReceived(_ack())).unwrap();
                assert_match!(ActorDidExchangeState::Inviter(DidExchangeState::Completed(_)), did_exchange_sm.state);

                // Problem Report
                did_exchange_sm = did_exchange_sm.step(DidExchangeMessages::ProblemReportReceived(_problem_report())).unwrap();
                assert_match!(ActorDidExchangeState::Inviter(DidExchangeState::Completed(_)), did_exchange_sm.state);
            }
        }

        mod find_message_to_handle {
            use super::*;

            #[test]
            #[cfg(feature = "general_test")]
            fn test_find_message_to_handle_from_null_state() {
                let _setup = AgencyModeSetup::init();

                let connection = inviter_sm();

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

            #[test]
            #[cfg(feature = "general_test")]
            fn test_find_message_to_handle_from_invited_state() {
                let _setup = AgencyModeSetup::init();

                let connection = inviter_sm().to_inviter_invited_state();

                // Connection Request
                {
                    let messages = map!(
                        "key_1".to_string() => A2AMessage::Ping(_ping()),
                        "key_2".to_string() => A2AMessage::ConnectionRequest(_request()),
                        "key_3".to_string() => A2AMessage::ConnectionResponse(_signed_response())
                    );

                    let (uid, message) = connection.find_message_to_handle(messages).unwrap();
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

            #[test]
            #[cfg(feature = "general_test")]
            fn test_find_message_to_handle_from_responded_state() {
                let _setup = AgencyModeSetup::init();

                let connection = inviter_sm().to_inviter_responded_state();

                // Ping
                {
                    let messages = map!(
                        "key_1".to_string() => A2AMessage::Ping(_ping()),
                        "key_2".to_string() => A2AMessage::ConnectionRequest(_request()),
                        "key_3".to_string() => A2AMessage::ConnectionResponse(_signed_response())
                    );

                    let (uid, message) = connection.find_message_to_handle(messages).unwrap();
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

                    let (uid, message) = connection.find_message_to_handle(messages).unwrap();
                    assert_eq!("key_2", uid);
                    assert_match!(A2AMessage::Ack(_), message);
                }

                // Connection Problem Report
                {
                    let messages = map!(
                        "key_1".to_string() => A2AMessage::ConnectionRequest(_request()),
                        "key_2".to_string() => A2AMessage::ConnectionProblemReport(_problem_report())
                    );

                    let (uid, message) = connection.find_message_to_handle(messages).unwrap();
                    assert_eq!("key_2", uid);
                    assert_match!(A2AMessage::ConnectionProblemReport(_), message);
                }

                // No messages
                {
                    let messages = map!(
                        "key_1".to_string() => A2AMessage::ConnectionRequest(_request()),
                        "key_2".to_string() => A2AMessage::ConnectionResponse(_signed_response())
                    );

                    assert!(connection.find_message_to_handle(messages).is_none());
                }
            }

            #[test]
            #[cfg(feature = "general_test")]
            fn test_find_message_to_handle_from_completed_state() {
                let _setup = AgencyModeSetup::init();

                let connection = inviter_sm().to_inviter_completed_state();

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

            #[test]
            #[cfg(feature = "general_test")]
            fn test_get_state() {
                let _setup = SetupAriesMocks::init();

                assert_eq!(VcxStateType::VcxStateInitialized as u32, inviter_sm().state());
                assert_eq!(VcxStateType::VcxStateOfferSent as u32, inviter_sm().to_inviter_invited_state().state());
                assert_eq!(VcxStateType::VcxStateRequestReceived as u32, inviter_sm().to_inviter_responded_state().state());
                assert_eq!(VcxStateType::VcxStateAccepted as u32, inviter_sm().to_inviter_completed_state().state());
            }
        }
    }
}