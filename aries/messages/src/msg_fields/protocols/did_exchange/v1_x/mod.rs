//! Common components for V1.X DIDExchange messages (v1.0 & v1.1).
//! Necessary to prevent duplicated code, since most types between v1.0 & v1.1 are identical

use serde::Serialize;

use super::{v1_0::DidExchangeV1_0, v1_1::DidExchangeV1_1, DidExchange};
use crate::{msg_types::protocols::did_exchange::DidExchangeTypeV1, AriesMessage};

pub mod complete;
pub mod problem_report;
pub mod request;
pub mod response;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum DidExchangeV1MessageVariant<V1_0, V1_1> {
    V1_0(V1_0),
    V1_1(V1_1),
}

impl<A, B> DidExchangeV1MessageVariant<A, B> {
    pub fn get_version(&self) -> DidExchangeTypeV1 {
        match self {
            DidExchangeV1MessageVariant::V1_0(_) => DidExchangeTypeV1::new_v1_0(),
            DidExchangeV1MessageVariant::V1_1(_) => DidExchangeTypeV1::new_v1_1(),
        }
    }
}

impl<T> DidExchangeV1MessageVariant<T, T> {
    pub fn into_inner(self) -> T {
        match self {
            DidExchangeV1MessageVariant::V1_0(r) | DidExchangeV1MessageVariant::V1_1(r) => r,
        }
    }

    pub fn inner(&self) -> &T {
        match self {
            DidExchangeV1MessageVariant::V1_0(r) | DidExchangeV1MessageVariant::V1_1(r) => r,
        }
    }
}

impl<A, B> From<DidExchangeV1MessageVariant<A, B>> for AriesMessage
where
    A: Into<DidExchangeV1_0>,
    B: Into<DidExchangeV1_1>,
{
    fn from(value: DidExchangeV1MessageVariant<A, B>) -> Self {
        match value {
            DidExchangeV1MessageVariant::V1_0(a) => {
                AriesMessage::DidExchange(DidExchange::V1_0(a.into()))
            }
            DidExchangeV1MessageVariant::V1_1(b) => {
                AriesMessage::DidExchange(DidExchange::V1_1(b.into()))
            }
        }
    }
}
