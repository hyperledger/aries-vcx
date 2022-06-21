use core::convert::TryFrom;
use lazy_static::lazy_static;
use regex::{Regex, Captures};
use super::vdr::ledger_types::DidMethod;

lazy_static! {
    pub static ref REGEX: Regex = Regex::new("^(did|schema|creddef):(indy|cheqd)?(:?:)?([a-z0-9-]+):(.*)$").unwrap();
}

#[derive(Deserialize, Debug, Serialize, PartialEq, Clone)]
pub(crate) struct FullyQualifiedId {
    pub prefix: String,
    pub did_method: DidMethod,
    pub did_subspace: String,
    pub id: String,
}

impl FullyQualifiedId {
    pub fn namespace(&self) -> String {
        format!("{}:{}", self.did_method.to_string(), self.did_subspace)
    }
}

impl TryFrom<&str> for FullyQualifiedId {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match REGEX.captures(value) {
            None => {
                Err(format!("Unable to parse FullyQualifiedId from the string: {}", value))
            }
            Some(caps) => {
                trace!("FullyQualifiedId::TryFrom str: parts {:?}", caps);
                let did_method = match get_opt_string_value(&caps, 2).as_ref().map(String::as_str) {
                    None | Some("indy") => DidMethod::Indy,
                    Some("cheqd") => DidMethod::Cheqd,
                    Some(type_) => {
                        return Err(format!("ID contains unsupported ledger type: {}", type_));
                    }
                };

                Ok(FullyQualifiedId {
                    prefix: get_string_value(&caps, 1),
                    did_method,
                    did_subspace: get_string_value(&caps, 4),
                    id: get_string_value(&caps, 5),
                })
            }
        }
    }
}


fn get_string_value(caps: &Captures, index: usize) -> String {
    get_opt_string_value(caps, index).unwrap_or_default()
}

fn get_opt_string_value(caps: &Captures, index: usize) -> Option<String> {
    caps.get(index).map(|m| m.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn _prefix() -> &'static str {
        "did"
    }

    fn _namespace() -> &'static str {
        "sovrin"
    }

    fn _id() -> &'static str {
        "NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0"
    }

    fn _cheqd_id() -> &'static str {
        "NcYxiDXkpYi6ov5FcYDi1e"
    }

    fn _cheqd_namespace() -> &'static str {
        "cheqd-testnet"
    }

    #[rstest(schema_id,
    // schema id with network
    case("did:indy:sovrin:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0"),
    // schema id without network
    case("did:sovrin:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0"),
    )]
    fn parse_schema_fully_qulified_id(schema_id: &str) {
        let parsed_id: FullyQualifiedId = FullyQualifiedId::try_from(schema_id).unwrap();
        let expected = FullyQualifiedId {
            prefix: _prefix().to_string(),
            did_method: DidMethod::Indy,
            did_subspace: _namespace().to_string(),
            id: _id().to_string(),
        };
        assert_eq!(parsed_id, expected);
    }

    #[test]
    fn parse_schema_fully_qulified_id_old_fully_qualified_format() {
        let parsed_id: FullyQualifiedId = FullyQualifiedId::try_from("schema:sovrin:did:sovrin:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0").unwrap();
        let expected = FullyQualifiedId {
            prefix: "schema".to_string(),
            did_method: DidMethod::Indy,
            did_subspace: _namespace().to_string(),
            id: "did:sovrin:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0".to_string(),
        };
        assert_eq!(parsed_id, expected);
    }

    #[test]
    fn test_parse_invalid_fully_qulified_id() {
        FullyQualifiedId::try_from("did:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0").unwrap_err();
    }

    #[test]
    fn cheqd_parse_fully_qulified_did() {
        let parsed_id: FullyQualifiedId = FullyQualifiedId::try_from("did:cheqd:cheqd-testnet:NcYxiDXkpYi6ov5FcYDi1e").unwrap();
        let expected = FullyQualifiedId {
            prefix: _prefix().to_string(),
            did_method: DidMethod::Cheqd,
            did_subspace: _cheqd_namespace().to_string(),
            id: _cheqd_id().to_string(),
        };
        assert_eq!(parsed_id, expected);
    }

    #[test]
    fn cheqd_test_parse_invalid_fully_qulified_id() {
        FullyQualifiedId::try_from("some:another:did").unwrap_err();
    }
}
