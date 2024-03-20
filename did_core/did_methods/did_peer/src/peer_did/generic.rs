use std::fmt::Display;

use did_parser_nom::Did;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::PeerDid;
use crate::{
    error::DidPeerError,
    peer_did::{
        numalgos::{kind::NumalgoKind, numalgo2::Numalgo2, numalgo3::Numalgo3, numalgo4::Numalgo4},
        parse::parse_numalgo,
    },
};

#[derive(Clone, Debug, PartialEq)]
pub enum AnyPeerDid {
    Numalgo2(PeerDid<Numalgo2>),
    Numalgo3(PeerDid<Numalgo3>),
    Numalgo4(PeerDid<Numalgo4>),
}

impl AnyPeerDid {
    pub fn parse<T>(did: T) -> Result<AnyPeerDid, DidPeerError>
    where
        T: Display,
        Did: TryFrom<T>,
        <Did as TryFrom<T>>::Error: Into<DidPeerError>,
    {
        log::info!("AnyPeerDid >> parsing input {} as peer:did", did);
        let did: Did = did.try_into().map_err(Into::into)?;
        log::info!("AnyPeerDid >> parsed did {}", did);
        let numalgo = parse_numalgo(&did)?;
        log::info!("AnyPeerDid >> parsed numalgo {}", numalgo.to_char());
        let parsed = match numalgo {
            NumalgoKind::MultipleInceptionKeys(numalgo2) => AnyPeerDid::Numalgo2(PeerDid {
                did,
                numalgo: numalgo2,
            }),
            NumalgoKind::DidShortening(numalgo3) => AnyPeerDid::Numalgo3(PeerDid {
                did,
                numalgo: numalgo3,
            }),
            NumalgoKind::DidPeer4(numalgo4) => AnyPeerDid::Numalgo4(PeerDid {
                did,
                numalgo: numalgo4,
            }),
            o => unimplemented!("Parsing numalgo {} is not supported", o.to_char()),
        };
        Ok(parsed)
    }

    pub fn numalgo(&self) -> NumalgoKind {
        match self {
            AnyPeerDid::Numalgo2(peer_did) => NumalgoKind::MultipleInceptionKeys(peer_did.numalgo),
            AnyPeerDid::Numalgo3(peer_did) => NumalgoKind::DidShortening(peer_did.numalgo),
            AnyPeerDid::Numalgo4(peer_did) => NumalgoKind::DidPeer4(peer_did.numalgo),
        }
    }
}

impl Serialize for AnyPeerDid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self {
            AnyPeerDid::Numalgo2(peer_did) => serializer.serialize_str(peer_did.did().did()),
            AnyPeerDid::Numalgo3(peer_did) => serializer.serialize_str(peer_did.did().did()),
            AnyPeerDid::Numalgo4(peer_did) => serializer.serialize_str(peer_did.did().did()),
        }
    }
}

impl<'de> Deserialize<'de> for AnyPeerDid {
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
       .SeyJpZCI6IiNzZXJ2aWNlLTAiLCJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXSwiYSI6WyJkaWRjb21tL3YyIiwiZGlkY29tbS9haXAyO2Vudj1yZmM1ODciXX0";

    const VALID_PEER_DID_NUMALGO3: &str =
        "did:peer:3.d8da5079c166b183cf815ee27747f34e116977103d8b23c96dcba9a9d9429688";

    fn generic_peer_did_numalgo2() -> AnyPeerDid {
        AnyPeerDid::Numalgo2(PeerDid {
            did: VALID_PEER_DID_NUMALGO2.parse().unwrap(),
            numalgo: Numalgo2,
        })
    }

    fn generic_peer_did_numalgo3() -> AnyPeerDid {
        AnyPeerDid::Numalgo3(PeerDid {
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
            let deserialized: AnyPeerDid =
                serde_json::from_str(&format!("\"{}\"", VALID_PEER_DID_NUMALGO2)).unwrap();
            assert_eq!(deserialized, generic_peer_did_numalgo2());
        }

        #[test]
        fn numalgo3() {
            let deserialized: AnyPeerDid =
                serde_json::from_str(&format!("\"{}\"", VALID_PEER_DID_NUMALGO3)).unwrap();
            assert_eq!(deserialized, generic_peer_did_numalgo3());
        }
    }
}
