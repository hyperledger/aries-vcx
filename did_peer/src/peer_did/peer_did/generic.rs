use crate::{
    error::DidPeerError,
    peer_did::{
        numalgos::{numalgo2::Numalgo2, numalgo3::Numalgo3, NumalgoKind},
        parse::parse_numalgo,
        validate::validate,
    },
};
use did_parser::Did;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::PeerDid;

#[derive(Clone, Debug, PartialEq)]
pub enum GenericPeerDid {
    Numalgo2(PeerDid<Numalgo2>),
    Numalgo3(PeerDid<Numalgo3>),
}

impl GenericPeerDid {
    pub fn parse<T>(did: T) -> Result<GenericPeerDid, DidPeerError>
    where
        Did: TryFrom<T>,
        <Did as TryFrom<T>>::Error: Into<DidPeerError>,
    {
        let did: Did = did.try_into().map_err(Into::into)?;
        let numalgo = parse_numalgo(&did)?;
        validate(&did)?;
        let parsed = match numalgo {
            NumalgoKind::MultipleInceptionKeys(numalgo) => GenericPeerDid::Numalgo2(PeerDid { did, numalgo }),
            _ => GenericPeerDid::Numalgo3(PeerDid { did, numalgo: Numalgo3 }),
        };
        Ok(parsed)
    }

    pub fn numalgo(&self) -> NumalgoKind {
        match self {
            GenericPeerDid::Numalgo2(peer_did) => NumalgoKind::MultipleInceptionKeys(peer_did.numalgo),
            GenericPeerDid::Numalgo3(peer_did) => NumalgoKind::DidShortening(peer_did.numalgo),
        }
    }
}

impl Serialize for GenericPeerDid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self {
            GenericPeerDid::Numalgo2(peer_did) => serializer.serialize_str(peer_did.did().did()),
            GenericPeerDid::Numalgo3(peer_did) => serializer.serialize_str(peer_did.did().did()),
        }
    }
}

impl<'de> Deserialize<'de> for GenericPeerDid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let did = String::deserialize(deserializer)?;
        Self::parse(did).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_PEER_DID_NUMALGO2: &str = "did:peer:2\
       .Ez6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH\
       .VzXwpBnMdCm1cLmKuzgESn29nqnonp1ioqrQMRHNsmjMyppzx8xB2pv7cw8q1PdDacSrdWE3dtB9f7Nxk886mdzNFoPtY\
       .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXSwiYSI6WyJkaWRjb21tL3YyIiwiZGlkY29tbS9haXAyO2Vudj1yZmM1ODciXX0";

    const INVALID_PEER_DID_NUMALGO2: &str = "did:peer:2\
       .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXSwiYSI6WyJkaWRjb21tL3YyIiwiZGlkY29tbS9haXAyO2Vudj1yZmM1ODciXX1";

    const VALID_PEER_DID_NUMALGO3: &str = "did:peer:3.d8da5079c166b183cf815ee27747f34e116977103d8b23c96dcba9a9d9429688";

    const INVALID_PEER_DID_NUMALGO3: &str =
        "did:peer:3.d8da5079c166b183cfz15ee27747f34e116977103d8b23c96dcba9a9d9429689";

    fn generic_peer_did_numalgo2() -> GenericPeerDid {
        GenericPeerDid::Numalgo2(PeerDid {
            did: VALID_PEER_DID_NUMALGO2.parse().unwrap(),
            numalgo: Numalgo2,
        })
    }

    fn generic_peer_did_numalgo3() -> GenericPeerDid {
        GenericPeerDid::Numalgo3(PeerDid {
            did: VALID_PEER_DID_NUMALGO3.parse().unwrap(),
            numalgo: Numalgo3,
        })
    }

    mod serialize {
        use super::*;

        #[test]
        fn numalgo2() {
            let serialized = serde_json::to_string(&generic_peer_did_numalgo2()).unwrap();
            assert_eq!(serialized, format!("\"{}\"", VALID_PEER_DID_NUMALGO2));
        }

        #[test]
        fn numalgo3() {
            let serialized = serde_json::to_string(&generic_peer_did_numalgo3()).unwrap();
            assert_eq!(serialized, format!("\"{}\"", VALID_PEER_DID_NUMALGO3));
        }
    }

    mod deserialize {
        use super::*;

        #[test]
        fn numalgo2() {
            let deserialized: GenericPeerDid =
                serde_json::from_str(&format!("\"{}\"", VALID_PEER_DID_NUMALGO2)).unwrap();
            assert_eq!(deserialized, generic_peer_did_numalgo2());
        }

        #[test]
        fn numalgo2_invalid() {
            let deserialized: Result<GenericPeerDid, _> =
                serde_json::from_str(&format!("\"{}\"", INVALID_PEER_DID_NUMALGO2));
            assert!(deserialized.is_err());
        }

        #[test]
        fn numalgo3() {
            let deserialized: GenericPeerDid =
                serde_json::from_str(&format!("\"{}\"", VALID_PEER_DID_NUMALGO3)).unwrap();
            assert_eq!(deserialized, generic_peer_did_numalgo3());
        }

        #[test]
        fn numalgo3_invalid() {
            let deserialized: Result<GenericPeerDid, _> =
                serde_json::from_str(&format!("\"{}\"", INVALID_PEER_DID_NUMALGO3));
            assert!(deserialized.is_err());
        }
    }
}
