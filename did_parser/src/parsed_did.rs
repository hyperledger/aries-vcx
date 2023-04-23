use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ParsedDIDUrl;
use crate::{error::ParseError, is_valid_did, utils::parse::parse_did_method_id, DIDRange};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParsedDID {
    did: String,
    method: DIDRange,
    id: DIDRange,
}

impl ParsedDID {
    pub fn parse(did: String) -> Result<Self, ParseError> {
        if is_valid_did(&did) {
            let (_, method, id) = parse_did_method_id(&did)?;

            Ok(Self { did, method, id })
        } else {
            Err(ParseError::InvalidInput(format!("Invalid DID: {}", did)))
        }
    }

    pub fn did(&self) -> &str {
        self.did.as_ref()
    }

    pub fn method(&self) -> &str {
        self.did[self.method.clone()].as_ref()
    }

    pub fn id(&self) -> &str {
        self.did[self.id.clone()].as_ref()
    }
}

impl FromStr for ParsedDID {
    type Err = ParseError;

    fn from_str(did: &str) -> Result<Self, Self::Err> {
        Self::parse(did.to_string())
    }
}

impl Display for ParsedDID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.did)
    }
}

impl Serialize for ParsedDID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.did())
    }
}

impl<'de> Deserialize<'de> for ParsedDID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let did = String::deserialize(deserializer)?;
        Self::parse(did).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<&ParsedDIDUrl> for ParsedDID {
    type Error = ParseError;

    fn try_from(did_url: &ParsedDIDUrl) -> Result<Self, Self::Error> {
        Self::parse(
            did_url
                .did()
                .ok_or(Self::Error::InvalidInput(
                    "No DID provided in the DID URL".to_string(),
                ))?
                .to_string(),
        )
    }
}
