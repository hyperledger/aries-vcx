mod conversions;
mod thin_state;

use did_doc_sov::DidDocumentSov;
use messages::msg_fields::protocols::did_exchange::{
    complete::Complete, problem_report::ProblemReport, request::Request, response::Response,
};

use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    protocols::did_exchange::{
        states::{
            abandoned::Abandoned, completed::Completed, requester::request_sent::RequestSent,
            responder::response_sent::ResponseSent,
        },
        transition::{transition_error::TransitionError, transition_result::TransitionResult},
    },
};

use super::{
    requester::{ConstructRequestConfig, DidExchangeRequester},
    responder::{DidExchangeResponder, ReceiveRequestConfig},
};

pub use thin_state::ThinState;

#[derive(Debug, Clone)]
pub enum GenericDidExchange {
    Requester(RequesterState),
    Responder(ResponderState),
}

#[derive(Debug, Clone)]
pub enum RequesterState {
    RequestSent(DidExchangeRequester<RequestSent>),
    Completed(DidExchangeRequester<Completed>),
    Abandoned(DidExchangeRequester<Abandoned>),
}

#[derive(Debug, Clone)]
pub enum ResponderState {
    ResponseSent(DidExchangeResponder<ResponseSent>),
    Completed(DidExchangeResponder<Completed>),
    Abandoned(DidExchangeResponder<Abandoned>),
}

impl GenericDidExchange {
    pub fn our_did_document(&self) -> &DidDocumentSov {
        match self {
            GenericDidExchange::Requester(requester_state) => match requester_state {
                RequesterState::RequestSent(request_sent_state) => request_sent_state.our_did_doc(),
                RequesterState::Completed(completed_state) => completed_state.our_did_doc(),
                RequesterState::Abandoned(abandoned_state) => todo!(),
            },
            GenericDidExchange::Responder(responder_state) => match responder_state {
                ResponderState::ResponseSent(response_sent_state) => response_sent_state.our_did_doc(),
                ResponderState::Completed(completed_state) => completed_state.our_did_doc(),
                ResponderState::Abandoned(abandoned_state) => todo!(),
            },
        }
    }

    pub fn their_did_doc(&self) -> &DidDocumentSov {
        match self {
            GenericDidExchange::Requester(requester_state) => match requester_state {
                RequesterState::RequestSent(request_sent_state) => request_sent_state.their_did_doc(),
                RequesterState::Completed(completed_state) => completed_state.their_did_doc(),
                RequesterState::Abandoned(abandoned_state) => todo!(),
            },
            GenericDidExchange::Responder(responder_state) => match responder_state {
                ResponderState::ResponseSent(response_sent_state) => response_sent_state.their_did_doc(),
                ResponderState::Completed(completed_state) => completed_state.their_did_doc(),
                ResponderState::Abandoned(abandoned_state) => todo!(),
            },
        }
    }

    pub fn invitation_id(&self) -> &str {
        match self {
            GenericDidExchange::Requester(requester_state) => match requester_state {
                RequesterState::RequestSent(request_sent_state) => request_sent_state.get_invitation_id(),
                RequesterState::Completed(completed_state) => completed_state.get_invitation_id(),
                RequesterState::Abandoned(_) => todo!(),
            },
            GenericDidExchange::Responder(responder_state) => match responder_state {
                ResponderState::ResponseSent(response_sent_state) => response_sent_state.get_invitation_id(),
                ResponderState::Completed(completed_state) => completed_state.get_invitation_id(),
                ResponderState::Abandoned(abandoned_state) => todo!(),
            },
        }
    }

    pub async fn construct_request(config: ConstructRequestConfig) -> Result<(Self, Request), AriesVcxError> {
        let TransitionResult { state, output } = DidExchangeRequester::<RequestSent>::construct_request(config).await?;
        Ok((
            GenericDidExchange::Requester(RequesterState::RequestSent(state)),
            output,
        ))
    }

    pub async fn handle_request(config: ReceiveRequestConfig) -> Result<(Self, Response), AriesVcxError> {
        let TransitionResult { state, output } = DidExchangeResponder::<ResponseSent>::receive_request(config).await?;
        Ok((
            GenericDidExchange::Responder(ResponderState::ResponseSent(state)),
            output,
        ))
    }

    pub async fn handle_response(self, response: Response) -> Result<(Self, Complete), (Self, AriesVcxError)> {
        match self {
            GenericDidExchange::Requester(requester_state) => match requester_state {
                RequesterState::RequestSent(request_sent_state) => {
                    match request_sent_state.receive_response(response).await {
                        Ok(TransitionResult { state, output }) => {
                            Ok((GenericDidExchange::Requester(RequesterState::Completed(state)), output))
                        }
                        Err(TransitionError { state, error }) => {
                            Err((GenericDidExchange::Requester(RequesterState::RequestSent(state)), error))
                        }
                    }
                }
                RequesterState::Completed(completed_state) => Err((
                    GenericDidExchange::Requester(RequesterState::Completed(completed_state)),
                    AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidState,
                        "Attempted to handle response in completed state",
                    ),
                )),
                RequesterState::Abandoned(abandoned_state) => Err((
                    GenericDidExchange::Requester(RequesterState::Abandoned(abandoned_state)),
                    AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidState,
                        "Attempted to handle response in abandoned state",
                    ),
                )),
            },
            GenericDidExchange::Responder(responder) => Err((
                GenericDidExchange::Responder(responder),
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Attempted to handle response as a responder",
                ),
            )),
        }
    }

    pub fn handle_complete(self, complete: Complete) -> Result<Self, (Self, AriesVcxError)> {
        match self {
            GenericDidExchange::Responder(responder_state) => match responder_state {
                ResponderState::ResponseSent(response_sent_state) => {
                    match response_sent_state.receive_complete(complete) {
                        Ok(state) => Ok(GenericDidExchange::Responder(ResponderState::Completed(state))),
                        Err(TransitionError { state, error }) => Err((
                            GenericDidExchange::Responder(ResponderState::ResponseSent(state)),
                            error,
                        )),
                    }
                }
                ResponderState::Completed(completed_state) => Err((
                    GenericDidExchange::Responder(ResponderState::Completed(completed_state)),
                    AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidState,
                        "Attempted to handle complete in completed state",
                    ),
                )),
                ResponderState::Abandoned(_) => Err((
                    GenericDidExchange::Responder(responder_state),
                    AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidState,
                        "Attempted to handle complete in abandoned state",
                    ),
                )),
            },
            GenericDidExchange::Requester(requester_state) => Err((
                GenericDidExchange::Requester(requester_state),
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Attempted to handle complete as a requester",
                ),
            )),
        }
    }

    pub fn handle_problem_report(self, problem_report: ProblemReport) -> Result<Self, (Self, AriesVcxError)> {
        match self {
            GenericDidExchange::Requester(requester_state) => match requester_state {
                RequesterState::RequestSent(request_sent_state) => Ok(GenericDidExchange::Requester(
                    RequesterState::Abandoned(request_sent_state.receive_problem_report(problem_report)),
                )),
                RequesterState::Completed(completed_state) => Err((
                    GenericDidExchange::Requester(RequesterState::Completed(completed_state)),
                    AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidState,
                        "Attempted to handle problem report in completed state",
                    ),
                )),
                RequesterState::Abandoned(abandoned_state) => Ok(GenericDidExchange::Requester(
                    RequesterState::Abandoned(abandoned_state),
                )),
            },
            GenericDidExchange::Responder(responder_state) => match responder_state {
                ResponderState::ResponseSent(response_sent_state) => Ok(GenericDidExchange::Responder(
                    ResponderState::Abandoned(response_sent_state.receive_problem_report(problem_report)),
                )),
                ResponderState::Completed(completed_state) => Err((
                    GenericDidExchange::Responder(ResponderState::Completed(completed_state)),
                    AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidState,
                        "Attempted to handle problem report in completed state",
                    ),
                )),
                ResponderState::Abandoned(abandoned_state) => Ok(GenericDidExchange::Responder(
                    ResponderState::Abandoned(abandoned_state),
                )),
            },
        }
    }

    pub fn get_state(&self) -> ThinState {
        match self {
            GenericDidExchange::Requester(requester_state) => match requester_state {
                RequesterState::RequestSent(_) => ThinState::RequestSent,
                RequesterState::Completed(_) => ThinState::Completed,
                RequesterState::Abandoned(_) => ThinState::Abandoned,
            },
            GenericDidExchange::Responder(responder_state) => match responder_state {
                ResponderState::ResponseSent(_) => ThinState::RequestSent,
                ResponderState::Completed(_) => ThinState::Completed,
                ResponderState::Abandoned(_) => ThinState::Abandoned,
            },
        }
    }
}
