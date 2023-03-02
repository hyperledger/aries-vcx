use derive_more::{From, TryInto};

use crate::error::{MsgTypeError, MsgTypeResult};

use self::{
    basic_message::BasicMessage, connection::Connection, cred_issuance::CredentialIssuance,
    discover_features::DiscoverFeatures, notification::Notification, out_of_band::OutOfBand,
    present_proof::PresentProof, report_problem::ReportProblem, revocation::Revocation, routing::Routing,
    traits::ResolveMajorVersion, trust_ping::TrustPing,
};

use super::MessageType;

pub mod basic_message;
pub mod connection;
pub mod cred_issuance;
pub mod discover_features;
pub mod notification;
pub mod out_of_band;
pub mod present_proof;
pub mod report_problem;
pub mod revocation;
pub mod routing;
pub mod traits;
pub mod trust_ping;

#[derive(Clone, Debug, From, TryInto, PartialEq)]
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
    Notification(Notification),
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
            Notification::FAMILY => Ok(Self::Notification(Notification::resolve_major_ver(major, minor, kind)?)),
            _ => Err(MsgTypeError::unknown_family(family.to_owned())),
        }
    }

    pub fn as_parts(&self) -> (&str, u8, u8, &str) {
        match &self {
            Self::Routing(v) => v.as_msg_type_parts(),
            Self::Connection(v) => v.as_msg_type_parts(),
            Self::Revocation(v) => v.as_msg_type_parts(),
            Self::CredentialIssuance(v) => v.as_msg_type_parts(),
            Self::ReportProblem(v) => v.as_msg_type_parts(),
            Self::PresentProof(v) => v.as_msg_type_parts(),
            Self::TrustPing(v) => v.as_msg_type_parts(),
            Self::DiscoverFeatures(v) => v.as_msg_type_parts(),
            Self::BasicMessage(v) => v.as_msg_type_parts(),
            Self::OutOfBand(v) => v.as_msg_type_parts(),
            Self::Notification(v) => v.as_msg_type_parts(),
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

impl From<MessageType> for MessageFamily {
    fn from(value: MessageType) -> Self {
        value.family
    }
}
