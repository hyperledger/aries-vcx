use std::collections::HashMap;
use std::marker::PhantomData;

use serde_json::Value;

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};
use crate::utils::openssl::encode;

/// `CredentialAttribute` contains credential attributes data in a given state.
#[derive(Debug, Clone)]
pub struct CredentialAttribute<'r, State> {
    /// Raw credential attribute data.
    raw: &'r str,

    /// Encoded credential attribute data.
    encoded: Option<String>,

    _marker: PhantomData<State>,
}

/// `RawAttributeValue` is a credential attribute raw state.
pub struct RawAttributeValue;

/// `RawAttributeValue` is a credential attribute in encoded state.
pub struct EncodedAttributeValue;

impl<'raw> CredentialAttribute<'raw, RawAttributeValue> {
    /// Create new credential attribute.
    pub fn new(raw: &'raw str) -> Self {
        Self {
            raw,
            encoded: None,
            _marker: PhantomData::default(),
        }
    }
    /// Encodes the attributes in a credential.
    pub fn encode(self) -> VcxResult<CredentialAttribute<'raw, EncodedAttributeValue>> {
        let dictionary: HashMap<String, Value>;
        match serde_json::from_str::<HashMap<String, serde_json::Value>>(self.raw) {
            Ok(attributes) => {
                dictionary = Self::encode_table_attribute(attributes)?;
            }
            Err(_err) => {
                // TODO: Check error type
                match serde_json::from_str::<Vec<serde_json::Value>>(self.raw) {
                    Ok(attributes) => {
                        dictionary = Self::encode_attribute_list(attributes)?;
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
        let encoded = serde_json::to_string_pretty(&dictionary).map_err(|err| {
            warn!("Invalid Json for Attribute data");
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Invalid Json for Attribute data: {}", err),
            )
        })?;

        let attr = CredentialAttribute {
            raw: self.raw,
            encoded: Some(encoded),
            _marker: PhantomData::<EncodedAttributeValue>::default(),
        };
        Ok(attr)
    }

    /// Encodes attributes in a hashmap.
    fn encode_table_attribute(attributes: HashMap<String, Value>) -> VcxResult<HashMap<String, Value>> {
        let mut dictionary = HashMap::with_capacity(attributes.len());
        for (attr, attr_data) in attributes {
            let first_attr = match &attr_data {
                // new style input such as {"address2":"101 Wilson Lane"}
                serde_json::Value::String(str_type) => str_type,

                // old style input such as {"address2":["101 Wilson Lane"]}
                serde_json::Value::Array(array_type) => {
                    warn!("Old attribute format detected. See vcx_issuer_create_credential api for additional information.");
                    array_type.get(0).and_then(serde_json::Value::as_str).ok_or_else(|| {
                        AriesVcxError::from_msg(
                            AriesVcxErrorKind::InvalidAttributesStructure,
                            "Attribute value not found",
                        )
                    })?
                }
                _ => {
                    return Err(AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidJson,
                        "Invalid Json for Attribute data",
                    ));
                }
            };

            let attrib_values = json!({
                "raw": first_attr,
                "encoded": encode(first_attr)?
            });

            dictionary.insert(attr, attrib_values);
        }

        Ok(dictionary)
    }

    /// Encodes a list of attributes.
    fn encode_attribute_list(attributes: Vec<Value>) -> VcxResult<HashMap<String, Value>> {
        let mut dictionary = HashMap::with_capacity(attributes.len());
        for cred_value in attributes {
            let name = cred_value.get("name").ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidAttributesStructure,
                format!("No 'name' field in cred_value: {:?}", cred_value),
            ))?;
            let value = cred_value.get("value").ok_or_else(|| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidAttributesStructure,
                    format!("No 'value' field in cred_value: {:?}", cred_value),
                )
            })?;
            let encoded = encode(value.as_str().ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidAttributesStructure,
                format!("Failed to convert value {:?} to string", value),
            ))?)?;
            let attrib_values = json!({
                "raw": value,
                "encoded": encoded
            });
            let name = name
                .as_str()
                .ok_or_else(|| {
                    AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidAttributesStructure,
                        format!("Failed to convert attribute name {:?} to string", cred_value),
                    )
                })?
                .to_string();
            dictionary.insert(name, attrib_values);
        }

        Ok(dictionary)
    }
}

impl CredentialAttribute<'_, EncodedAttributeValue> {
    /// Return the encoded value of the credential attribute.
    pub fn encoded(&self) -> VcxResult<&str> {
        self.encoded
            .as_deref()
            .ok_or_else(|| AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, "raw attribute value"))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
        let attr_data = CredentialAttribute::new(
            r#"{"address2":["101 Wilson Lane"],
            "zip":["87121"],
            "state":["UT"],
            "city":["SLC"],
            "address1":["101 Tela Lane"]
            }"#,
        );

        let results = attr_data.encode().unwrap();
        let results: Value = serde_json::from_str(&results.encoded().unwrap()).unwrap();
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

        let attr_data = CredentialAttribute::new(r#"{"address2":["101 Wilson Lane"]}"#);

        let expected_json = serde_json::to_string_pretty(&expected).unwrap();

        let results = attr_data.encode().unwrap();

        assert_eq!(
            expected_json,
            results.encoded().unwrap(),
            "encode_attributes failed to return expected results"
        );
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

        let attr_data = CredentialAttribute::new(
            r#"[
            {"name": "address2", "value": "101 Wilson Lane"},
            {"name": "zip", "value": "87121"},
            {"name": "state", "value": "UT"},
            {"name": "city", "value": "SLC"},
            {"name": "address1", "value": "101 Tela Lane"}
            ]"#,
        );

        let results = attr_data.encode().unwrap();

        let results: Value = serde_json::from_str(&results.encoded().unwrap()).unwrap();
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

        let attr_data = CredentialAttribute::new(
            r#"{"address2":"101 Wilson Lane",
            "zip":"87121",
            "state":"UT",
            "city":"SLC",
            "address1":"101 Tela Lane"
            }"#,
        );

        let results = attr_data.encode().unwrap();

        let results: Value = serde_json::from_str(&results.encoded().unwrap()).unwrap();
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

        let attr_data = CredentialAttribute::new(r#"{"address2": "101 Wilson Lane"}"#);

        let expected_json = serde_json::to_string_pretty(&expected).unwrap();

        let results = attr_data.encode().unwrap();

        assert_eq!(
            expected_json,
            results.encoded().unwrap(),
            "encode_attributes failed to return expected results"
        );
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

        let attr_data = CredentialAttribute::new(
            r#"{"address2":["101 Wilson Lane"],
            "zip":"87121",
            "state":"UT",
            "city":["SLC"],
            "address1":"101 Tela Lane"
            }"#,
        );

        let results = attr_data.encode().unwrap();

        let results: Value = serde_json::from_str(&results.encoded().unwrap()).unwrap();
        assert_eq!(expected, results);
    }

    #[test]
    fn test_encode_bad_format_returns_error() {
        let _setup = SetupDefaults::init();

        let bad_attr_data = CredentialAttribute::new(r#"{"format doesnt make sense"}"#);

        assert!(bad_attr_data.encode().is_err())
    }

    #[test]
    fn test_encode_old_format_empty_array_error() {
        let _setup = SetupDefaults::init();

        let bad_attr_data = CredentialAttribute::new(r#"{"address2":[]}"#);

        assert!(bad_attr_data.encode().is_err())
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

        let bad_attr_data = CredentialAttribute::new(r#"{"empty_field": ""}"#);

        let results = bad_attr_data.encode().unwrap();

        let results: Value = serde_json::from_str(&results.encoded().unwrap()).unwrap();
        assert_eq!(expected, results);
    }
}
