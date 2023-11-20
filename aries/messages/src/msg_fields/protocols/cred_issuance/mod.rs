use derive_more::From;

use self::{v1::CredentialIssuanceV1, v2::CredentialIssuanceV2};

pub mod common;
pub mod v1;
pub mod v2;

#[derive(Clone, Debug, From, PartialEq)]
pub enum CredentialIssuance {
    V1(CredentialIssuanceV1),
    V2(CredentialIssuanceV2),
}
