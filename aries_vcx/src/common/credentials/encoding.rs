use std::collections::HashMap;
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};
use crate::utils::openssl::encode;

/// `CredentialAttribute` contains credential attributes data in a given state.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CredentialAttribute<State> {
    /// Raw credential attribute data.
    raw: String,

    /// The name of the attribute.
    #[serde(skip)]
    name: String,

    /// Encoded credential attribute data.
    encoded: String,

    #[serde(skip)]
    _marker: PhantomData<State>,
}

/// `RawAttributeValue` is a credential attribute raw state.
#[derive(Debug, Serialize, Deserialize)]
pub struct RawAttributeValue;

/// `RawAttributeValue` is a credential attribute in encoded state.
#[derive(Debug, Serialize, Deserialize)]
pub struct EncodedAttributeValue;

impl CredentialAttribute<EncodedAttributeValue> {
    /// Return the encoded value of the credential attribute.
    pub fn encoded(&self) -> &str {
        &self.encoded
    }
}

impl TryFrom<Value> for CredentialAttribute<RawAttributeValue> {
    type Error = AriesVcxError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let name = value.get("name").ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidAttributesStructure,
            format!("No 'name' field in cred_value: {:?}", value),
        ))?;
        let value = value.get("value").ok_or_else(|| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidAttributesStructure,
                format!("No 'value' field in cred_value: {:?}", value),
            )
        })?;

        let name = name
            .as_str()
            .ok_or_else(|| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidAttributesStructure,
                    format!("Failed to convert attribute name {:?} to string", value),
                )
            })?
            .to_string();

        let raw = value
            .as_str()
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidAttributesStructure,
                format!("Failed to convert value {:?} to string", value),
            ))
            .map(|v| v.to_string())?;

        Ok(CredentialAttribute {
            raw,
            name,
            encoded: String::new(),
            _marker: PhantomData::default(),
        })
    }
}

impl CredentialAttribute<RawAttributeValue> {
    pub fn with_name_value(name: String, value: Value) -> VcxResult<Self> {
        let raw = match &value {
            // new style input such as {"address2":"101 Wilson Lane"}
            serde_json::Value::String(str_type) => str_type.to_string(),

            // old style input such as {"address2":["101 Wilson Lane"]}
            serde_json::Value::Array(array_type) => {
                warn!(
                    "Old attribute format detected. See vcx_issuer_create_credential api for additional information."
                );
                array_type
                    .get(0)
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| {
                        AriesVcxError::from_msg(
                            AriesVcxErrorKind::InvalidAttributesStructure,
                            "Attribute value not found",
                        )
                    })
                    .map(|v| v.to_string())?
            }
            _ => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidJson,
                    "Invalid Json for Attribute data",
                ));
            }
        };

        Ok(CredentialAttribute {
            raw,
            name,
            encoded: String::new(),
            _marker: PhantomData::default(),
        })
    }

    /// Encodes a credential attribute.
    pub fn encode(self) -> VcxResult<CredentialAttribute<EncodedAttributeValue>> {
        let CredentialAttribute { raw, name, .. } = self;

        let encoded = encode(&raw)?;
        let cred_attr: CredentialAttribute<EncodedAttributeValue> = CredentialAttribute {
            name,
            raw,
            encoded,
            _marker: PhantomData::default(),
        };

        Ok(cred_attr)
    }
}

/// A list of all credential attributes.
pub struct CredentialAttributes<State> {
    /// A key/value pair of the attribute's name and the attribute itself.
    // Ideally this field should be a vec because `CredentialAttribute`
    // already has a `name` field.
    attributes: HashMap<String, CredentialAttribute<State>>,

    /// An encoded value of all the attributes.
    encoded: String,
}

impl CredentialAttributes<EncodedAttributeValue> {
    /// Return a reference to the encoded value of the attribute.
    pub fn encoded(&self) -> &str {
        &self.encoded
    }
}

impl CredentialAttributes<RawAttributeValue> {
    pub fn new(data: &str) -> VcxResult<Self> {
        let attributes: Vec<CredentialAttribute<RawAttributeValue>>;
        match serde_json::from_str::<HashMap<String, serde_json::Value>>(data) {
            Ok(attr_values) => {
                let result: VcxResult<_> = attr_values
                    .into_iter()
                    .map(|(name, value)| CredentialAttribute::with_name_value(name, value))
                    .collect();
                attributes = result?;
            }
            Err(_err) => {
                // TODO: Check error type
                match serde_json::from_str::<Vec<serde_json::Value>>(data) {
                    Ok(attr_values) => {
                        let result: VcxResult<_> = attr_values.into_iter().map(CredentialAttribute::try_from).collect();
                        attributes = result?
                    }
                    Err(err) => {
                        return Err(AriesVcxError::from_msg(
                            AriesVcxErrorKind::InvalidAttributesStructure,
                            format!("Attribute value not found: {:?}", err),
                        ));
                    }
                }
            }
        }
        let attributes = attributes.into_iter().map(|attr| (attr.name.clone(), attr)).collect();
        Ok(CredentialAttributes {
            attributes,
            encoded: String::new(),
        })
    }

    /// Encode all the attributes in the credential.
    ///
    /// # Error
    ///
    /// It returns an error if encoding any of the attributes in the credential
    /// fails.
    pub fn encode_all(self) -> VcxResult<CredentialAttributes<EncodedAttributeValue>> {
        let mut attributes = HashMap::with_capacity(self.attributes.len());
        for (name, attr) in self.attributes {
            let attr = attr.encode()?;
            attributes.insert(name, attr);
        }

        let encoded = serde_json::to_string_pretty(&attributes).map_err(|err| {
            warn!("Invalid Json for Attribute data");
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Invalid Json for Attribute data: {}", err),
            )
        })?;
        Ok(CredentialAttributes { attributes, encoded })
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use serde_json::Value;

    use super::*;
    use crate::utils::devsetup::*;

    #[test]
    fn test_encode_with_several_attributes_success() {
        let _setup = SetupDefaults::init();

        let expected = json!({
            "address2": {
                "encoded": "68086943237164982734333428280784300550565381723532936263016368251445461241953",
                "raw": "101 Wilson Lane"
            },
            "zip": {
                "encoded": "87121",
                "raw": "87121"
            },
            "city": {
                "encoded": "101327353979588246869873249766058188995681113722618593621043638294296500696424",
                "raw": "SLC"
            },
            "address1": {
                "encoded": "63690509275174663089934667471948380740244018358024875547775652380902762701972",
                "raw": "101 Tela Lane"
            },
            "state": {
                "encoded": "93856629670657830351991220989031130499313559332549427637940645777813964461231",
                "raw": "UT"
            }
        });
        let attributes = CredentialAttributes::new(
            r#"{"address2":["101 Wilson Lane"],
            "zip":["87121"],
            "state":["UT"],
            "city":["SLC"],
            "address1":["101 Tela Lane"]
            }"#,
        )
        .unwrap();

        let results = attributes.encode_all().unwrap();
        let results: Value = serde_json::from_str(&results.encoded()).unwrap();
        assert_eq!(expected, results);
    }

    #[test]
    fn test_encode_with_one_attribute_success() {
        let _setup = SetupDefaults::init();

        let expected = json!({
            "address2": {
                "encoded": "68086943237164982734333428280784300550565381723532936263016368251445461241953",
                "raw": "101 Wilson Lane"
            }
        });

        let attributes = CredentialAttributes::new(r#"{"address2":["101 Wilson Lane"]}"#).unwrap();
        let results = attributes.encode_all().unwrap();
        let got: Value = serde_json::from_str(&results.encoded()).unwrap();

        assert_eq!(expected, got, "encode_attributes failed to return expected results");
    }

    #[test]
    fn test_encode_with_aries_format_several_attributes_success() {
        let _setup = SetupDefaults::init();

        let expected = json!({
            "address2": {
                "encoded": "68086943237164982734333428280784300550565381723532936263016368251445461241953",
                "raw": "101 Wilson Lane"
            },
            "zip": {
                "encoded": "87121",
                "raw": "87121"
            },
            "city": {
                "encoded": "101327353979588246869873249766058188995681113722618593621043638294296500696424",
                "raw": "SLC"
            },
            "address1": {
                "encoded": "63690509275174663089934667471948380740244018358024875547775652380902762701972",
                "raw": "101 Tela Lane"
            },
            "state": {
                "encoded": "93856629670657830351991220989031130499313559332549427637940645777813964461231",
                "raw": "UT"
            }
        });

        let attributes = CredentialAttributes::new(
            r#"[
            {"name": "address2", "value": "101 Wilson Lane"},
            {"name": "zip", "value": "87121"},
            {"name": "state", "value": "UT"},
            {"name": "city", "value": "SLC"},
            {"name": "address1", "value": "101 Tela Lane"}
            ]"#,
        )
        .unwrap();

        let results = attributes.encode_all().unwrap();

        let results: Value = serde_json::from_str(&results.encoded()).unwrap();
        assert_eq!(expected, results);
    }

    #[test]
    fn test_encode_with_new_format_several_attributes_success() {
        let _setup = SetupDefaults::init();

        let expected = json!({
            "address2": {
                "encoded": "68086943237164982734333428280784300550565381723532936263016368251445461241953",
                "raw": "101 Wilson Lane"
            },
            "zip": {
                "encoded": "87121",
                "raw": "87121"
            },
            "city": {
                "encoded": "101327353979588246869873249766058188995681113722618593621043638294296500696424",
                "raw": "SLC"
            },
            "address1": {
                "encoded": "63690509275174663089934667471948380740244018358024875547775652380902762701972",
                "raw": "101 Tela Lane"
            },
            "state": {
                "encoded": "93856629670657830351991220989031130499313559332549427637940645777813964461231",
                "raw": "UT"
            }
        });

        let attributes = CredentialAttributes::new(
            r#"{"address2":"101 Wilson Lane",
            "zip":"87121",
            "state":"UT",
            "city":"SLC",
            "address1":"101 Tela Lane"
            }"#,
        )
        .unwrap();

        let results = attributes.encode_all().unwrap();

        let results: Value = serde_json::from_str(&results.encoded()).unwrap();
        assert_eq!(expected, results);
    }

    #[test]
    fn test_encode_with_new_format_one_attribute_success() {
        let _setup = SetupDefaults::init();

        let expected = json!({
            "address2": {
                "encoded": "68086943237164982734333428280784300550565381723532936263016368251445461241953",
                "raw": "101 Wilson Lane"
            }
        });

        let attributes = CredentialAttributes::new(r#"{"address2": "101 Wilson Lane"}"#).unwrap();
        let results = attributes.encode_all().unwrap();
        let got: Value = serde_json::from_str(&results.encoded()).unwrap();

        assert_eq!(expected, got, "encode_attributes failed to return expected results");
    }

    #[test]
    fn test_encode_with_mixed_format_several_attributes_success() {
        let _setup = SetupDefaults::init();

        //        for reference....expectation is encode_attributes returns this:

        let expected = json!({
            "address2": {
                "encoded": "68086943237164982734333428280784300550565381723532936263016368251445461241953",
                "raw": "101 Wilson Lane"
            },
            "zip": {
                "encoded": "87121",
                "raw": "87121"
            },
            "city": {
                "encoded": "101327353979588246869873249766058188995681113722618593621043638294296500696424",
                "raw": "SLC"
            },
            "address1": {
                "encoded": "63690509275174663089934667471948380740244018358024875547775652380902762701972",
                "raw": "101 Tela Lane"
            },
            "state": {
                "encoded": "93856629670657830351991220989031130499313559332549427637940645777813964461231",
                "raw": "UT"
            }
        });

        let attributes = CredentialAttributes::new(
            r#"{"address2":["101 Wilson Lane"],
            "zip":"87121",
            "state":"UT",
            "city":["SLC"],
            "address1":"101 Tela Lane"
            }"#,
        )
        .unwrap();

        let results = attributes.encode_all().unwrap();

        let results: Value = serde_json::from_str(&results.encoded()).unwrap();
        assert_eq!(expected, results);
    }

    #[test]
    #[should_panic(expected = "Attribute value not found")]
    fn test_create_credential_attribute_with_bad_format_returns_error() {
        let _setup = SetupDefaults::init();
        CredentialAttributes::new(r#"{"format doesnt make sense"}"#).unwrap();
    }

    #[test]
    #[should_panic(expected = "Attribute value not found")]
    fn test_encode_old_format_empty_array_error() {
        let _setup = SetupDefaults::init();

        let attributes = CredentialAttributes::new(r#"{"address2":[]}"#).unwrap();
        attributes.encode_all().unwrap();
    }

    #[test]
    fn test_encode_empty_field() {
        let _setup = SetupDefaults::init();

        let expected = json!({
            "empty_field": {
                "encoded": "102987336249554097029535212322581322789799900648198034993379397001115665086549",
                "raw": ""
            }
        });

        let attributes = CredentialAttributes::new(r#"{"empty_field": ""}"#).unwrap();

        let results = attributes.encode_all().unwrap();

        let results: Value = serde_json::from_str(&results.encoded()).unwrap();
        assert_eq!(expected, results);
    }
}
