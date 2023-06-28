mod request_sent;

use super::DidExchangeService;

pub use request_sent::config::{ConstructRequestConfig, PairwiseConstructRequestConfig, PublicConstructRequestConfig};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Requester;

pub type DidExchangeServiceRequester<S> = DidExchangeService<Requester, S>;
