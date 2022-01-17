use cosmrs::proto::cosmos::base::v1beta1::Coin as ProtoCoin;
use indy_api_types::errors::IndyResult;

use super::super::CheqdProtoBase;

#[derive(Eq, Clone, PartialEq, Debug, Serialize, Deserialize )]
pub struct Coin {
    pub denom: String,
    pub amount: String,
}

impl Coin {
    pub fn new(
        denom: String,
        amount: String,
    ) -> Self {
        Coin {
            denom,
            amount,
        }
    }
}

impl CheqdProtoBase for Coin {
    type Proto = ProtoCoin;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            denom: self.denom.clone(),
            amount: self.amount.clone()
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(
            proto.denom.clone(),
            proto.amount.clone()
        ))
    }
}
