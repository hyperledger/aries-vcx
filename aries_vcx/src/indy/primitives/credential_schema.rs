use vdrtools::{PoolHandle, WalletHandle};

use vdrtools::{Locator, DidValue, AttributeNames};
use crate::error::{VcxError, VcxResult, VcxErrorKind};
use crate::global::settings;
use crate::indy::ledger::transactions::{_check_schema_response, build_schema_request, get_schema_json, sign_and_submit_to_ledger, set_endorser};
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
    submitter_did: String,
    #[serde(default)]
    pub state: PublicEntityStateType,
    #[serde(default)]
    schema_json: String // added in 0.45.0, #[serde(default)] use for backwards compatibility
}

impl Schema {
    pub async fn create(source_id: &str, submitter_did: &str, name: &str, version: &str, data: &Vec<String>) -> VcxResult<Self> {
        trace!("Schema::create >>> submitter_did: {}, name: {}, version: {}, data: {:?}", submitter_did, name, version, data);

        if settings::indy_mocks_enabled() {
            return Ok(Self {
                source_id: source_id.to_string(),
                version: version.to_string(),
                submitter_did: submitter_did.to_string(),
                schema_id: SCHEMA_ID.to_string(),
                schema_json: SCHEMA_JSON.to_string(),
                name: name.to_string(),
                state: PublicEntityStateType::Built,
                ..Self::default()
            });
        }

        let data_str = serde_json::to_string(data)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to serialize schema attributes, err: {}", err)))?;

        let (schema_id, schema_json) = libindy_issuer_create_schema(&submitter_did, name, version, &data_str).await?;

        Ok(Self {
            source_id: source_id.to_string(),
            name: name.to_string(),
            data: data.clone(),
            version: version.to_string(),
            schema_id,
            submitter_did: submitter_did.to_string(),
            schema_json,
            state: PublicEntityStateType::Built,
        })
    }

    pub async fn create_from_ledger_json(wallet_handle: WalletHandle, pool_handle: PoolHandle, source_id: &str, schema_id: &str) -> VcxResult<Self> {
        let schema_json = get_schema_json(wallet_handle, pool_handle, schema_id).await?.1;
        let schema_data: SchemaData = serde_json::from_str(&schema_json)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize schema: {}", err)))?;

        Ok(Self {
            source_id: source_id.to_string(),
            schema_id: schema_id.to_string(),
            schema_json: schema_json.to_string(),
            name: schema_data.name,
            version: schema_data.version,
            data: schema_data.attr_names,
            submitter_did: "".to_string(),
            state: PublicEntityStateType::Published,
        })
    }

    pub async fn publish(self, wallet_handle: WalletHandle, pool_handle: PoolHandle, endorser_did: Option<String>) -> VcxResult<Self> {
        trace!("Schema::publish >>>");

        if settings::indy_mocks_enabled() {
            return Ok(Self { state: PublicEntityStateType::Published, ..self });
        }

        let mut request = build_schema_request(&self.submitter_did, &self.schema_json).await?;
        if let Some(endorser_did) = endorser_did {
            request = set_endorser(wallet_handle, &self.submitter_did, &request, &endorser_did).await?;
        }
        let response = sign_and_submit_to_ledger(wallet_handle, pool_handle, &self.submitter_did, &request).await?;
        _check_schema_response(&response)?;

        return Ok(Self {
            state: PublicEntityStateType::Published,
            ..self
        });
    }

    pub fn get_source_id(&self) -> String {
        self.source_id.clone()
    }

    pub fn get_schema_id(&self) -> String {
        self.schema_id.clone()
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

    pub async fn get_schema_json(&self, wallet_handle: WalletHandle, pool_handle: PoolHandle) -> VcxResult<String> {
        if !self.schema_json.is_empty() {
            Ok(self.schema_json.clone())
        } else {
            Ok(get_schema_json(wallet_handle, pool_handle, &self.schema_id).await?.1)
        }
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

    let attrs = serde_json::from_str::<AttributeNames>(attrs)?;

    let res = Locator::instance()
        .issuer_controller
        .create_schema(
            DidValue(issuer_did.into()),
            name.into(),
            version.into(),
            attrs,
        )?;

    Ok(res)
}
