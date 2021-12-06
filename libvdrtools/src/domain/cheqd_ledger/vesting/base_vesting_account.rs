use cosmrs::proto::cosmos::vesting::v1beta1::BaseVestingAccount as ProtoBaseVestingAccount;
use indy_api_types::errors::{IndyResult, IndyErrorKind};

use super::super::CheqdProtoBase;
use indy_api_types::IndyError;
use super::super::bank::Coin;
use super::super::auth::BaseAccount;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct BaseVestingAccount {
    pub base_account: BaseAccount,
    pub original_vesting: Vec<Coin>,
    pub delegated_free: Vec<Coin>,
    pub delegated_vesting: Vec<Coin>,
    pub end_time: i64,
}

impl BaseVestingAccount {
    pub fn new(
        base_account: BaseAccount,
        original_vesting: Vec<Coin>,
        delegated_free: Vec<Coin>,
        delegated_vesting: Vec<Coin>,
        end_time: i64,
    ) -> Self {
        BaseVestingAccount {
            base_account,
            original_vesting,
            delegated_free,
            delegated_vesting,
            end_time,
        }
    }
}

impl CheqdProtoBase for BaseVestingAccount {
    type Proto = ProtoBaseVestingAccount;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            base_account: Some(self.base_account.to_proto()?),
            original_vesting: self.original_vesting.to_proto()?,
            delegated_free: self.delegated_free.to_proto()?,
            delegated_vesting: self.delegated_vesting.to_proto()?,
            end_time: self.end_time,
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        let base_account = proto.base_account.as_ref().ok_or(
            IndyError::from_msg(
                IndyErrorKind::InvalidStructure, "Failed to get BaseAccount from BaseVestingAccount object"))?;

        Ok(Self::new(
            BaseAccount::from_proto(base_account)?,
            Vec::<Coin>::from_proto(&proto.original_vesting)?,
            Vec::<Coin>::from_proto(&proto.delegated_free)?,
            Vec::<Coin>::from_proto(&proto.delegated_vesting)?,
            proto.end_time.clone(),
        ))
    }
}
