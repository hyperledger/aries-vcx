mod request_sent;

use super::DidExchange;

pub use request_sent::config::{ConstructRequestConfig, PairwiseConstructRequestConfig, PublicConstructRequestConfig};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Requester;

pub type DidExchangeRequester<S> = DidExchange<Requester, S>;
