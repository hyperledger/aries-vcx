use derive_more::From;
use v1_0::DidExchangeV1_0;

pub mod v1_0;
pub mod v1_1;

#[derive(Clone, Debug, From, PartialEq)]
pub enum DidExchange {
    V1_0(DidExchangeV1_0),
}
