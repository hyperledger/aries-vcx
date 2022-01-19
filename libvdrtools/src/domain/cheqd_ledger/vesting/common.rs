use cosmrs::proto::cosmos::vesting::v1beta1::Period as ProtoPeriod;
use super::super::CheqdProtoBase;
use indy_api_types::errors::IndyResult;
use super::super::bank::Coin;

#[derive(Eq, Clone, PartialEq, Debug, Serialize, Deserialize )]
pub struct Period {
    pub length: i64,
    pub amount: Vec<Coin>,
}

impl Period {
    pub fn new(
        length: i64,
        amount: Vec<Coin>,
    ) -> Self {
        Period {
            length,
            amount,
        }
    }
}

impl CheqdProtoBase for Period {
    type Proto = ProtoPeriod;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            length: self.length.clone(),
            amount: self.amount.to_proto()?
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(
            proto.length.clone(),
            Vec::<Coin>::from_proto(&proto.amount)?,
        ))
    }
}
