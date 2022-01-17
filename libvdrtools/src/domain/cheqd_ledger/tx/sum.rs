use indy_api_types::errors::{IndyResult, IndyErrorKind};

use cosmrs::proto::cosmos::tx::v1beta1::mode_info::Sum as ProtoSum;

use super::Single;
use super::super::super::cheqd_ledger::CheqdProtoBase;
use indy_api_types::IndyError;

/// sum is the oneof that specifies whether this represents a single or nested
/// multisig signer
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub enum Sum {
    /// single represents a single signer
    Single(Single),
}

impl CheqdProtoBase for Sum {
    type Proto = ProtoSum;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        match self {
            Sum::Single(single) => {
                Ok(ProtoSum::Single(single.to_proto()?))
            }
        }
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        match proto {
            ProtoSum::Single(proto_single) => {
                let single = Single::from_proto(proto_single)?;
                Ok(Sum::Single(single))
            }
            ProtoSum::Multi(_) => {
                Err(
                    IndyError::from_msg(IndyErrorKind::InvalidStructure,
                                        "Only SINGLE type of Sum is supported")) }
        }
    }
}
