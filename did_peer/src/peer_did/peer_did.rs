use std::fmt::Display;

use crate::{error::DidPeerError, numalgos::numalgo2::generate_numalgo2};
use did_doc::schema::did_doc::DidDocument;
use did_doc_sov::extra_fields::ExtraFieldsSov;
use did_parser::Did;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{numalgo::Numalgo, transform::Transform};

// TODO: This regex does not cover peer:did:3
static PEER_DID_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^did:peer:(([01](z)([1-9a-km-zA-HJ-NP-Z]{5,200}))|(2((.[AEVID](z)([1-9a-km-zA-HJ-NP-Z]{5,200}))+(.(S)[0-9a-zA-Z=]*)?)))$").unwrap()
});

#[derive(Clone, Debug, PartialEq)]
pub struct PeerDid {
    did: Did,
    numalgo: Numalgo,
    transform: Option<Transform>,
}

impl PeerDid {
    pub fn parse<T>(did: T) -> Result<Self, DidPeerError>
    where
        Did: TryFrom<T>,
        <Did as TryFrom<T>>::Error: Into<DidPeerError>,
    {
        let did: Did = did.try_into().map_err(Into::into)?;
        let numalgo = Self::parse_numalgo(&did)?;
        let transform = match numalgo {
            Numalgo::InceptionKeyWithoutDoc | Numalgo::GenesisDoc => Some(Self::parse_transform(&did)?),
            _ => None,
        };
        Self::validate(&did)?;
        Ok(Self {
            did,
            numalgo,
            transform,
        })
    }
    pub fn did(&self) -> &Did {
        &self.did
    }

    pub fn numalgo(&self) -> &Numalgo {
        &self.numalgo
    }

    pub fn transform(&self) -> Option<&Transform> {
        self.transform.as_ref()
    }

    pub fn generate_numalgo2(did_document: DidDocument<ExtraFieldsSov>) -> Result<PeerDid, DidPeerError> {
        generate_numalgo2(did_document)
    }

    fn validate(did: &Did) -> Result<(), DidPeerError> {
        if !PEER_DID_REGEX.is_match(did.did()) {
            Err(DidPeerError::DidValidationError(format!("Invalid did: {}", did.did())))
        } else {
            Ok(())
        }
    }

    fn parse_numalgo(did: &Did) -> Result<Numalgo, DidPeerError> {
        did.id()
            .chars()
            .nth(0)
            .ok_or_else(|| DidPeerError::DidValidationError(format!("Invalid did: {}", did.did())))?
            .try_into()
    }

    fn parse_transform(did: &Did) -> Result<Transform, DidPeerError> {
        did.id()
            .chars()
            .nth(1)
            .ok_or_else(|| DidPeerError::DidValidationError(format!("Invalid did: {}", did.did())))?
            .try_into()
    }
}

impl Serialize for PeerDid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.did.did())
    }
}

impl<'de> Deserialize<'de> for PeerDid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let did = String::deserialize(deserializer)?;
        Self::parse(did).map_err(serde::de::Error::custom)
    }
}

impl Display for PeerDid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.did)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod parse {
        use super::*;

        macro_rules! generate_parse_test {
            ($test_name:ident, $input:expr, $error_pattern:pat) => {
                #[test]
                fn $test_name() {
                    let result = PeerDid::parse($input.to_string());
                    assert!(matches!(result, Err($error_pattern)));
                }
            };
        }

        generate_parse_test!(
            test_peer_did_parse_unsupported_transform_code,
            "did:peer:2\
            .Ea6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc\
            .Va6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V\
            .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXX0",
            DidPeerError::DidValidationError(_)
        );

        generate_parse_test!(
            test_peer_did_parse_malformed_base58_encoding_signing,
            "did:peer:2\
            .Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc\
            .Vz6MkqRYqQiSgvZQdnBytw86Qbs0ZWUkGv22od935YF4s8M7V\
            .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXX0",
            DidPeerError::DidValidationError(_)
        );

        generate_parse_test!(
            test_peer_did_parse_malformed_base58_encoding_encryption,
            "did:peer:2\
            .Ez6LSbysY2xFMRpG0hb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc\
            .Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V\
            .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXX0=",
            DidPeerError::DidParserError(_)
        );
    }
}
