use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ParsedDIDUrl;
use crate::{error::ParseError, utils::parse::parse_did_method_id, DIDRange};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParsedDID {
    did: String,
    method: DIDRange,
    id: DIDRange,
}

impl ParsedDID {
    pub fn parse(did: String) -> Result<Self, ParseError> {
        let (_, method, id) = parse_did_method_id(&did)?;

        if id.end > did.len() {
            return Err(ParseError::InvalidInput(format!("Invalid DID: {}", did)));
        }

        Ok(Self { did, method, id })
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

impl TryFrom<String> for ParsedDID {
    type Error = ParseError;

    fn try_from(did: String) -> Result<Self, Self::Error> {
        Self::parse(did)
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
