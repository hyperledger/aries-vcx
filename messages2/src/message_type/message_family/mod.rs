use derive_more::From;

use crate::error::{MsgTypeError, MsgTypeResult};

use self::{
    basic_message::BasicMessage, connection::Connection, cred_issuance::CredentialIssuance,
    discover_features::DiscoverFeatures, out_of_band::OutOfBand, present_proof::PresentProof,
    report_problem::ReportProblem, revocation::Revocation, routing::Routing, traits::ResolveMajorVersion,
    trust_ping::TrustPing,
};

pub mod basic_message;
pub mod connection;
pub mod cred_issuance;
pub mod discover_features;
pub mod out_of_band;
pub mod present_proof;
pub mod report_problem;
pub mod revocation;
pub mod routing;
pub mod traits;
pub mod trust_ping;

#[derive(Clone, Debug, From, PartialEq)]
pub enum MessageFamily {
    Routing(Routing),
    Connection(Connection),
    Revocation(Revocation),
    CredentialIssuance(CredentialIssuance),
    ReportProblem(ReportProblem),
    PresentProof(PresentProof),
    TrustPing(TrustPing),
    DiscoverFeatures(DiscoverFeatures),
    BasicMessage(BasicMessage),
    OutOfBand(OutOfBand),
}

impl MessageFamily {
    pub fn from_parts(family: &str, major: u8, minor: u8, kind: &str) -> MsgTypeResult<Self> {
        match family {
            Routing::FAMILY => Ok(Self::Routing(Routing::resolve_major_ver(major, minor, kind)?)),
            Connection::FAMILY => Ok(Self::Connection(Connection::resolve_major_ver(major, minor, kind)?)),
            Revocation::FAMILY => Ok(Self::Revocation(Revocation::resolve_major_ver(major, minor, kind)?)),
            CredentialIssuance::FAMILY => Ok(Self::CredentialIssuance(CredentialIssuance::resolve_major_ver(
                major, minor, kind,
            )?)),
            ReportProblem::FAMILY => Ok(Self::ReportProblem(ReportProblem::resolve_major_ver(
                major, minor, kind,
            )?)),
            PresentProof::FAMILY => Ok(Self::PresentProof(PresentProof::resolve_major_ver(major, minor, kind)?)),
            TrustPing::FAMILY => Ok(Self::TrustPing(TrustPing::resolve_major_ver(major, minor, kind)?)),
            DiscoverFeatures::FAMILY => Ok(Self::DiscoverFeatures(DiscoverFeatures::resolve_major_ver(
                major, minor, kind,
            )?)),
            BasicMessage::FAMILY => Ok(Self::BasicMessage(BasicMessage::resolve_major_ver(major, minor, kind)?)),
            OutOfBand::FAMILY => Ok(Self::OutOfBand(OutOfBand::resolve_major_ver(major, minor, kind)?)),
            _ => Err(MsgTypeError::unknown_family(family.to_owned())),
        }
    }

    /// Steps the provided iterator of parts and returns the string slice element.
    ///
    /// # Errors:
    ///
    /// Will return an error if the iterator returns [`None`].
    pub fn next_part<'a, I>(iter: &mut I, name: &'static str) -> MsgTypeResult<&'a str>
    where
        I: Iterator<Item = &'a str>,
    {
        iter.next().ok_or_else(|| MsgTypeError::not_found(name))
    }
}
