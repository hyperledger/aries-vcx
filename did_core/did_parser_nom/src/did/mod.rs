pub(crate) mod parsing;

use std::{
    convert::TryFrom,
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use self::parsing::parse_did;
use crate::{error::ParseError, DidRange, DidUrl};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Did {
    did: String,
    method: Option<DidRange>,
    namespace: Option<DidRange>,
    id: DidRange,
}

// TODO: Add a builder / constructor so that we don't have to create strings and parse them?
impl Did {
    pub fn parse(did: String) -> Result<Self, ParseError> {
        parse_did(did)
    }

    pub fn did(&self) -> &str {
        self.did.as_ref()
    }

    pub fn method(&self) -> Option<&str> {
        self.method.as_ref().map(|range| &self.did[range.clone()])
    }

    pub fn namespace(&self) -> Option<&str> {
        self.namespace
            .as_ref()
            .map(|range| &self.did[range.clone()])
    }

    pub fn id(&self) -> &str {
        self.did[self.id.clone()].as_ref()
    }

    pub(crate) fn from_parts(
        did: String,
        method: Option<DidRange>,
        namespace: Option<DidRange>,
        id: DidRange,
    ) -> Self {
        Self {
            did,
            method,
            namespace,
            id,
        }
    }
}

impl TryFrom<String> for Did {
    type Error = ParseError;

    fn try_from(did: String) -> Result<Self, Self::Error> {
        Self::parse(did)
    }
}

impl FromStr for Did {
    type Err = ParseError;

    fn from_str(did: &str) -> Result<Self, Self::Err> {
        Self::parse(did.to_string())
    }
}

impl Display for Did {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.did)
    }
}

impl std::fmt::Debug for Did {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Did")
            .field("did", &self.did)
            .field("method", &self.method())
            .field("id", &self.id())
            .field("namespace", &self.namespace())
            .finish()
    }
}

impl Serialize for Did {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.did())
    }
}

impl<'de> Deserialize<'de> for Did {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let did = String::deserialize(deserializer)?;
        Self::parse(did).map_err(serde::de::Error::custom)
    }
}

impl From<Did> for DidUrl {
    fn from(did: Did) -> Self {
        Self::from_did_parts(did.did().to_string(), 0..did.did.len(), did.method, did.id)
    }
}
