use indy_sys::{WalletHandle, PoolHandle};

use crate::error::prelude::*;
use crate::libindy::credential_def::PublicEntityStateType;
use crate::libindy::utils::anoncreds;
use crate::utils::constants::DEFAULT_SERIALIZE_VERSION;
use crate::utils::serialization::ObjectWithVersion;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SchemaData {
    pub name: String,
    pub version: String,
    #[serde(rename = "attrNames")]
    pub attr_names: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Schema {
    pub data: Vec<String>,
    pub version: String,
    pub schema_id: String,
    pub name: String,
    pub source_id: String,
    #[serde(default)]
    pub state: PublicEntityStateType,
}

impl Schema {
    pub fn get_source_id(&self) -> &String {
        &self.source_id
    }

    pub fn get_schema_id(&self) -> &String {
        &self.schema_id
    }

    pub fn to_string(&self) -> VcxResult<String> {
        ObjectWithVersion::new(DEFAULT_SERIALIZE_VERSION, self.to_owned())
            .serialize()
            .map_err(|err| err)
            .map_err(|err: VcxError| err.extend("Cannot serialize Schema"))
    }

    pub fn from_str(data: &str) -> VcxResult<Schema> {
        ObjectWithVersion::deserialize(data)
            .map(|obj: ObjectWithVersion<Schema>| obj.data)
            .map_err(|err| err)
            .map_err(|err: VcxError| err.extend("Cannot deserialize Schema"))
    }

    pub async fn update_state(&mut self, wallet_handle: WalletHandle, pool_handle: PoolHandle) -> VcxResult<u32> {
        if anoncreds::get_schema_json(wallet_handle, pool_handle, &self.schema_id).await.is_ok() {
            self.state = PublicEntityStateType::Published
        }
        Ok(self.state as u32)
    }

    pub fn get_state(&self) -> u32 {
        self.state as u32
    }
}
