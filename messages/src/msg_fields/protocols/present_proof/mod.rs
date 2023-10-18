use derive_more::From;

use self::v1::PresentProofV1;

pub mod v1;

#[derive(Clone, Debug, From, PartialEq)]
pub enum PresentProof {
    V1(PresentProofV1),
}
