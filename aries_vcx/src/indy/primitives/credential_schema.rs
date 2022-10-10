use vdrtools_sys::{PoolHandle, WalletHandle};
use crate::error::{VcxError, VcxResult};
use crate::global::settings;
use crate::indy::ledger::transactions::{
    _check_schema_response, build_schema_request, get_schema_json,
    sign_and_submit_to_ledger,
};
use crate::indy::primitives::credential_definition::PublicEntityStateType;
use crate::utils::constants::{DEFAULT_SERIALIZE_VERSION, SCHEMA_ID, SCHEMA_JSON};
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
        if get_schema_json(wallet_handle, pool_handle, &self.schema_id).await.is_ok() {
            self.state = PublicEntityStateType::Published
        }
        Ok(self.state as u32)
    }

    pub fn get_state(&self) -> u32 {
        self.state as u32
    }
}

pub async fn libindy_issuer_create_schema(
    issuer_did: &str,
    name: &str,
    version: &str,
    attrs: &str,
) -> VcxResult<(String, String)> {
    trace!(
        "libindy_issuer_create_schema >>> issuer_did: {}, name: {}, version: {}, attrs: {}",
        issuer_did,
        name,
        version,
        attrs
    );

    vdrtools::anoncreds::issuer_create_schema(issuer_did, name, version, attrs)
        .await
        .map_err(VcxError::from)
}

pub async fn create_schema(submitter_did: &str, name: &str, version: &str, data: &str) -> VcxResult<(String, String)> {
    trace!("create_schema >>> submitter_did: {}, name: {}, version: {}, data: {}", submitter_did, name, version, data);

    if settings::indy_mocks_enabled() {
        return Ok((SCHEMA_ID.to_string(), SCHEMA_JSON.to_string()));
    }

    let (id, create_schema) = libindy_issuer_create_schema(&submitter_did, name, version, data).await?;

    Ok((id, create_schema))
}

pub async fn publish_schema(submitter_did: &str, wallet_handle: WalletHandle, pool_handle: PoolHandle, schema: &str) -> VcxResult<()> {
    trace!("publish_schema >>> submitter_did: {}, schema: {}", submitter_did, schema);

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    let request = build_schema_request(submitter_did, schema).await?;

    let response = sign_and_submit_to_ledger(wallet_handle, pool_handle, submitter_did, &request).await?;

    _check_schema_response(&response)?;

    Ok(())
}
