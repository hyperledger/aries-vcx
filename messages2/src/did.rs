use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize};
use std::{fmt::Display, str::FromStr};

use crate::error::DidError;

#[derive(Clone, Debug)]
pub struct Did {
    method_name: DidMethod,
    id: String,
}

impl Did {
    const PREFIX: &str = "did";
}

impl FromStr for Did {
    type Err = DidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts_iter = s.split(':');

        // Check prefix
        if Some(Self::PREFIX) != parts_iter.next() {
            return Err(DidError::InvalidPrefix);
        }

        // Get method
        let Some(method_str) = parts_iter.next() else {
            return Err(DidError::MissingComponent("method"));
        };

        // Get id
        let Some(id) = parts_iter.next() else {
            return Err(DidError::MissingComponent("id"));
        };

        let did = Self {
            method_name: method_str.into(),
            id: id.to_owned(),
        };

        Ok(did)
    }
}

impl Display for Did {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", Self::PREFIX, self.method_name, self.id)
    }
}

impl Serialize for Did {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Did {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let did_str = <&str>::deserialize(deserializer)?;
        did_str
            .parse()
            .map_err(|e| D::Error::custom(format!("DID {did_str}; {e}")))
    }
}

#[derive(Clone, Debug)]
pub enum DidMethod {
    Sov,
    Peer,
    Other(String),
}

impl DidMethod {
    const SOV: &str = "sov";
    const PEER: &str = "peer";
}

impl<'a> From<&'a str> for DidMethod {
    fn from(value: &'a str) -> Self {
        match value {
            Self::SOV => Self::Sov,
            Self::PEER => Self::Peer,
            s => Self::Other(s.to_owned()),
        }
    }
}

impl AsRef<str> for DidMethod {
    fn as_ref(&self) -> &str {
        match self {
            Self::Sov => Self::SOV,
            Self::Peer => Self::PEER,
            Self::Other(s) => s.as_ref(),
        }
    }
}

impl Display for DidMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}
