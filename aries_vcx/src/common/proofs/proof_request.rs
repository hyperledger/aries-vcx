use std::{collections::HashMap, sync::Arc, vec::Vec};

use serde_json;

use super::proof_request_internal::{AttrInfo, NonRevokedInterval, PredicateInfo};
use crate::{core::profile::profile::Profile, errors::error::prelude::*};

#[derive(Serialize, Deserialize, Builder, Debug, PartialEq, Eq, Clone)]
#[builder(setter(into), default)]
pub struct ProofRequestData {
    pub nonce: String,
    pub name: String,
    #[serde(rename = "version")]
    pub data_version: String,
    #[serde(default)]
    pub requested_attributes: HashMap<String, AttrInfo>,
    #[serde(default)]
    pub requested_predicates: HashMap<String, PredicateInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_revoked: Option<NonRevokedInterval>,
}

impl ProofRequestData {
    const DEFAULT_VERSION: &'static str = "1.0";

    pub async fn create(profile: &Arc<dyn Profile>, name: &str) -> VcxResult<Self> {
        let nonce = Arc::clone(profile).inject_anoncreds().generate_nonce().await?;
        Ok(Self {
            name: name.to_string(),
            nonce,
            ..Self::default()
        })
    }

    pub fn set_requested_attributes_as_string(mut self, requested_attrs: String) -> VcxResult<Self> {
        match serde_json::from_str::<HashMap<String, AttrInfo>>(&requested_attrs) {
            Ok(attrs) => self.requested_attributes = attrs,
            Err(_err) => {
                let requested_attributes: Vec<AttrInfo> = ::serde_json::from_str(&requested_attrs).map_err(|err| {
                    AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidJson,
                        format!("Invalid Requested Attributes: {:?}, err: {:?}", requested_attrs, err),
                    )
                })?;
                for attribute in requested_attributes.iter() {
                    if attribute.name.is_some() && attribute.names.is_some() {
                        return Err(AriesVcxError::from_msg(
                            AriesVcxErrorKind::InvalidProofRequest,
                            "Requested attribute can contain either 'name' or 'names'. Not both.".to_string(),
                        ));
                    };
                }
                self = self.set_requested_attributes_as_vec(requested_attributes)?;
            }
        }
        Ok(self)
    }

    pub fn set_requested_attributes_as_vec(mut self, requested_attrs: Vec<AttrInfo>) -> VcxResult<Self> {
        self.requested_attributes = requested_attrs
            .into_iter()
            .enumerate()
            .map(|(index, attribute)| (format!("attribute_{}", index), attribute))
            .collect();
        Ok(self)
    }

    pub fn set_requested_predicates_as_string(mut self, requested_predicates: String) -> VcxResult<Self> {
        let requested_predicates: Vec<PredicateInfo> =
            ::serde_json::from_str(&requested_predicates).map_err(|err| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidJson,
                    format!(
                        "Invalid Requested Predicates: {:?}, err: {:?}",
                        requested_predicates, err
                    ),
                )
            })?;

        self.requested_predicates = requested_predicates
            .into_iter()
            .enumerate()
            .map(|(index, attribute)| (format!("predicate_{}", index), attribute))
            .collect();
        Ok(self)
    }

    pub fn set_not_revoked_interval(mut self, non_revoc_interval: String) -> VcxResult<Self> {
        let non_revoc_interval: NonRevokedInterval = ::serde_json::from_str(&non_revoc_interval).map_err(|_| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Invalid Revocation Interval: {:?}", non_revoc_interval),
            )
        })?;

        self.non_revoked = match (non_revoc_interval.from, non_revoc_interval.to) {
            (None, None) => None,
            (from, to) => Some(NonRevokedInterval { from, to }),
        };

        Ok(self)
    }
}

impl Default for ProofRequestData {
    fn default() -> Self {
        Self {
            nonce: String::new(),
            name: String::new(),
            data_version: String::from(ProofRequestData::DEFAULT_VERSION),
            requested_attributes: HashMap::new(),
            requested_predicates: HashMap::new(),
            non_revoked: None,
        }
    }
}

pub type PresentationRequestData = ProofRequestData;

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use super::*;

    pub fn _presentation_request_data() -> PresentationRequestData {
        PresentationRequestData::default()
            .set_requested_attributes_as_string(json!([{"name": "name"}]).to_string())
            .unwrap()
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {
    use serde_json::Value;

    use super::*;
    use crate::{
        common::test_utils::mock_profile,
        utils,
        utils::{
            constants::{REQUESTED_ATTRS, REQUESTED_PREDICATES},
            devsetup::SetupDefaults,
            mockdata::mockdata_proof,
        },
    };

    fn _expected_req_attrs() -> HashMap<String, AttrInfo> {
        let mut check_req_attrs: HashMap<String, AttrInfo> = HashMap::new();

        let attr_info1: AttrInfo = serde_json::from_str(mockdata_proof::ATTR_INFO_1).unwrap();
        let attr_info2: AttrInfo = serde_json::from_str(mockdata_proof::ATTR_INFO_2).unwrap();

        check_req_attrs.insert("attribute_0".to_string(), attr_info1);
        check_req_attrs.insert("attribute_1".to_string(), attr_info2);

        check_req_attrs
    }

    #[tokio::test]
    async fn test_proof_request_msg() {
        let _setup = SetupDefaults::init();

        let request = ProofRequestData::create(&mock_profile(), "Test")
            .await
            .unwrap()
            .set_not_revoked_interval(r#"{"from":1100000000, "to": 1600000000}"#.into())
            .unwrap()
            .set_requested_attributes_as_string(REQUESTED_ATTRS.into())
            .unwrap()
            .set_requested_predicates_as_string(REQUESTED_PREDICATES.into())
            .unwrap();

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

    #[tokio::test]
    async fn test_requested_attrs_constructed_correctly() {
        let _setup = SetupDefaults::init();

        let request = ProofRequestData::create(&mock_profile(), "")
            .await
            .unwrap()
            .set_requested_attributes_as_string(REQUESTED_ATTRS.into())
            .unwrap();
        assert_eq!(request.requested_attributes, _expected_req_attrs());
    }

    #[tokio::test]
    async fn test_requested_attrs_constructed_correctly_preformatted() {
        let _setup = SetupDefaults::init();

        let expected_req_attrs = _expected_req_attrs();
        let req_attrs_string = serde_json::to_string(&expected_req_attrs).unwrap();

        let request = ProofRequestData::create(&mock_profile(), "")
            .await
            .unwrap()
            .set_requested_attributes_as_string(req_attrs_string)
            .unwrap();
        assert_eq!(request.requested_attributes, expected_req_attrs);
    }

    #[tokio::test]
    async fn test_requested_predicates_constructed_correctly() {
        let _setup = SetupDefaults::init();

        let mut check_predicates: HashMap<String, PredicateInfo> = HashMap::new();
        let attr_info1: PredicateInfo = serde_json::from_str(
            r#"{
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
        }"#,
        )
        .unwrap();
        check_predicates.insert("predicate_0".to_string(), attr_info1);

        let request = ProofRequestData::create(&mock_profile(), "")
            .await
            .unwrap()
            .set_requested_predicates_as_string(REQUESTED_PREDICATES.into())
            .unwrap();
        assert_eq!(request.requested_predicates, check_predicates);
    }

    #[tokio::test]
    async fn test_requested_attrs_constructed_correctly_for_names() {
        let _setup = SetupDefaults::init();

        let attr_info = json!({
          "names": ["name", "age", "email"],
          "restrictions": [{"schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11"}]
        });
        let attr_info_2 = json!({
          "name":"name",
          "restrictions": [{"schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11" }]
        });

        let requested_attrs = json!([attr_info, attr_info_2]).to_string();

        let request = ProofRequestData::create(&mock_profile(), "")
            .await
            .unwrap()
            .set_requested_attributes_as_string(requested_attrs.into())
            .unwrap();

        let mut expected_req_attrs: HashMap<String, AttrInfo> = HashMap::new();
        expected_req_attrs.insert("attribute_0".to_string(), serde_json::from_value(attr_info).unwrap());
        expected_req_attrs.insert("attribute_1".to_string(), serde_json::from_value(attr_info_2).unwrap());
        assert_eq!(request.requested_attributes, expected_req_attrs);
    }

    #[tokio::test]
    async fn test_should_return_error_if_name_and_names_passed_together() {
        let _setup = SetupDefaults::init();

        let attr_info = json!({
          "name": "name",
          "names": ["name", "age", "email"],
          "restrictions": [{"schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11"}]
        });

        let requested_attrs = json!([attr_info]).to_string();

        let err = ProofRequestData::create(&mock_profile(), "")
            .await
            .unwrap()
            .set_requested_attributes_as_string(requested_attrs.into())
            .unwrap_err();

        assert_eq!(AriesVcxErrorKind::InvalidProofRequest, err.kind());
    }

    #[test]
    fn test_indy_proof_req_parses_correctly() {
        let _setup = SetupDefaults::init();

        let _proof_req: ProofRequestData = serde_json::from_str(utils::constants::INDY_PROOF_REQ_JSON).unwrap();
    }
}
