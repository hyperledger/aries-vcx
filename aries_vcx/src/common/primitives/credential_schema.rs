use std::sync::Arc;

use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds,
    ledger::base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite},
    wallet::base_wallet::BaseWallet,
};

use super::credential_definition::PublicEntityStateType;
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    utils::{constants::DEFAULT_SERIALIZE_VERSION, serialization::ObjectWithVersion},
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
    pub schema_json: String, // added in 0.45.0, #[serde(default)] use for backwards compatibility
}

impl Schema {
    pub async fn create(
        anoncreds: &impl BaseAnonCreds,
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

        let data_str = serde_json::to_string(data).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::SerializationError,
                format!("Failed to serialize schema attributes, err: {}", err),
            )
        })?;

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

    pub async fn submitter_did(&self) -> String {
        self.submitter_did.clone()
    }

    pub async fn publish(
        self,
        wallet: &impl BaseWallet,
        ledger: &impl AnoncredsLedgerWrite,
    ) -> VcxResult<Self> {
        trace!("Schema::publish >>>");

        ledger
            .publish_schema(wallet, &self.schema_json, &self.submitter_did, None)
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

    pub async fn update_state(&mut self, ledger: &impl AnoncredsLedgerRead) -> VcxResult<u32> {
        if ledger.get_schema(&self.schema_id, None).await.is_ok() {
            self.state = PublicEntityStateType::Published
        }
        Ok(self.state as u32)
    }

    pub async fn get_schema_json(
        &self,
        ledger: &Arc<dyn AnoncredsLedgerRead>,
    ) -> VcxResult<String> {
        if !self.schema_json.is_empty() {
            Ok(self.schema_json.clone())
        } else {
            Ok(ledger.get_schema(&self.schema_id, None).await?)
        }
    }

    pub fn get_state(&self) -> u32 {
        self.state as u32
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::utils::constants::SCHEMA_ID;

    #[test]
    fn test_to_string_versioned() {
        let schema = Schema {
            data: vec!["name".to_string(), "age".to_string()],
            version: "1.0".to_string(),
            schema_id: SCHEMA_ID.to_string(),
            name: "test".to_string(),
            source_id: "1".to_string(),
            ..Schema::default()
        };
        let serialized = schema.to_string_versioned().unwrap();
        assert!(serialized.contains(r#""version":"1.0""#));
        assert!(serialized.contains(r#""name":"test""#));
        assert!(serialized.contains(r#""schema_id":""#));
        assert!(serialized.contains(r#""source_id":"1""#));
        assert!(serialized.contains(r#""data":["name","age"]"#));
    }

    #[test]
    fn test_from_string_versioned() {
        let serialized = r#"
{
  "version": "1.0",
  "data": {
    "data": [
      "name",
      "age"
    ],
    "version": "1.0",
    "schema_id": "test_schema_id",
    "name": "test",
    "source_id": "1",
    "submitter_did": "",
    "state": 1,
    "schema_json": ""
  }
}
"#;
        let schema_result = Schema::from_string_versioned(serialized);
        assert!(schema_result.is_ok());
        let schema = schema_result.unwrap();

        assert_eq!(schema.version, "1.0");
        assert_eq!(schema.data, vec!["name".to_string(), "age".to_string()]);
        assert_eq!(schema.schema_id, "test_schema_id");
        assert_eq!(schema.name, "test");
        assert_eq!(schema.source_id, "1");
        assert_eq!(schema.state, PublicEntityStateType::Published);
    }

    #[test]
    fn test_get_schema_id() {
        let schema = Schema {
            data: vec!["name".to_string(), "age".to_string()],
            version: "1.0".to_string(),
            schema_id: SCHEMA_ID.to_string(),
            name: "test".to_string(),
            source_id: "1".to_string(),
            ..Schema::default()
        };
        assert_eq!(schema.get_schema_id(), SCHEMA_ID);
    }
}
