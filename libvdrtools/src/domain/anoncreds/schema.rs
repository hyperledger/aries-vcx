use std::collections::{HashMap, HashSet};

use indy_api_types::{
    errors::{IndyErrorKind, IndyResult},
    IndyError,
};

use super::{super::crypto::did::DidValue, indy_identifiers, DELIMITER};
use crate::utils::qualifier;

pub const MAX_ATTRIBUTES_COUNT: usize = 125;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SchemaV1 {
    pub id: SchemaId,
    pub name: String,
    pub version: String,
    #[serde(rename = "attrNames")]
    pub attr_names: AttributeNames,
    pub seq_no: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "ver")]
pub enum Schema {
    #[serde(rename = "1.0")]
    SchemaV1(SchemaV1),
}

impl Schema {
    pub fn to_unqualified(self) -> Schema {
        match self {
            Schema::SchemaV1(schema) => Schema::SchemaV1(SchemaV1 {
                id: schema.id.to_unqualified(),
                name: schema.name,
                version: schema.version,
                attr_names: schema.attr_names,
                seq_no: schema.seq_no,
            }),
        }
    }
}

impl From<Schema> for SchemaV1 {
    fn from(schema: Schema) -> Self {
        match schema {
            Schema::SchemaV1(schema) => schema,
        }
    }
}

pub type Schemas = HashMap<SchemaId, Schema>;

pub fn schemas_map_to_schemas_v1_map(schemas: Schemas) -> HashMap<SchemaId, SchemaV1> {
    schemas
        .into_iter()
        .map(|(schema_id, schema)| (schema_id, SchemaV1::from(schema)))
        .collect()
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AttributeNames(pub HashSet<String>);

impl AttributeNames {
    pub fn new() -> Self {
        AttributeNames(HashSet::new())
    }
}

impl From<HashSet<String>> for AttributeNames {
    fn from(attrs: HashSet<String>) -> Self {
        AttributeNames(attrs)
    }
}

impl From<AttributeNames> for HashSet<String> {
    fn from(value: AttributeNames) -> HashSet<String> {
        value.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaId(pub String);

impl SchemaId {
    pub const PREFIX: &'static str = "/anoncreds/v0/SCHEMA/";

    pub fn get_method(&self) -> Option<String> {
        qualifier::method(&self.0)
    }

    pub fn new(did: &DidValue, name: &str, version: &str) -> IndyResult<SchemaId> {
        const MARKER: &str = "2";
        match did.get_method() {
            Some(method) if method.starts_with("indy") => Ok(SchemaId(format!(
                "{}{}{}/{}",
                did.0,
                Self::PREFIX,
                name,
                version
            ))),
            Some(_method) => Err(IndyError::from_msg(
                IndyErrorKind::InvalidStructure,
                "Unsupported DID method",
            )),
            None => Ok(SchemaId(format!(
                "{}:{}:{}:{}",
                did.0, MARKER, name, version
            ))),
        }
    }

    pub fn parts(&self) -> Option<(DidValue, String, String)> {
        trace!("SchemaId::parts >> {:?}", self.0);
        if let Some((did, name, ver)) = indy_identifiers::try_parse_indy_schema_id(&self.0) {
            return Some((DidValue(did), name, ver));
        }

        let parts = self.0.split_terminator(DELIMITER).collect::<Vec<&str>>();

        if parts.len() == 1 {
            // 1
            return None;
        }

        if parts.len() == 4 {
            // NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0
            let did = parts[0].to_string();
            let name = parts[2].to_string();
            let version = parts[3].to_string();
            return Some((DidValue(did), name, version));
        }

        if parts.len() == 8 {
            // schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0
            let did = parts[2..5].join(DELIMITER);
            let name = parts[6].to_string();
            let version = parts[7].to_string();
            return Some((DidValue(did), name, version));
        }

        None
    }

    pub fn qualify(&self, method: &str) -> IndyResult<SchemaId> {
        match self.parts() {
            Some((did, name, version)) => SchemaId::new(&did.qualify(method), &name, &version),
            None => Ok(self.clone()),
        }
    }

    pub fn to_unqualified(&self) -> SchemaId {
        trace!("SchemaId::to_unqualified >> {}", &self.0);
        match self.parts() {
            Some((did, name, version)) => {
                trace!(
                    "SchemaId::to_unqualified: parts {:?}",
                    (&did, &name, &version)
                );
                SchemaId::new(&did.to_unqualified(), &name, &version)
                    .expect("Can't create unqualified SchemaId")
            }
            None => self.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _did() -> DidValue {
        DidValue("NcYxiDXkpYi6ov5FcYDi1e".to_string())
    }

    fn _did_qualified() -> DidValue {
        DidValue("did:indy:sovrin:builder:NcYxiDXkpYi6ov5FcYDi1e".to_string())
    }

    fn _schema_id_seq_no() -> SchemaId {
        SchemaId("1".to_string())
    }

    fn _schema_id_unqualified() -> SchemaId {
        SchemaId("NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0".to_string())
    }

    fn _schema_id_qualified() -> SchemaId {
        SchemaId(
            "did:indy:sovrin:builder:NcYxiDXkpYi6ov5FcYDi1e/anoncreds/v0/SCHEMA/gvt/1.0"
                .to_string(),
        )
    }

    fn _schema_id_invalid() -> SchemaId {
        SchemaId("NcYxiDXkpYi6ov5FcYDi1e:2".to_string())
    }

    mod to_unqualified {
        use super::*;

        #[test]
        fn test_schema_id_unqualify_for_id_as_seq_no() {
            assert_eq!(_schema_id_seq_no(), _schema_id_seq_no().to_unqualified());
        }

        #[test]
        fn test_schema_id_parts_for_id_as_unqualified() {
            assert_eq!(
                _schema_id_unqualified(),
                _schema_id_unqualified().to_unqualified()
            );
        }

        #[test]
        fn test_schema_id_parts_for_id_as_qualified() {
            assert_eq!(
                _schema_id_unqualified(),
                _schema_id_qualified().to_unqualified()
            );
        }

        #[test]
        fn test_schema_id_parts_for_invalid_unqualified() {
            assert_eq!(_schema_id_invalid(), _schema_id_invalid().to_unqualified());
        }
    }

    mod parts {
        use super::*;

        #[test]
        fn test_schema_id_parts_for_id_as_seq_no() {
            assert!(_schema_id_seq_no().parts().is_none());
        }

        #[test]
        fn test_schema_id_parts_for_id_as_unqualified() {
            let (did, _, _) = _schema_id_unqualified().parts().unwrap();
            assert_eq!(_did(), did);
        }

        #[test]
        fn test_schema_id_parts_for_id_as_qualified() {
            let (did, _, _) = _schema_id_qualified().parts().unwrap();
            assert_eq!(_did_qualified(), did);
        }

        #[test]
        fn test_schema_id_parts_for_invalid_unqualified() {
            assert!(_schema_id_invalid().parts().is_none());
        }
    }
}
