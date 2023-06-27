use did_doc::schema::did_doc::DidDocument;
use did_doc_sov::extra_fields::ExtraFieldsSov;
use did_parser::Did;

use crate::{
    error::DidPeerError,
    peer_did::{peer_did::PeerDid, validate::validate},
    peer_did_resolver::options::PublicKeyEncoding,
};

use super::numalgo3::Numalgo3;

pub trait Numalgo: Sized + Default {
    const NUMALGO_CHAR: char;

    fn parse<T>(did: T) -> Result<PeerDid<Self>, DidPeerError>
    where
        Did: TryFrom<T>,
        <Did as TryFrom<T>>::Error: Into<DidPeerError>,
    {
        let did: Did = did.try_into().map_err(Into::into)?;

        let numalgo_char = did.id().chars().nth(0).ok_or_else(|| {
            DidPeerError::DidValidationError(format!(
                "Invalid did: unable to read numalgo character in did {}",
                did.did()
            ))
        })?;

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
    ) -> Result<DidDocument<ExtraFieldsSov>, DidPeerError>;
}

pub trait ToNumalgo3: Numalgo {
    fn to_numalgo3(&self, did: &Did) -> Result<PeerDid<Numalgo3>, DidPeerError>;
}
