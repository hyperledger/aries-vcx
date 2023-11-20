use derive_more::From;

use self::{v1::PresentProofV1, v2::PresentProofV2};

pub mod v1;
pub mod v2;

#[derive(Clone, Debug, From, PartialEq)]
pub enum PresentProof {
    V1(PresentProofV1),
    V2(PresentProofV2),
}
