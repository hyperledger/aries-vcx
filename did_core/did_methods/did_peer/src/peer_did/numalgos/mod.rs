pub mod kind;
pub mod numalgo0;
pub mod numalgo1;
pub mod numalgo2;
pub mod numalgo3;

use did_doc::schema::did_doc::DidDocument;
use did_parser::Did;

use crate::{
    error::DidPeerError,
    peer_did::{parse::parse_numalgo, validate::validate, PeerDid},
    resolver::options::PublicKeyEncoding,
};

pub trait Numalgo: Sized + Default {
    const NUMALGO_CHAR: char;

    fn parse<T>(did: T) -> Result<PeerDid<Self>, DidPeerError>
    where
        Did: TryFrom<T>,
        <Did as TryFrom<T>>::Error: Into<DidPeerError>,
    {
        let did: Did = did.try_into().map_err(Into::into)?;
        let numalgo_char = parse_numalgo(&did)?.to_char();
        if numalgo_char != Self::NUMALGO_CHAR {
            return Err(DidPeerError::InvalidNumalgoCharacter(numalgo_char));
        }
        validate(&did)?;
        Ok(PeerDid::from_parts(did, Self::default()))
    }
}

pub trait ResolvableNumalgo: Numalgo {
    fn resolve(
        &self,
        did: &Did,
        public_key_encoding: PublicKeyEncoding,
    ) -> Result<DidDocument, DidPeerError>;
}
