use crate::{
    data_types::identifiers::{issuer_id::IssuerId, schema_id::SchemaId},
    utils::validation::Validatable,
};

use std::collections::HashSet;

pub const MAX_ATTRIBUTES_COUNT: usize = 125;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    pub id: SchemaId,
    pub seq_no: Option<u32>,
    pub name: String,
    pub version: String,
    pub attr_names: AttributeNames,
    pub issuer_id: IssuerId,
}

// QUESTION: If these must be unique, why not directly store them as a set?
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AttributeNames(pub Vec<String>);

impl From<&[&str]> for AttributeNames {
    fn from(attrs: &[&str]) -> Self {
        Self(attrs.iter().map(|s| String::from(*s)).collect::<Vec<_>>())
    }
}

impl From<Vec<String>> for AttributeNames {
    fn from(attrs: Vec<String>) -> Self {
        Self(attrs)
    }
}

impl From<HashSet<String>> for AttributeNames {
    fn from(attrs: HashSet<String>) -> Self {
        Self(attrs.into_iter().collect::<Vec<_>>())
    }
}

impl From<AttributeNames> for HashSet<String> {
    fn from(value: AttributeNames) -> Self {
        value.0.into_iter().collect::<HashSet<String>>()
    }
}

impl From<AttributeNames> for Vec<String> {
    fn from(a: AttributeNames) -> Self {
        a.0
    }
}

impl Validatable for Schema {
    fn validate(&self) -> Result<(), crate::error::Error> {
        self.issuer_id.validate()?;
        self.attr_names.validate()?;
        Ok(())
    }
}

impl Validatable for AttributeNames {
    fn validate(&self) -> Result<(), crate::error::Error> {
        let mut unique = HashSet::new();
        let is_unique = self.0.iter().all(move |name| unique.insert(name));

        if !is_unique {
            return Err(crate::error::Error::from_msg(
                crate::error::ErrorKind::Input,
                "Attributes inside the schema must be unique",
            ));
        }

        if self.0.is_empty() {
            return Err(crate::error::Error::from_msg(
                crate::error::ErrorKind::Input,
                "Empty list of Schema attributes has been passed",
            ));
        }

        if self.0.len() > MAX_ATTRIBUTES_COUNT {
            return Err(crate::error::Error::from_msg(
                crate::error::ErrorKind::Input,
                format!(
                    "The number of Schema attributes {} cannot be greater than {}",
                    self.0.len(),
                    MAX_ATTRIBUTES_COUNT
                ),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod test_schema_validation {
    use super::*;

    #[test]
    fn test_schema_valid() {
        let schema_json = json!({
            "name": "gvt",
            "version": "1.0",
            "attrNames": ["aaa", "bbb", "ccc"],
            "issuerId": "mock:uri"
        });

        let schema: Schema = serde_json::from_value(schema_json).unwrap();
        assert_eq!(schema.name, "gvt");
        assert_eq!(schema.version, "1.0");
    }

    #[test]
    fn test_attribute_names_valid_ordering_consistent() {
        // This test runs 10 times as the ordering can accidentally match
        for _ in 0..10 {
            let one: &[&str] = &["a", "b", "c", "d"];
            let two: &[&str] = &["1", "2", "3", "4"];

            let attr_names_one: AttributeNames = one.into();
            let attr_names_two: AttributeNames = two.into();

            assert_eq!(attr_names_one.0, one);
            assert_eq!(attr_names_two.0, two);
        }
    }

    #[test]
    fn test_schema_invalid_missing_properties() {
        let schema_json = json!({
            "name": "gvt",
        });

        let schema = serde_json::from_value::<Schema>(schema_json);
        assert!(schema.is_err());
    }

    #[test]
    fn test_schema_invalid_issuer_id() {
        let schema_json = json!({
            "name": "gvt",
            "version": "1.0",
            "attrNames": ["aaa", "bbb", "ccc"],
            "issuerId": "bob"
        });

        let schema: Schema = serde_json::from_value(schema_json).unwrap();
        assert!(schema.validate().is_err());
    }

    #[test]
    fn test_schema_invalid_attr_names() {
        let schema_json = json!({
            "name": "gvt1",
            "version": "1.0",
            "attrNames": [],
            "issuerId": "mock:uri"
        });

        let schema: Schema = serde_json::from_value(schema_json).unwrap();
        assert!(schema.validate().is_err());
    }
}
