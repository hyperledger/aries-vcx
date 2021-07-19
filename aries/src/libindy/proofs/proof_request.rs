use std::collections::HashMap;
use std::vec::Vec;

use serde_json;

use crate::error::prelude::*;
use crate::libindy::proofs::proof_request_internal::{AttrInfo, NonRevokedInterval, PredicateInfo};
use crate::libindy::utils::anoncreds;
use crate::utils::qualifier;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ProofRequestData {
    pub nonce: String,
    pub name: String,
    #[serde(rename = "version")]
    pub data_version: String,
    #[serde(default)]
    pub requested_attributes: HashMap<String, AttrInfo>,
    #[serde(default)]
    pub requested_predicates: HashMap<String, PredicateInfo>,
    pub non_revoked: Option<NonRevokedInterval>,
}

impl ProofRequestData {
    const DEFAULT_VERSION: &'static str = "1.0";

    pub fn create() -> ProofRequestData {
        ProofRequestData::default()
    }

    pub fn set_name(mut self, name: String) -> ProofRequestData {
        self.name = name;
        self
    }

    pub fn set_nonce(mut self) -> VcxResult<ProofRequestData> {
        self.nonce = anoncreds::generate_nonce()?;
        Ok(self)
    }

    pub fn set_requested_attributes(mut self, requested_attrs: String) -> VcxResult<ProofRequestData> {
        match serde_json::from_str::<HashMap<String, AttrInfo>>(&requested_attrs) {
            Ok(attrs) => self.requested_attributes = attrs,
            Err(err) => {
                let requested_attributes: Vec<AttrInfo> = ::serde_json::from_str(&requested_attrs)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Invalid Requested Attributes: {:?}, err: {:?}", requested_attrs, err)))?;
                for attribute in requested_attributes.iter() {
                    if attribute.name.is_some() && attribute.names.is_some() {
                        return Err(VcxError::from_msg(VcxErrorKind::InvalidProofRequest,
                                                      format!("Requested attribute can contain either 'name' or 'names'. Not both.")));
                    };
                }
                self.requested_attributes = requested_attributes
                    .into_iter()
                    .enumerate()
                    .map(|(index, attribute)| (format!("attribute_{}", index), attribute))
                    .collect();
            }
        }
        Ok(self)
    }

    pub fn set_requested_predicates(mut self, requested_predicates: String) -> VcxResult<ProofRequestData> {
        let requested_predicates: Vec<PredicateInfo> = ::serde_json::from_str(&requested_predicates)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Invalid Requested Predicates: {:?}, err: {:?}", requested_predicates, err)))?;

        self.requested_predicates = requested_predicates
            .into_iter()
            .enumerate()
            .map(|(index, attribute)| (format!("predicate_{}", index), attribute))
            .collect();
        Ok(self)
    }

    pub fn set_not_revoked_interval(mut self, non_revoc_interval: String) -> VcxResult<ProofRequestData> {
        let non_revoc_interval: NonRevokedInterval = ::serde_json::from_str(&non_revoc_interval)
            .map_err(|_| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Invalid Revocation Interval: {:?}", non_revoc_interval)))?;

        self.non_revoked = match (non_revoc_interval.from, non_revoc_interval.to) {
            (None, None) => None,
            (from, to) => Some(NonRevokedInterval { from, to })
        };

        Ok(self)
    }
}

impl Default for ProofRequestData {
    fn default() -> ProofRequestData {
        ProofRequestData {
            nonce: String::new(),
            name: String::new(),
            data_version: String::from(ProofRequestData::DEFAULT_VERSION),
            requested_attributes: HashMap::new(),
            requested_predicates: HashMap::new(),
            non_revoked: None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils;
    use crate::utils::constants::{REQUESTED_ATTRS, REQUESTED_PREDICATES};
    use crate::utils::devsetup::SetupDefaults;
    use crate::utils::mockdata::mockdata_proof;

    use super::*;
    use serde_json::Value;

    fn _expected_req_attrs() -> HashMap<String, AttrInfo> {
        let mut check_req_attrs: HashMap<String, AttrInfo> = HashMap::new();

        let attr_info1: AttrInfo = serde_json::from_str(mockdata_proof::ATTR_INFO_1).unwrap();
        let attr_info2: AttrInfo = serde_json::from_str(mockdata_proof::ATTR_INFO_2).unwrap();

        check_req_attrs.insert("attribute_0".to_string(), attr_info1);
        check_req_attrs.insert("attribute_1".to_string(), attr_info2);

        check_req_attrs
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_proof_request_msg() {
        let _setup = SetupDefaults::init();

        let request = ProofRequestData::create()
            .set_name("Test".into())
            .set_nonce().unwrap()
            .set_not_revoked_interval(r#"{"from":1100000000, "to": 1600000000}"#.into()).unwrap()
            .set_requested_attributes(REQUESTED_ATTRS.into()).unwrap()
            .set_requested_predicates(REQUESTED_PREDICATES.into()).unwrap();

        let serialized_msg = serde_json::to_string(&request).unwrap();
        warn!("serialized_msg: {}", serialized_msg);
        // todo: Does it really need to have both "version" and "ver" field?
        assert!(serialized_msg.contains(r#""name":"Test","version":"1.0""#));
        assert!(serialized_msg.contains(r#""non_revoked":{"from":1100000000,"to":1600000000}"#));
        let msg_as_value: Value = serde_json::from_str(&serialized_msg).unwrap();
        assert_eq!(msg_as_value["requested_attributes"]["attribute_0"]["name"], "age");
        assert_eq!(msg_as_value["requested_attributes"]["attribute_1"]["name"], "name");
        assert_eq!(msg_as_value["requested_predicates"]["predicate_0"]["name"], "age");
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_requested_attrs_constructed_correctly() {
        let _setup = SetupDefaults::init();

        let request = ProofRequestData::create()
            .set_nonce().unwrap()
            .set_requested_attributes(REQUESTED_ATTRS.into()).unwrap();
        assert_eq!(request.requested_attributes, _expected_req_attrs());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_requested_attrs_constructed_correctly_preformatted() {
        let _setup = SetupDefaults::init();

        let expected_req_attrs = _expected_req_attrs();
        let req_attrs_string = serde_json::to_string(&expected_req_attrs).unwrap();

        let request = ProofRequestData::create()
            .set_nonce().unwrap()
            .set_requested_attributes(req_attrs_string).unwrap();
        assert_eq!(request.requested_attributes, expected_req_attrs);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_requested_predicates_constructed_correctly() {
        let _setup = SetupDefaults::init();

        let mut check_predicates: HashMap<String, PredicateInfo> = HashMap::new();
        let attr_info1: PredicateInfo = serde_json::from_str(r#"{
            "name": "age",
            "p_type": "GE",
            "p_value": 22,
            "restrictions": [
                {
                    "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11",
                    "schema_name": "Faber Student Info",
                    "schema_version": "1.0",
                    "schema_issuer_did": "6XFh8yBzrpJQmNyZzgoTqB",
                    "issuer_did": "8XFh8yBzrpJQmNyZzgoTqB",
                    "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766"
                },
                {
                    "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11",
                    "schema_name": "BYU Student Info",
                    "schema_version": "1.0",
                    "schema_issuer_did": "5XFh8yBzrpJQmNyZzgoTqB",
                    "issuer_did": "66Fh8yBzrpJQmNyZzgoTqB",
                    "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766"
                }
            ]
        }"#).unwrap();
        check_predicates.insert("predicate_0".to_string(), attr_info1);

        let request = ProofRequestData::create()
            .set_nonce().unwrap()
            .set_requested_predicates(REQUESTED_PREDICATES.into()).unwrap();
        assert_eq!(request.requested_predicates, check_predicates);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_requested_attrs_constructed_correctly_for_names() {
        let _setup = SetupDefaults::init();

        let attr_info = json!({
          "names": ["name", "age", "email"],
          "restrictions": [{"schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11"}]
        });
        let attr_info_2 = json!({
          "name":"name",
          "restrictions": [{"schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11" }]
        });

        let requested_attrs = json!([ attr_info, attr_info_2 ]).to_string();

        let request = ProofRequestData::create()
            .set_nonce().unwrap()
            .set_requested_attributes(requested_attrs.into()).unwrap();

        let mut expected_req_attrs: HashMap<String, AttrInfo> = HashMap::new();
        expected_req_attrs.insert("attribute_0".to_string(), serde_json::from_value(attr_info).unwrap());
        expected_req_attrs.insert("attribute_1".to_string(), serde_json::from_value(attr_info_2).unwrap());
        assert_eq!(request.requested_attributes, expected_req_attrs);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_should_return_error_if_name_and_names_passed_together() {
        let _setup = SetupDefaults::init();

        let attr_info = json!({
          "name": "name",
          "names": ["name", "age", "email"],
          "restrictions": [{"schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11"}]
        });

        let requested_attrs = json!([ attr_info ]).to_string();

        let err = ProofRequestData::create()
            .set_nonce().unwrap()
            .set_requested_attributes(requested_attrs.into()).unwrap_err();

        assert_eq!(VcxErrorKind::InvalidProofRequest, err.kind());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_indy_proof_req_parses_correctly() {
        let _setup = SetupDefaults::init();

        let _proof_req: ProofRequestData = serde_json::from_str(utils::constants::INDY_PROOF_REQ_JSON).unwrap();
    }
}
