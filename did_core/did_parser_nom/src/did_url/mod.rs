mod parsing;

use std::{collections::HashMap, fmt::Display, str::FromStr};

use nom::combinator::all_consuming;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use self::parsing::{fragment_parser, parse_did_url};
use crate::{error::ParseError, Did, DidRange};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct DidUrl {
    did_url: String,
    did: Option<DidRange>,
    method: Option<DidRange>,
    namespace: Option<DidRange>,
    id: Option<DidRange>,
    path: Option<DidRange>,
    fragment: Option<DidRange>,
    queries: HashMap<DidRange, DidRange>,
}

impl DidUrl {
    // todo: can be &str
    pub fn parse(did_url: String) -> Result<Self, ParseError> {
        parse_did_url(did_url)
    }

    pub fn did_url(&self) -> &str {
        self.did_url.as_ref()
    }

    pub fn did(&self) -> Option<&str> {
        self.did.clone().map(|range| self.did_url[range].as_ref())
    }

    pub fn method(&self) -> Option<&str> {
        self.method
            .clone()
            .map(|range| self.did_url[range].as_ref())
    }

    pub fn namespace(&self) -> Option<&str> {
        self.namespace
            .clone()
            .map(|range| self.did_url[range].as_ref())
    }

    pub fn id(&self) -> Option<&str> {
        self.id.clone().map(|range| self.did_url[range].as_ref())
    }

    pub fn path(&self) -> Option<&str> {
        self.path.as_ref().map(|path| &self.did_url[path.clone()])
    }

    pub fn queries(&self) -> HashMap<String, String> {
        self.queries
            .iter()
            .map(|(k, v)| {
                (
                    query_percent_decode(&self.did_url[k.clone()]),
                    query_percent_decode(&self.did_url[v.clone()]),
                )
            })
            .collect()
    }

    pub fn fragment(&self) -> Option<&str> {
        self.fragment
            .as_ref()
            .map(|fragment| &self.did_url[fragment.clone()])
    }

    // TODO: Ideally we would have a builder instead of purpose-specific constructors
    pub fn from_fragment(fragment: String) -> Result<Self, ParseError> {
        if all_consuming(fragment_parser)(&fragment).is_err() {
            return Err(ParseError::InvalidInput("Invalid fragment"));
        }
        let len = fragment.len();
        Ok(Self {
            did_url: format!("#{}", fragment),
            did: None,
            method: None,
            namespace: None,
            id: None,
            path: None,
            fragment: Some(1..len + 1),
            queries: HashMap::new(),
        })
    }

    pub(crate) fn from_did_parts(
        did_url: String,
        did: DidRange,
        method: Option<DidRange>,
        id: DidRange,
    ) -> Self {
        Self {
            did_url,
            did: Some(did),
            method,
            namespace: None,
            id: Some(id),
            path: None,
            fragment: None,
            queries: HashMap::new(),
        }
    }
}

/// Decode percent-encoded URL query item (application/x-www-form-urlencoded encoded).
/// Primary difference from general percent encoding is encoding of ' ' as '+'
fn query_percent_decode(input: &str) -> String {
    percent_encoding::percent_decode_str(&input.replace('+', " "))
        .decode_utf8_lossy()
        .into_owned()
}

impl TryFrom<String> for DidUrl {
    type Error = ParseError;

    fn try_from(did_url: String) -> Result<Self, Self::Error> {
        Self::parse(did_url)
    }
}

impl FromStr for DidUrl {
    type Err = ParseError;

    fn from_str(did: &str) -> Result<Self, Self::Err> {
        Self::parse(did.to_string())
    }
}

impl Display for DidUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.did_url)
    }
}

impl Serialize for DidUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.did_url)
    }
}

impl<'de> Deserialize<'de> for DidUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let did_url = String::deserialize(deserializer)?;
        DidUrl::parse(did_url).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<&DidUrl> for Did {
    type Error = ParseError;

    fn try_from(did_url: &DidUrl) -> Result<Self, Self::Error> {
        let err = || ParseError::InvalidInput("Unable to construct a DID from relative DID URL");
        Ok(Did::from_parts(
            did_url.did().ok_or_else(err)?.to_owned(),
            did_url.method.to_owned(),
            did_url.namespace.to_owned(),
            did_url.id.to_owned().ok_or_else(err)?,
        ))
    }
}
