use cosmrs::proto::cosmos::vesting::v1beta1::ContinuousVestingAccount as ProtoContinuousVestingAccount;
use super::BaseVestingAccount;
use indy_api_types::errors::{IndyResult, IndyErrorKind};

use super::super::CheqdProtoBase;
use indy_api_types::IndyError;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct ContinuousVestingAccount {
    pub base_vesting_account: BaseVestingAccount,
    pub start_time: i64,
}

impl ContinuousVestingAccount {
    pub fn new(
        base_vesting_account: BaseVestingAccount,
        start_time: i64,
    ) -> Self {
        ContinuousVestingAccount {
            base_vesting_account,
            start_time,
        }
    }
}

impl CheqdProtoBase for ContinuousVestingAccount {
    type Proto = ProtoContinuousVestingAccount;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            base_vesting_account: Some(self.base_vesting_account.to_proto()?),
            start_time: self.start_time.clone(),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        let base_vesting_account = proto.base_vesting_account.as_ref().ok_or(
            IndyError::from_msg(
                IndyErrorKind::InvalidStructure, "Failed to get BaseVestingAccount from ContinuousVestingAccount object"))?;

        Ok(Self::new(
                    BaseVestingAccount::from_proto(base_vesting_account)?,
                    proto.start_time.clone()
        ))
    }
}

