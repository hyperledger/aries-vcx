use std::{collections::HashMap, fmt::Display, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    error::ParseError,
    utils::parse::{parse_did_method_id, parse_key_value, parse_path},
    Did, DidRange,
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct DidUrl {
    did_url: String,
    did: Option<DidRange>,
    method: Option<DidRange>,
    id: Option<DidRange>,
    path: Option<DidRange>,
    fragment: Option<DidRange>,
    queries: HashMap<DidRange, DidRange>,
    params: HashMap<DidRange, DidRange>,
}

impl DidUrl {
    pub fn parse(did_url: String) -> Result<Self, ParseError> {
        let (did, method, id) = if did_url.starts_with('#')
            || did_url.starts_with('/')
            || did_url.starts_with('?')
            || did_url.starts_with(';')
        {
            (None, None, None)
        } else {
            let (did, method, id) = parse_did_method_id(&did_url)?;
            (Some(did), method, Some(id))
        };

        let mut path = None;
        let mut fragment = None;
        let mut queries = HashMap::new();
        let mut params = HashMap::new();

        let mut current_pos = id.clone().map_or(0, |id| id.end);

        while current_pos < did_url.len() {
            match did_url.chars().nth(current_pos) {
                Some(';') => {
                    let (key_start, value_start, next_pos) =
                        parse_key_value(&did_url, current_pos, did_url.len())?;
                    params.insert(key_start..value_start - 1, value_start..next_pos);
                    current_pos = next_pos;
                }
                Some('/') => {
                    if path.is_none() {
                        path = Some(parse_path(&did_url, current_pos)?);
                        current_pos = path.as_ref().unwrap().end;
                    } else {
                        current_pos += 1;
                    }
                }
                Some('?') | Some('&') => {
                    let (key_start, value_start, next_pos) =
                        parse_key_value(&did_url, current_pos, did_url.len())?;
                    queries.insert(key_start..value_start - 1, value_start..next_pos);
                    current_pos = next_pos;
                }
                Some('#') => {
                    if fragment.is_none() {
                        fragment = Some(current_pos + 1..did_url.len());
                    }
                    current_pos += 1;
                }
                _ => break,
            };
        }

        if did.is_none()
            && method.is_none()
            && id.is_none()
            && path.is_none()
            && fragment.is_none()
            && queries.is_empty()
            && params.is_empty()
        {
            return Err(ParseError::InvalidInput("Empty DID URL"));
        }

        Ok(DidUrl {
            did_url,
            did,
            method,
            id,
            path,
            queries,
            fragment,
            params,
        })
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
                    self.did_url[k.clone()].to_string(),
                    self.did_url[v.clone()].to_string(),
                )
            })
            .collect()
    }

    pub fn fragment(&self) -> Option<&str> {
        self.fragment
            .as_ref()
            .map(|fragment| &self.did_url[fragment.clone()])
    }

    pub fn params(&self) -> HashMap<String, String> {
        self.params
            .iter()
            .map(|(k, v)| {
                (
                    self.did_url[k.clone()].to_string(),
                    self.did_url[v.clone()].to_string(),
                )
            })
            .collect()
    }

    // TODO: Ideally we would have a builder instead of purpose-specific constructors
    pub fn from_fragment(fragment: String) -> Result<Self, ParseError> {
        // TODO: Better validation
        if fragment.contains("#") {
            return Err(ParseError::InvalidInput(
                "Fragment cannot contain '#' character",
            ));
        }
        if fragment.is_empty() {
            return Err(ParseError::InvalidInput("Empty fragment"));
        }
        let len = fragment.len();
        Ok(Self {
            did_url: format!("#{}", fragment),
            did: None,
            method: None,
            id: None,
            path: None,
            fragment: Some(1..len + 1),
            queries: HashMap::new(),
            params: HashMap::new(),
        })
    }

    pub(crate) fn from_did_parts(
        did_url: String,
        did: DidRange,
        method: DidRange,
        id: DidRange,
    ) -> Self {
        Self {
            did_url,
            did: Some(did),
            method: Some(method),
            id: Some(id),
            path: None,
            fragment: None,
            queries: HashMap::new(),
            params: HashMap::new(),
        }
    }
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
        if let (Some(did), Some(method), Some(id)) = (did_url.did(), &did_url.method, &did_url.id) {
            Ok(Did::from_parts(
                did.to_owned(),
                method.to_owned(),
                id.to_owned(),
            ))
        } else {
            Err(ParseError::InvalidInput(
                "Unable to construct a DID from relative DID URL",
            ))
        }
    }
}
