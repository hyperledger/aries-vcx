use super::{GenericDidExchange, RequesterState, ResponderState};
use crate::protocols::did_exchange::{
    state_machine::{requester::DidExchangeRequester, responder::DidExchangeResponder},
    states::{
        completed::Completed, requester::request_sent::RequestSent,
        responder::response_sent::ResponseSent,
    },
};

impl From<DidExchangeRequester<RequestSent>> for GenericDidExchange {
    fn from(state: DidExchangeRequester<RequestSent>) -> Self {
        Self::Requester(RequesterState::RequestSent(state))
    }
}

impl From<DidExchangeRequester<Completed>> for GenericDidExchange {
    fn from(state: DidExchangeRequester<Completed>) -> Self {
        Self::Requester(RequesterState::Completed(state))
    }
}

impl From<DidExchangeResponder<ResponseSent>> for GenericDidExchange {
    fn from(state: DidExchangeResponder<ResponseSent>) -> Self {
        Self::Responder(ResponderState::ResponseSent(state))
    }
}

impl From<DidExchangeResponder<Completed>> for GenericDidExchange {
    fn from(state: DidExchangeResponder<Completed>) -> Self {
        Self::Responder(ResponderState::Completed(state))
    }
}
