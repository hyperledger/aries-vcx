use std::collections::HashMap;

use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    utils::openssl::encode,
};

pub fn encode_attributes(attributes: &str) -> VcxResult<String> {
    let mut dictionary = HashMap::new();
    match serde_json::from_str::<HashMap<String, serde_json::Value>>(attributes) {
        Ok(mut attributes) => {
            for (attr, attr_data) in attributes.iter_mut() {
                let first_attr = match attr_data {
                    // new style input such as {"address2":"101 Wilson Lane"}
                    serde_json::Value::String(str_type) => str_type,

                    // old style input such as {"address2":["101 Wilson Lane"]}
                    serde_json::Value::Array(array_type) => {
                        let attrib_value: &str = match array_type.get(0).and_then(serde_json::Value::as_str) {
                            Some(x) => x,
                            None => {
                                return Err(AriesVcxError::from_msg(
                                    AriesVcxErrorKind::InvalidAttributesStructure,
                                    "Attribute value not found",
                                ));
                            }
                        };

                        warn!(
                            "Old attribute format detected. See vcx_issuer_create_credential api for additional \
                             information."
                        );
                        attrib_value
                    }
                    _ => {
                        return Err(AriesVcxError::from_msg(
                            AriesVcxErrorKind::InvalidJson,
                            "Invalid Json for Attribute data",
                        ));
                    }
                };

                let encoded = encode(first_attr)?;
                let attrib_values = json!({
                    "raw": first_attr,
                    "encoded": encoded
                });

                dictionary.insert(attr.to_string(), attrib_values);
            }
            serde_json::to_string_pretty(&dictionary).map_err(|err| {
                warn!("Invalid Json for Attribute data");
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidJson,
                    format!("Invalid Json for Attribute data: {}", err),
                )
            })
        }
        Err(_err) => {
            // TODO: Check error type
            match serde_json::from_str::<Vec<serde_json::Value>>(attributes) {
                Ok(mut attributes) => {
                    for cred_value in attributes.iter_mut() {
                        let name = cred_value.get("name").ok_or(AriesVcxError::from_msg(
                            AriesVcxErrorKind::InvalidAttributesStructure,
                            format!("No 'name' field in cred_value: {:?}", cred_value),
                        ))?;
                        let value = cred_value.get("value").ok_or(AriesVcxError::from_msg(
                            AriesVcxErrorKind::InvalidAttributesStructure,
                            format!("No 'value' field in cred_value: {:?}", cred_value),
                        ))?;
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
                            .ok_or(AriesVcxError::from_msg(
                                AriesVcxErrorKind::InvalidAttributesStructure,
                                format!("Failed to convert attribute name {:?} to string", cred_value),
                            ))?
                            .to_string();
                        dictionary.insert(name, attrib_values);
                    }
                    serde_json::to_string_pretty(&dictionary).map_err(|err| {
                        warn!("Invalid Json for Attribute data");
                        AriesVcxError::from_msg(
                            AriesVcxErrorKind::InvalidJson,
                            format!("Invalid Json for Attribute data: {}", err),
                        )
                    })
                }
                Err(err) => Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidAttributesStructure,
                    format!("Attribute value not found: {:?}", err),
                )),
            }
        }
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use serde_json::Value;

    use crate::{common::credentials::encoding::encode_attributes, utils::devsetup::*};

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
        static TEST_CREDENTIAL_DATA: &str = r#"{"address2":["101 Wilson Lane"],
            "zip":["87121"],
            "state":["UT"],
            "city":["SLC"],
            "address1":["101 Tela Lane"]
            }"#;

        let results_json = encode_attributes(TEST_CREDENTIAL_DATA).unwrap();

        let results: Value = serde_json::from_str(&results_json).unwrap();
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

        static TEST_CREDENTIAL_DATA: &str = r#"{"address2":["101 Wilson Lane"]}"#;

        let expected_json = serde_json::to_string_pretty(&expected).unwrap();

        let results_json = encode_attributes(TEST_CREDENTIAL_DATA).unwrap();

        assert_eq!(
            expected_json, results_json,
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

        static TEST_CREDENTIAL_DATA: &str = r#"[
            {"name": "address2", "value": "101 Wilson Lane"},
            {"name": "zip", "value": "87121"},
            {"name": "state", "value": "UT"},
            {"name": "city", "value": "SLC"},
            {"name": "address1", "value": "101 Tela Lane"}
            ]"#;

        let results_json = encode_attributes(TEST_CREDENTIAL_DATA).unwrap();

        let results: Value = serde_json::from_str(&results_json).unwrap();
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

        static TEST_CREDENTIAL_DATA: &str = r#"{"address2":"101 Wilson Lane",
            "zip":"87121",
            "state":"UT",
            "city":"SLC",
            "address1":"101 Tela Lane"
            }"#;

        let results_json = encode_attributes(TEST_CREDENTIAL_DATA).unwrap();

        let results: Value = serde_json::from_str(&results_json).unwrap();
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

        static TEST_CREDENTIAL_DATA: &str = r#"{"address2": "101 Wilson Lane"}"#;

        let expected_json = serde_json::to_string_pretty(&expected).unwrap();

        let results_json = encode_attributes(TEST_CREDENTIAL_DATA).unwrap();

        assert_eq!(
            expected_json, results_json,
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

        static TEST_CREDENTIAL_DATA: &str = r#"{"address2":["101 Wilson Lane"],
            "zip":"87121",
            "state":"UT",
            "city":["SLC"],
            "address1":"101 Tela Lane"
            }"#;

        let results_json = encode_attributes(TEST_CREDENTIAL_DATA).unwrap();

        let results: Value = serde_json::from_str(&results_json).unwrap();
        assert_eq!(expected, results);
    }

    #[test]
    fn test_encode_bad_format_returns_error() {
        let _setup = SetupDefaults::init();

        static BAD_TEST_CREDENTIAL_DATA: &str = r#"{"format doesnt make sense"}"#;

        assert!(encode_attributes(BAD_TEST_CREDENTIAL_DATA).is_err())
    }

    #[test]
    fn test_encode_old_format_empty_array_error() {
        let _setup = SetupDefaults::init();

        static BAD_TEST_CREDENTIAL_DATA: &str = r#"{"address2":[]}"#;

        assert!(encode_attributes(BAD_TEST_CREDENTIAL_DATA).is_err())
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

        static TEST_CREDENTIAL_DATA: &str = r#"{"empty_field": ""}"#;

        let results_json = encode_attributes(TEST_CREDENTIAL_DATA).unwrap();

        let results: Value = serde_json::from_str(&results_json).unwrap();
        assert_eq!(expected, results);
    }
}
