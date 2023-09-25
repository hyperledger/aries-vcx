use derive_more::From;

use self::{v1::CredentialIssuanceV1, v2::CredentialIssuanceV2};
use crate::AriesMessage;

pub mod common;
pub mod v1;
pub mod v2;

#[derive(Clone, Debug, From, PartialEq)]
pub enum CredentialIssuance {
    V1(CredentialIssuanceV1),
    V2(CredentialIssuanceV2),
}

impl From<CredentialIssuanceV1> for AriesMessage {
    fn from(value: CredentialIssuanceV1) -> Self {
        AriesMessage::CredentialIssuance(CredentialIssuance::V1(value))
    }
}

impl From<CredentialIssuanceV2> for AriesMessage {
    fn from(value: CredentialIssuanceV2) -> Self {
        AriesMessage::CredentialIssuance(CredentialIssuance::V2(value))
    }
}
