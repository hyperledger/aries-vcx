use indy_api_types::errors::IndyResult;

use cosmrs::proto::cosmos::tx::v1beta1::ModeInfo as ProtoModeInfo;

use super::super::super::cheqd_ledger::CheqdProtoBase;
use super::Sum;

/// ModeInfo describes the signing mode of a single or nested multisig signer.
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct ModeInfo {
    /// sum is the oneof that specifies whether this represents a single or nested
    /// multisig signer
    pub sum: Option<Sum>,
}

impl ModeInfo {
    pub fn new(
        sum: Option<Sum>,
    ) -> Self {
        ModeInfo {
            sum,
        }
    }
}

impl CheqdProtoBase for ModeInfo {
    type Proto = ProtoModeInfo;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            sum: self.sum.to_proto()?,
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(
            Option::<Sum>::from_proto(&proto.sum)?
        ))
    }
}
