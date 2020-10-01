use std::collections::HashMap;

use error::{VcxError, VcxErrorKind, VcxResult};
use utils::error;
use utils::openssl::encode;

pub fn encode_attributes(attributes: &str) -> VcxResult<String> {
    let mut attributes: HashMap<String, serde_json::Value> = serde_json::from_str(attributes)
        .map_err(|err| {
            warn!("Invalid Json for Attribute data");
            VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize credential attributes: {}", err))
        })?;

    let mut dictionary = HashMap::new();

    for (attr, attr_data) in attributes.iter_mut() {
        let first_attr: &str = match attr_data {
            // old style input such as {"address2":["101 Wilson Lane"]}
            serde_json::Value::Array(array_type) => {
                let attrib_value: &str = match array_type.get(0).and_then(serde_json::Value::as_str) {
                    Some(x) => x,
                    None => {
                        warn!("Cannot encode attribute: {}", error::INVALID_ATTRIBUTES_STRUCTURE.message);
                        return Err(VcxError::from_msg(VcxErrorKind::InvalidAttributesStructure, "Attribute value not found"));
                    }
                };

                warn!("Old attribute format detected. See vcx_issuer_create_credential api for additional information.");
                attrib_value
            }

            // new style input such as {"address2":"101 Wilson Lane"}
            serde_json::Value::String(str_type) => str_type,
            // anything else is an error
            _ => {
                warn!("Invalid Json for Attribute data");
                return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, "Invalid Json for Attribute data"));
            }
        };

        let encoded = encode(&first_attr)?;
        let attrib_values = json!({
            "raw": first_attr,
            "encoded": encoded
        });

        dictionary.insert(attr, attrib_values);
    }

    serde_json::to_string_pretty(&dictionary)
        .map_err(|err| {
            warn!("Invalid Json for Attribute data");
            VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Invalid Json for Attribute data: {}", err))
        })
}


#[cfg(test)]
pub mod tests {
    use serde_json::Value;
    use ::{settings};
    
    #[allow(unused_imports)]
    use utils::{constants::*,
                get_temp_dir_path,
                libindy::{anoncreds::{libindy_create_and_store_credential_def,
                                      libindy_issuer_create_credential_offer,
                                      libindy_prover_create_credential_req},
                          LibindyMock,
                          wallet, wallet::get_wallet_handle},
    };
    use utils::devsetup::*;
    
    
    

    use super::*;

    static DEFAULT_CREDENTIAL_NAME: &str = "Credential";
    static DEFAULT_CREDENTIAL_ID: &str = "defaultCredentialId";

    static CREDENTIAL_DATA: &str =
        r#"{"address2":["101 Wilson Lane"],
        "zip":["87121"],
        "state":["UT"],
        "city":["SLC"],
        "address1":["101 Tela Lane"]
        }"#;

    pub fn util_put_credential_def_in_issuer_wallet(_schema_seq_num: u32, _wallet_handle: i32) {
        let issuer_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let tag = "test_tag";
        let config = "{support_revocation: false}";

        libindy_create_and_store_credential_def(&issuer_did, SCHEMAS_JSON, tag, None, config).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_encode_with_several_attributes_success() {
        let _setup = SetupDefaults::init();
        settings::set_config_value(settings::CONFIG_PROTOCOL_TYPE, "4.0");

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
        static TEST_CREDENTIAL_DATA: &str =
            r#"{"address2":["101 Wilson Lane"],
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
    #[cfg(feature = "general_test")]
    fn test_encode_with_one_attribute_success() {
        let _setup = SetupDefaults::init();
        settings::set_config_value(settings::CONFIG_PROTOCOL_TYPE, "4.0");

        let expected = json!({
            "address2": {
                "encoded": "68086943237164982734333428280784300550565381723532936263016368251445461241953",
                "raw": "101 Wilson Lane"
            }
        });

        static TEST_CREDENTIAL_DATA: &str =
            r#"{"address2":["101 Wilson Lane"]}"#;

        let expected_json = serde_json::to_string_pretty(&expected).unwrap();

        let results_json = encode_attributes(TEST_CREDENTIAL_DATA).unwrap();

        assert_eq!(expected_json, results_json, "encode_attributes failed to return expected results");
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_encode_with_new_format_several_attributes_success() {
        let _setup = SetupDefaults::init();
        settings::set_config_value(settings::CONFIG_PROTOCOL_TYPE, "4.0");

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

        static TEST_CREDENTIAL_DATA: &str =
            r#"{"address2":"101 Wilson Lane",
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
    #[cfg(feature = "general_test")]
    fn test_encode_with_new_format_one_attribute_success() {
        let _setup = SetupDefaults::init();
        settings::set_config_value(settings::CONFIG_PROTOCOL_TYPE, "4.0");

        let expected = json!({
            "address2": {
                "encoded": "68086943237164982734333428280784300550565381723532936263016368251445461241953",
                "raw": "101 Wilson Lane"
            }
        });

        static TEST_CREDENTIAL_DATA: &str =
            r#"{"address2": "101 Wilson Lane"}"#;

        let expected_json = serde_json::to_string_pretty(&expected).unwrap();

        let results_json = encode_attributes(TEST_CREDENTIAL_DATA).unwrap();

        assert_eq!(expected_json, results_json, "encode_attributes failed to return expected results");
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_encode_with_mixed_format_several_attributes_success() {
        let _setup = SetupDefaults::init();
        settings::set_config_value(settings::CONFIG_PROTOCOL_TYPE, "4.0");

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


        static TEST_CREDENTIAL_DATA: &str =
            r#"{"address2":["101 Wilson Lane"],
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
    #[cfg(feature = "general_test")]
    fn test_encode_bad_format_returns_error() {
        let _setup = SetupDefaults::init();
        settings::set_config_value(settings::CONFIG_PROTOCOL_TYPE, "4.0");

        static BAD_TEST_CREDENTIAL_DATA: &str =
            r#"{"format doesnt make sense"}"#;

        assert!(encode_attributes(BAD_TEST_CREDENTIAL_DATA).is_err())
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_encode_old_format_empty_array_error() {
        let _setup = SetupDefaults::init();
        settings::set_config_value(settings::CONFIG_PROTOCOL_TYPE, "4.0");

        static BAD_TEST_CREDENTIAL_DATA: &str =
            r#"{"address2":[]}"#;

        assert!(encode_attributes(BAD_TEST_CREDENTIAL_DATA).is_err())
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_encode_empty_field() {
        let _setup = SetupDefaults::init();
        settings::set_config_value(settings::CONFIG_PROTOCOL_TYPE, "4.0");

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
