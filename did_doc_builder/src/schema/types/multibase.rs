use std::{ops::Deref, str::FromStr};

use multibase::{decode, Base};
use serde::{Deserialize, Serialize};

use crate::error::DIDDocumentBuilderError;

#[derive(Clone, Debug, PartialEq)]
pub struct Multibase {
    base: Base,
    bytes: Vec<u8>,
}

impl Multibase {
    pub fn new(multibase: String) -> Result<Self, DIDDocumentBuilderError> {
        let (base, bytes) = decode(multibase).map_err(|err| {
            DIDDocumentBuilderError::InvalidInput(format!("Invalid multibase key: {}", err))
        })?;
        Ok(Self { base, bytes })
    }
}

impl Serialize for Multibase {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.base.encode(&self.bytes))
    }
}

impl<'de> Deserialize<'de> for Multibase {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

impl FromStr for Multibase {
    type Err = DIDDocumentBuilderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

impl ToString for Multibase {
    fn to_string(&self) -> String {
        self.base.encode(&self.bytes)
    }
}

impl Deref for Multibase {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multibase_new_valid() {
        let multibase =
            Multibase::new("zQmWvQxTqbG2Z9HPJgG57jjwR154cKhbtJenbyYTWkjgF3e".to_string());
        assert!(multibase.is_ok());
    }

    #[test]
    fn test_multibase_new_invalid() {
        let multibase = Multibase::new("invalidmultibasekey".to_string());
        assert!(multibase.is_err());
    }

    #[test]
    fn test_multibase_deserialize_valid() {
        let multibase: Multibase =
            serde_json::from_str("\"zQmWvQxTqbG2Z9HPJgG57jjwR154cKhbtJenbyYTWkjgF3e\"").unwrap();
        assert_eq!(
            multibase,
            Multibase {
                base: Base::Base58Btc,
                bytes: decode("zQmWvQxTqbG2Z9HPJgG57jjwR154cKhbtJenbyYTWkjgF3e")
                    .unwrap()
                    .1
            }
        )
    }

    #[test]
    fn test_multibase_deserialize_invalid() {
        let multibase: Result<Multibase, _> = serde_json::from_str("\"invalidmultibasekey\"");
        assert!(multibase.is_err());
    }

    #[test]
    fn test_multibase_from_str_valid() {
        let multibase = "zQmWvQxTqbG2Z9HPJgG57jjwR154cKhbtJenbyYTWkjgF3e".parse::<Multibase>();
        assert!(multibase.is_ok());
    }

    #[test]
    fn test_multibase_from_str_invalid() {
        let multibase = "invalidmultibasekey".parse::<Multibase>();
        assert!(multibase.is_err());
    }

    #[test]
    fn test_multibase_to_string() {
        let multibase =
            Multibase::new("zQmWvQxTqbG2Z9HPJgG57jjwR154cKhbtJenbyYTWkjgF3e".to_string()).unwrap();
        assert_eq!(
            multibase.to_string(),
            "QmWvQxTqbG2Z9HPJgG57jjwR154cKhbtJenbyYTWkjgF3e"
        );
    }
}
