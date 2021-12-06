use indy_api_types::errors::IndyResult;

use cosmrs::proto::cosmos::tx::v1beta1::mode_info::Single as ProtoSingle;

use super::super::super::cheqd_ledger::CheqdProtoBase;

/// Nested message and enum types in `ModeInfo`.
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Single {
    /// mode is the signing mode of the single signer
    pub mode: i32,
}

impl Single {
    pub fn new(
        mode: i32,
    ) -> Self {
        Single {
            mode,
        }
    }
}

impl CheqdProtoBase for Single {
    type Proto = ProtoSingle;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            mode: self.mode.clone(),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(
            proto.mode.clone(),
        ))
    }
}
