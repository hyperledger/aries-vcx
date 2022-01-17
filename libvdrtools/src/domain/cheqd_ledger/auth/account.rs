//! Helper class to handle accounts generic proto conversion

use indy_api_types::errors::{IndyErrorKind, IndyResult};
use indy_api_types::IndyError;

use super::super::CheqdProtoBase;

use super::*;
use super::super::vesting::*;
use super::super::CheqdProto;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "type_url", content = "value")]
pub enum Account {
    BaseAccount(BaseAccount),
    ModuleAccount(ModuleAccount),
    BaseVestingAccount(BaseVestingAccount),
    ContinuousVestingAccount(ContinuousVestingAccount),
    DelayedVestingAccount(DelayedVestingAccount),
    PeriodicVestingAccount(PeriodicVestingAccount)
}

impl Account {
    pub fn account_number(&self) -> u64 {
        match self {
            Account::BaseAccount(account) => account.account_number,
            Account::ModuleAccount(account) => account.base_account.account_number,
            Account::BaseVestingAccount(account) => account.base_account.account_number,
            Account::ContinuousVestingAccount(account) => account.base_vesting_account.base_account.account_number,
            Account::DelayedVestingAccount(account) => account.base_vesting_account.base_account.account_number,
            Account::PeriodicVestingAccount(account) => account.base_vesting_account.base_account.account_number,
        }
    }
    pub fn account_sequence(&self) -> u64 {
        match self {
            Account::BaseAccount(account) => account.sequence,
            Account::ModuleAccount(account) => account.base_account.sequence,
            Account::BaseVestingAccount(account) => account.base_account.sequence,
            Account::ContinuousVestingAccount(account) => account.base_vesting_account.base_account.sequence,
            Account::DelayedVestingAccount(account) => account.base_vesting_account.base_account.sequence,
            Account::PeriodicVestingAccount(account) => account.base_vesting_account.base_account.sequence,
        }
    }
}

impl CheqdProtoBase for Account {
    type Proto = prost_types::Any;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        unimplemented!()
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        match &proto.type_url[..] {
            "/cosmos.auth.v1beta1.BaseAccount" => {
                let val = BaseAccount::from_proto_bytes(&proto.value)?;
                Ok(Account::BaseAccount(val))
            }
            "/cosmos.auth.v1beta1.ModuleAccount" => {
               let val = ModuleAccount::from_proto_bytes(&proto.value)?;
               Ok(Account::ModuleAccount(val))
            }
            "/cosmos.vesting.v1beta1.BaseVestingAccount" => {
               let val = BaseVestingAccount::from_proto_bytes(&proto.value)?;
               Ok(Account::BaseVestingAccount(val))
            }
            "/cosmos.vesting.v1beta1.ContinuousVestingAccount" => {
               let val = ContinuousVestingAccount::from_proto_bytes(&proto.value)?;
               Ok(Account::ContinuousVestingAccount(val))
            }
            "/cosmos.vesting.v1beta1.DelayedVestingAccount" => {
               let val = DelayedVestingAccount::from_proto_bytes(&proto.value)?;
               Ok(Account::DelayedVestingAccount(val))
            }
            "/cosmos.vesting.v1beta1.PeriodicVestingAccount" => {
                let val = PeriodicVestingAccount::from_proto_bytes(&proto.value)?;
                Ok(Account::PeriodicVestingAccount(val))
            }
            unknown_type => Err(IndyError::from_msg(
                IndyErrorKind::InvalidStructure,
                format!("Unknown account type: {}", unknown_type),
            )),
        }
    }
}
