use cosmrs::proto::cosmos::vesting::v1beta1::DelayedVestingAccount as ProtoDelayedVestingAccount;
use super::BaseVestingAccount;
use indy_api_types::errors::{IndyResult, IndyErrorKind};

use super::super::CheqdProtoBase;
use indy_api_types::IndyError;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct DelayedVestingAccount {
    pub base_vesting_account: BaseVestingAccount,
}

impl DelayedVestingAccount {
    pub fn new(
        base_vesting_account: BaseVestingAccount,
    ) -> Self {
        DelayedVestingAccount {
            base_vesting_account,
        }
    }
}

impl CheqdProtoBase for DelayedVestingAccount {
    type Proto = ProtoDelayedVestingAccount;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            base_vesting_account: Some(self.base_vesting_account.to_proto()?),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        let base_vesting_account = proto.base_vesting_account.as_ref().ok_or(
            IndyError::from_msg(
                IndyErrorKind::InvalidStructure, "Failed to get BaseVestingAccount from DelayedVestingAccount object"))?;

        Ok(Self::new(
            BaseVestingAccount::from_proto(base_vesting_account)?
        ))
    }
}
