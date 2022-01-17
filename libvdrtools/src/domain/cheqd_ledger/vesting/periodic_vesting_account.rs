use cosmrs::proto::cosmos::vesting::v1beta1::PeriodicVestingAccount as ProtoPeriodicVestingAccount;
use super::BaseVestingAccount;
use indy_api_types::errors::{IndyResult, IndyErrorKind};

use super::super::CheqdProtoBase;
use indy_api_types::IndyError;
use super::super::vesting::Period;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct PeriodicVestingAccount {
    pub base_vesting_account: BaseVestingAccount,
    pub start_time: i64,
    pub vesting_periods: Vec<Period>
}

impl PeriodicVestingAccount {
    pub fn new(
        base_vesting_account: BaseVestingAccount,
        start_time: i64,
        vesting_periods: Vec<Period>
    ) -> Self {
        PeriodicVestingAccount {
            base_vesting_account,
            start_time,
            vesting_periods
        }
    }
}

impl CheqdProtoBase for PeriodicVestingAccount {
    type Proto = ProtoPeriodicVestingAccount;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            base_vesting_account: Some(self.base_vesting_account.to_proto()?),
            start_time: self.start_time.clone(),
            vesting_periods: self.vesting_periods.to_proto()?
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {

        let base_vesting_account = proto.base_vesting_account.as_ref().ok_or(
            IndyError::from_msg(
                IndyErrorKind::InvalidStructure, "Failed to get BaseVestingAccount from PeriodicVestingAccount object"))?;

        Ok(Self::new(
            BaseVestingAccount::from_proto(base_vesting_account)?,
            proto.start_time.clone(),
            Vec::<Period>::from_proto(&proto.vesting_periods)?,
        ))
    }
}
