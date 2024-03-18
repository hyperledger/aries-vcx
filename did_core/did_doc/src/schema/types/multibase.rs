use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub struct MultibaseWrapperError {
    reason: &'static str,
    #[source]
    source: Box<dyn Error + Sync + Send>,
}

impl Display for MultibaseWrapperError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MultibaseWrapperError, reason: {}, source: {}",
            self.reason, self.source
        )
    }
}

// https://datatracker.ietf.org/doc/html/draft-multiformats-multibase-07
#[derive(Clone, Debug, PartialEq)]
pub struct Multibase {
    base: multibase::Base,
    bytes: Vec<u8>,
}

impl Multibase {
    pub fn new(multibase: String) -> Result<Self, MultibaseWrapperError> {
        let (base, bytes) = multibase::decode(multibase).map_err(|err| MultibaseWrapperError {
            reason: "Decoding multibase value failed",
            source: Box::new(err),
        })?;
        Ok(Self { base, bytes })
    }

    pub fn base_to_multibase(base: multibase::Base, encoded: &str) -> Self {
        let multibase_encoded = format!("{}{}", base.code(), encoded);
        Self {
            base,
            bytes: multibase_encoded.as_bytes().to_vec(),
        }
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
    type Err = MultibaseWrapperError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

impl Display for Multibase {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.base.encode(&self.bytes))
    }
}

impl AsRef<[u8]> for Multibase {
    fn as_ref(&self) -> &[u8] {
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
        let err = multibase.expect_err("Error was expected.");
        assert!(err
            .source()
            .expect("Error was expected to has source set up.")
            .is::<multibase::Error>());
        assert!(err
            .to_string()
            .contains("Decoding multibase value failed, source: "));
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

    #[test]
    fn test_base_to_multibase() {
        let multibase = Multibase::base_to_multibase(
            Base::Base58Btc,
            "QmWvQxTqbG2Z9HPJgG57jjwR154cKhbtJenbyYTWkjgF3e",
        );
        assert_eq!(
            multibase,
            Multibase {
                base: Base::Base58Btc,
                bytes: "zQmWvQxTqbG2Z9HPJgG57jjwR154cKhbtJenbyYTWkjgF3e"
                    .as_bytes()
                    .to_vec()
            }
        )
    }
}
