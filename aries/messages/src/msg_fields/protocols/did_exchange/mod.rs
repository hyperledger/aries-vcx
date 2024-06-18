use derive_more::From;
use v1_0::DidExchangeV1_0;
use v1_1::DidExchangeV1_1;

pub mod v1_0;
pub mod v1_1;
pub mod v1_x;

#[derive(Clone, Debug, From, PartialEq)]
pub enum DidExchange {
    V1_0(DidExchangeV1_0),
    V1_1(DidExchangeV1_1),
}
