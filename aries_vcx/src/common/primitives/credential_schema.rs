use std::sync::Arc;

use super::credential_definition::PublicEntityStateType;
use crate::{
    core::profile::profile::Profile,
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    global::settings,
    utils::{
        constants::{DEFAULT_SERIALIZE_VERSION, SCHEMA_ID, SCHEMA_JSON},
        serialization::ObjectWithVersion,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SchemaData {
    pub name: String,
    pub version: String,
    #[serde(rename = "attrNames")]
    pub attr_names: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
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
    schema_json: String, // added in 0.45.0, #[serde(default)] use for backwards compatibility
}

impl Schema {
    pub async fn create(
        profile: &Arc<dyn Profile>,
        source_id: &str,
        submitter_did: &str,
        name: &str,
        version: &str,
        data: &Vec<String>,
    ) -> VcxResult<Self> {
        trace!(
            "Schema::create >>> submitter_did: {}, name: {}, version: {}, data: {:?}",
            submitter_did,
            name,
            version,
            data
        );

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

        let data_str = serde_json::to_string(data).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::SerializationError,
                format!("Failed to serialize schema attributes, err: {}", err),
            )
        })?;

        let anoncreds = Arc::clone(profile).inject_anoncreds();
        let (schema_id, schema_json) = anoncreds
            .issuer_create_schema(submitter_did, name, version, &data_str)
            .await?;

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

    pub async fn create_from_ledger_json(
        profile: &Arc<dyn Profile>,
        source_id: &str,
        schema_id: &str,
    ) -> VcxResult<Self> {
        let ledger = Arc::clone(profile).inject_ledger();
        let schema_json = ledger.get_schema(schema_id, None).await?;
        let schema_data: SchemaData = serde_json::from_str(&schema_json).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Cannot deserialize schema: {}", err),
            )
        })?;

        Ok(Self {
            source_id: source_id.to_string(),
            schema_id: schema_id.to_string(),
            schema_json,
            name: schema_data.name,
            version: schema_data.version,
            data: schema_data.attr_names,
            submitter_did: "".to_string(),
            state: PublicEntityStateType::Published,
        })
    }

    pub async fn publish(self, profile: &Arc<dyn Profile>, endorser_did: Option<String>) -> VcxResult<Self> {
        trace!("Schema::publish >>>");

        if settings::indy_mocks_enabled() {
            return Ok(Self {
                state: PublicEntityStateType::Published,
                ..self
            });
        }

        let ledger = Arc::clone(profile).inject_ledger();
        ledger
            .publish_schema(&self.schema_json, &self.submitter_did, endorser_did)
            .await?;

        Ok(Self {
            state: PublicEntityStateType::Published,
            ..self
        })
    }

    pub fn get_source_id(&self) -> String {
        self.source_id.clone()
    }

    pub fn get_schema_id(&self) -> String {
        self.schema_id.clone()
    }

    pub fn to_string_versioned(&self) -> VcxResult<String> {
        ObjectWithVersion::new(DEFAULT_SERIALIZE_VERSION, self.to_owned())
            .serialize()
            .map_err(|err: AriesVcxError| err.extend("Cannot serialize Schema"))
    }

    pub fn from_string_versioned(data: &str) -> VcxResult<Schema> {
        ObjectWithVersion::deserialize(data)
            .map(|obj: ObjectWithVersion<Schema>| obj.data)
            .map_err(|err: AriesVcxError| err.extend("Cannot deserialize Schema"))
    }

    pub async fn update_state(&mut self, profile: &Arc<dyn Profile>) -> VcxResult<u32> {
        let ledger = Arc::clone(profile).inject_ledger();
        if ledger.get_schema(&self.schema_id, None).await.is_ok() {
            self.state = PublicEntityStateType::Published
        }
        Ok(self.state as u32)
    }

    pub async fn get_schema_json(&self, profile: &Arc<dyn Profile>) -> VcxResult<String> {
        if !self.schema_json.is_empty() {
            Ok(self.schema_json.clone())
        } else {
            let ledger = Arc::clone(profile).inject_ledger();
            Ok(ledger.get_schema(&self.schema_id, None).await?)
        }
    }

    pub fn get_state(&self) -> u32 {
        self.state as u32
    }
}
