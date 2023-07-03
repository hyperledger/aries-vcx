pub mod generic;

use core::fmt;
use std::{fmt::Display, marker::PhantomData};

use crate::error::DidPeerError;
use did_parser::Did;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

use super::numalgos::{
    numalgo3::Numalgo3,
    traits::{Numalgo, ToNumalgo3},
};

#[derive(Clone, Debug, PartialEq)]
pub struct PeerDid<N: Numalgo> {
    did: Did,
    numalgo: N,
}

impl<N: Numalgo> PeerDid<N> {
    pub fn parse<T>(did: T) -> Result<PeerDid<N>, DidPeerError>
    where
        Did: TryFrom<T>,
        <Did as TryFrom<T>>::Error: Into<DidPeerError>,
    {
        N::parse(did)
    }

    pub fn did(&self) -> &Did {
        &self.did
    }

    pub(crate) fn from_parts(did: Did, numalgo: N) -> PeerDid<N> {
        Self { did, numalgo }
    }
}

impl<N: ToNumalgo3> PeerDid<N> {
    pub fn to_numalgo3(&self) -> Result<PeerDid<Numalgo3>, DidPeerError> {
        N::to_numalgo3(self.did())
    }
}

impl<N: Numalgo> Serialize for PeerDid<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.did.did())
    }
}

impl<'de, N: Numalgo> Deserialize<'de> for PeerDid<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PeerDidVisitor<N: Numalgo>(PhantomData<N>);

        impl<'de, N: Numalgo> Visitor<'de> for PeerDidVisitor<N> {
            type Value = PeerDid<N>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing a DID")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match N::parse(value.to_string()) {
                    Ok(peer_did) => Ok(peer_did),
                    Err(err) => Err(E::custom(format!("Failed to parse numalgo: {err}"))),
                }
            }
        }

        deserializer.deserialize_str(PeerDidVisitor(PhantomData))
    }
}

impl<N: Numalgo> Display for PeerDid<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.did)
    }
}

#[cfg(test)]
mod tests {
    use crate::peer_did::numalgos::numalgo2::Numalgo2;

    use super::*;

    const VALID_PEER_DID_NUMALGO2: &str = "did:peer:2\
       .Ez6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH\
       .VzXwpBnMdCm1cLmKuzgESn29nqnonp1ioqrQMRHNsmjMyppzx8xB2pv7cw8q1PdDacSrdWE3dtB9f7Nxk886mdzNFoPtY\
       .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXSwiYSI6WyJkaWRjb21tL3YyIiwiZGlkY29tbS9haXAyO2Vudj1yZmM1ODciXX0";

    const VALID_PEER_DID_NUMALGO3: &str = "did:peer:3.d8da5079c166b183cf815ee27747f34e116977103d8b23c96dcba9a9d9429688";

    fn peer_did_numalgo2() -> PeerDid<Numalgo2> {
        PeerDid {
            did: VALID_PEER_DID_NUMALGO2.parse().unwrap(),
            numalgo: Numalgo2,
        }
    }

    fn peer_did_numalgo3() -> PeerDid<Numalgo3> {
        PeerDid {
            did: VALID_PEER_DID_NUMALGO3.parse().unwrap(),
            numalgo: Numalgo3,
        }
    }

    mod parse {
        use super::*;

        macro_rules! generate_negative_parse_test {
            ($test_name:ident, $input:expr, $error_pattern:pat) => {
                #[test]
                fn $test_name() {
                    let result = PeerDid::<Numalgo2>::parse($input.to_string());
                    assert!(matches!(result, Err($error_pattern)));
                }
            };
        }

        generate_negative_parse_test!(
            unsupported_transform_code,
            "did:peer:2\
            .Ea6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc\
            .Va6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V\
            .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXX0",
            DidPeerError::DidValidationError(_)
        );

        generate_negative_parse_test!(
            malformed_base58_encoding_signing,
            "did:peer:2\
            .Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc\
            .Vz6MkqRYqQiSgvZQdnBytw86Qbs0ZWUkGv22od935YF4s8M7V\
            .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXX0",
            DidPeerError::DidValidationError(_)
        );

        generate_negative_parse_test!(
            malformed_base58_encoding_encryption,
            "did:peer:2\
            .Ez6LSbysY2xFMRpG0hb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc\
            .Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V\
            .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXX0=",
            DidPeerError::DidParserError(_)
        );

        #[test]
        fn numalgo3() {
            let expected = PeerDid {
                did: VALID_PEER_DID_NUMALGO3.parse().unwrap(),
                numalgo: Numalgo3,
            };
            assert_eq!(
                expected,
                PeerDid::<Numalgo3>::parse(VALID_PEER_DID_NUMALGO3.to_string()).unwrap()
            );
        }
    }

    mod to_numalgo3 {
        use super::*;

        #[test]
        fn numalgo2() {
            assert_eq!(peer_did_numalgo3(), peer_did_numalgo2().to_numalgo3().unwrap());
        }

        #[test]
        fn numalgo3() {
            assert_eq!(peer_did_numalgo3(), peer_did_numalgo3().to_numalgo3().unwrap());
        }
    }

    mod serialize {
        use super::*;

        #[test]
        fn numalgo2() {
            assert_eq!(
                serde_json::to_string(&peer_did_numalgo2()).unwrap(),
                format!("\"{}\"", VALID_PEER_DID_NUMALGO2)
            );
        }

        #[test]
        fn numalgo3() {
            assert_eq!(
                serde_json::to_string(&peer_did_numalgo3()).unwrap(),
                format!("\"{VALID_PEER_DID_NUMALGO3}\"")
            );
        }
    }

    mod deserialize {
        use super::*;

        #[test]
        fn numalgo2() {
            let deserialized: PeerDid<Numalgo2> =
                serde_json::from_str(&format!("\"{}\"", VALID_PEER_DID_NUMALGO2)).unwrap();
            assert_eq!(peer_did_numalgo2(), deserialized);
        }

        #[test]
        fn numalgo3() {
            let deserialized: PeerDid<Numalgo3> =
                serde_json::from_str(&format!("\"{}\"", VALID_PEER_DID_NUMALGO3)).unwrap();
            assert_eq!(peer_did_numalgo3(), deserialized);
        }
    }
}
