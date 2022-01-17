use cosmrs::proto::cosmos::auth::v1beta1::ModuleAccount as ProtoModuleAccount;
use super::base_account::BaseAccount;
use indy_api_types::errors::{IndyResult, IndyError, IndyErrorKind};

use super::super::CheqdProtoBase;


#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct ModuleAccount {
    pub base_account: BaseAccount,
    pub name: String,
    pub permissions: Vec<String>,
}

impl ModuleAccount {
    pub fn new(
        base_account: BaseAccount,
        name: String,
        permissions: Vec<String>,
    ) -> Self {
        ModuleAccount {
            base_account,
            name,
            permissions,
        }
    }
}

impl CheqdProtoBase for ModuleAccount {
    type Proto = ProtoModuleAccount;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            base_account: Some(self.base_account.to_proto()?),
            name: self.name.clone(),
            permissions: self.permissions.clone(),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        let base_account = proto.base_account.as_ref().ok_or(
            IndyError::from_msg(
                IndyErrorKind::InvalidStructure,"Failed to get BaseAccount from ModuleAccount object"))?;

        Ok(Self::new(
            BaseAccount::from_proto(base_account)?,
            proto.name.clone(),
            proto.permissions.clone(),
        ))
    }
}

