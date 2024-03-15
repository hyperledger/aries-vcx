pub mod helpers;
mod request_sent;

use super::DidExchange;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Requester;

pub type DidExchangeRequester<S> = DidExchange<Requester, S>;
