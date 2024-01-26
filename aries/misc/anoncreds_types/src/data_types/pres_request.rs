use anoncreds_clsignatures::PredicateType;
use std::collections::HashMap;
use std::fmt;

use serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use super::credential::Credential;
use super::nonce::Nonce;
use crate::error::ValidationError;
use crate::invalid;
use crate::utils::{
    query::Query,
    validation::{self, Validatable},
};

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PresentationRequestPayload {
    pub nonce: Nonce,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub requested_attributes: HashMap<String, AttributeInfo>,
    #[serde(default)]
    pub requested_predicates: HashMap<String, PredicateInfo>,
    pub non_revoked: Option<NonRevokedInterval>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PresentationRequest {
    PresentationRequestV1(PresentationRequestPayload),
    PresentationRequestV2(PresentationRequestPayload),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PresentationRequestVersion {
    V1,
    V2,
}

impl PresentationRequest {
    #[must_use]
    pub const fn value(&self) -> &PresentationRequestPayload {
        match self {
            Self::PresentationRequestV1(req) | Self::PresentationRequestV2(req) => req,
        }
    }

    #[must_use]
    pub const fn version(&self) -> PresentationRequestVersion {
        match self {
            Self::PresentationRequestV1(_) => PresentationRequestVersion::V1,
            Self::PresentationRequestV2(_) => PresentationRequestVersion::V2,
        }
    }
}

impl<'de> Deserialize<'de> for PresentationRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            ver: Option<String>,
        }

        let v = Value::deserialize(deserializer)?;

        let helper = Helper::deserialize(&v).map_err(de::Error::custom)?;

        let req = if let Some(version) = helper.ver {
            match version.as_ref() {
                "1.0" => {
                    let request =
                        PresentationRequestPayload::deserialize(v).map_err(de::Error::custom)?;
                    Self::PresentationRequestV1(request)
                }
                "2.0" => {
                    let request =
                        PresentationRequestPayload::deserialize(v).map_err(de::Error::custom)?;
                    Self::PresentationRequestV2(request)
                }
                _ => return Err(de::Error::unknown_variant(&version, &["2.0"])),
            }
        } else {
            let request = PresentationRequestPayload::deserialize(v).map_err(de::Error::custom)?;
            Self::PresentationRequestV1(request)
        };
        Ok(req)
    }
}

impl Serialize for PresentationRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match self {
            Self::PresentationRequestV1(v1) => {
                let mut value = ::serde_json::to_value(v1).map_err(ser::Error::custom)?;
                value
                    .as_object_mut()
                    .unwrap()
                    .insert("ver".into(), Value::from("1.0"));
                value
            }
            Self::PresentationRequestV2(v2) => {
                let mut value = ::serde_json::to_value(v2).map_err(ser::Error::custom)?;
                value
                    .as_object_mut()
                    .unwrap()
                    .insert("ver".into(), Value::from("2.0"));
                value
            }
        };

        value.serialize(serializer)
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct NonRevokedInterval {
    pub from: Option<u64>,
    pub to: Option<u64>,
}

impl NonRevokedInterval {
    #[must_use]
    pub const fn new(from: Option<u64>, to: Option<u64>) -> Self {
        Self { from, to }
    }
    // Returns the most stringent interval,
    // i.e. the latest from and the earliest to
    pub fn compare_and_set(&mut self, to_compare: &Self) {
        // Update if
        // - the new `from` value is later, smaller interval
        // - the new `from` value is Some if previouly was None
        match (self.from, to_compare.from) {
            (Some(old_from), Some(new_from)) => {
                if old_from.lt(&new_from) {
                    self.from = to_compare.from;
                }
            }
            (None, Some(_)) => self.from = to_compare.from,
            _ => (),
        }
        // Update if
        // - the new `to` value is earlier, smaller interval
        // - the new `to` value is Some if previouly was None
        match (self.to, to_compare.to) {
            (Some(old_to), Some(new_to)) => {
                if new_to.lt(&old_to) {
                    self.to = to_compare.to;
                }
            }
            (None, Some(_)) => self.to = to_compare.to,
            _ => (),
        }
    }

    pub fn update_with_override(&mut self, override_map: &HashMap<u64, u64>) {
        self.from.map(|from| {
            override_map
                .get(&from)
                .map(|&override_timestamp| self.from = Some(override_timestamp))
        });
    }

    pub fn is_valid(&self, timestamp: u64) -> Result<(), ValidationError> {
        if timestamp.lt(&self.from.unwrap_or(0)) || timestamp.gt(&self.to.unwrap_or(u64::MAX)) {
            Err(invalid!("Invalid timestamp"))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct AttributeInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<Vec<String>>,
    pub restrictions: Option<Query>,
    pub non_revoked: Option<NonRevokedInterval>,
}

pub type PredicateValue = i32;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct PredicateInfo {
    pub name: String,
    pub p_type: PredicateTypes,
    pub p_value: PredicateValue,
    pub restrictions: Option<Query>,
    pub non_revoked: Option<NonRevokedInterval>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum PredicateTypes {
    #[serde(rename = ">=")]
    GE,
    #[serde(rename = "<=")]
    LE,
    #[serde(rename = ">")]
    GT,
    #[serde(rename = "<")]
    LT,
}

impl fmt::Display for PredicateTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::GE => write!(f, "GE"),
            Self::GT => write!(f, "GT"),
            Self::LE => write!(f, "LE"),
            Self::LT => write!(f, "LT"),
        }
    }
}

impl From<PredicateTypes> for PredicateType {
    fn from(value: PredicateTypes) -> Self {
        match value {
            PredicateTypes::GE => PredicateType::GE,
            PredicateTypes::GT => PredicateType::GT,
            PredicateTypes::LE => PredicateType::LE,
            PredicateTypes::LT => PredicateType::LT,
        }
    }
}

// #[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
// pub struct RequestedAttributeInfo {
//     pub attr_referent: String,
//     pub attr_info: AttributeInfo,
//     pub revealed: bool,
// }
//
// #[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
// pub struct RequestedPredicateInfo {
//     pub predicate_referent: String,
//     pub predicate_info: PredicateInfo,
// }

impl Validatable for PresentationRequest {
    fn validate(&self) -> Result<(), ValidationError> {
        let value = self.value();
        let version = self.version();

        if value.requested_attributes.is_empty() && value.requested_predicates.is_empty() {
            return Err(invalid!("Presentation request validation failed: both `requested_attributes` and `requested_predicates` are empty"));
        }

        for requested_attribute in value.requested_attributes.values() {
            let has_name = !requested_attribute
                .name
                .as_ref()
                .map_or(true, String::is_empty);
            let has_names = !requested_attribute
                .names
                .as_ref()
                .map_or(true, Vec::is_empty);
            if !has_name && !has_names {
                return Err(invalid!(
                    "Presentation request validation failed: there is empty requested attribute: {:?}",
                    requested_attribute
                ));
            }

            if has_name && has_names {
                return Err(invalid!("Presentation request validation failed: there is a requested attribute with both name and names: {:?}", requested_attribute));
            }

            if let Some(ref restrictions) = requested_attribute.restrictions {
                _process_operator(restrictions, &version)?;
            }
        }

        for requested_predicate in value.requested_predicates.values() {
            if requested_predicate.name.is_empty() {
                return Err(invalid!(
                    "Presentation request validation failed: there is empty requested attribute: {:?}",
                    requested_predicate
                ));
            }
            if let Some(ref restrictions) = requested_predicate.restrictions {
                _process_operator(restrictions, &version)?;
            }
        }

        Ok(())
    }
}

fn _process_operator(
    restriction_op: &Query,
    version: &PresentationRequestVersion,
) -> Result<(), ValidationError> {
    match restriction_op {
        Query::Eq(ref tag_name, ref tag_value)
        | Query::Neq(ref tag_name, ref tag_value)
        | Query::Gt(ref tag_name, ref tag_value)
        | Query::Gte(ref tag_name, ref tag_value)
        | Query::Lt(ref tag_name, ref tag_value)
        | Query::Lte(ref tag_name, ref tag_value)
        | Query::Like(ref tag_name, ref tag_value) => {
            _check_restriction(tag_name, tag_value, version)
        }
        Query::In(ref tag_name, ref tag_values) => {
            tag_values
                .iter()
                .map(|tag_value| _check_restriction(tag_name, tag_value, version))
                .collect::<Result<Vec<()>, ValidationError>>()?;
            Ok(())
        }
        Query::Exist(ref tag_names) => {
            tag_names
                .iter()
                .map(|tag_name| _check_restriction(tag_name, "", version))
                .collect::<Result<Vec<()>, ValidationError>>()?;
            Ok(())
        }
        Query::And(ref operators) | Query::Or(ref operators) => {
            operators
                .iter()
                .map(|operator| _process_operator(operator, version))
                .collect::<Result<Vec<()>, ValidationError>>()?;
            Ok(())
        }
        Query::Not(ref operator) => _process_operator(operator, version),
    }
}

fn _check_restriction(
    tag_name: &str,
    tag_value: &str,
    version: &PresentationRequestVersion,
) -> Result<(), ValidationError> {
    if *version == PresentationRequestVersion::V1
        && Credential::QUALIFIABLE_TAGS.contains(&tag_name)
        && validation::is_uri_identifier(tag_value)
    {
        return Err(invalid!("Presentation request validation failed: fully qualified identifiers can not be used for presentation request of the first version. \
                    Please, set \"ver\":\"2.0\" to use fully qualified identifiers."));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod invalid_nonce {
        use super::*;

        #[test]
        fn presentation_request_valid_nonce() {
            let req_json = json!({
                "nonce": "123456",
                "name": "name",
                "version": "2.0",
                "requested_attributes": {},
                "requested_predicates": {},
            })
            .to_string();

            let req: PresentationRequest = serde_json::from_str(&req_json).unwrap();
            let payload = match req {
                PresentationRequest::PresentationRequestV1(p) => p,
                PresentationRequest::PresentationRequestV2(p) => p,
            };

            assert_eq!(&*payload.nonce, "123456");
        }

        #[test]
        fn presentation_request_invalid_nonce() {
            let req_json = json!({
                "nonce": "123abc",
                "name": "name",
                "version": "2.0",
                "requested_attributes": {},
                "requested_predicates": {},
            })
            .to_string();

            serde_json::from_str::<PresentationRequest>(&req_json).unwrap_err();
        }
    }

    #[test]
    fn override_works() {
        let mut interval = NonRevokedInterval::default();
        let override_map = HashMap::from([(10u64, 5u64)]);

        interval.from = Some(10);
        interval.update_with_override(&override_map);
        assert_eq!(interval.from.unwrap(), 5u64);
    }

    #[test]
    fn compare_and_set_works() {
        let mut int = NonRevokedInterval::default();
        let wide_int = NonRevokedInterval::new(Some(1), Some(100));
        let mid_int = NonRevokedInterval::new(Some(5), Some(80));
        let narrow_int = NonRevokedInterval::new(Some(10), Some(50));

        assert_eq!(int.from, None);
        assert_eq!(int.to, None);

        // From None to Some
        int.compare_and_set(&wide_int);
        assert_eq!(int.from, wide_int.from);
        assert_eq!(int.to, wide_int.to);

        // Update when more narrow
        int.compare_and_set(&mid_int);
        assert_eq!(int.from, mid_int.from);
        assert_eq!(int.to, mid_int.to);

        // Do Not Update when wider
        int.compare_and_set(&wide_int);
        assert_eq!(int.from, mid_int.from);
        assert_eq!(int.to, mid_int.to);

        int.compare_and_set(&narrow_int);
        assert_eq!(int.from, narrow_int.from);
        assert_eq!(int.to, narrow_int.to);
    }
}
