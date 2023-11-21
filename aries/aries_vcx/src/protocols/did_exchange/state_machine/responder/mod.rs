mod response_sent;

use super::DidExchange;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Responder;

pub type DidExchangeResponder<S> = DidExchange<Responder, S>;
