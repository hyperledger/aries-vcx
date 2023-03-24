use std::{fmt::Display, str::FromStr};

use derive_more::{From, TryInto};
use serde::{Deserialize, Serialize};

use self::{
    basic_message::BasicMessageProtocol, connection::ConnectionProtocol, cred_issuance::CredentialIssuanceProtocol,
    discover_features::DiscoverFeaturesProtocol, notification::NotificationProtocol, out_of_band::OutOfBandProtocol,
    present_proof::PresentProofProtocol, report_problem::ReportProblemProtocol, revocation::RevocationProtocol,
    routing::RoutingProtocol, trust_ping::TrustPingProtocol,
};
use crate::{
    error::{MsgTypeError, MsgTypeResult},
    msg_types::traits::ProtocolName,
};

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
pub mod trust_ping;

/// Type representing all protocols that are currently supported.
///
/// They are composed from protocol names, protocol major versions and protocol minor versions.
/// The protocol message kind types, while linked to their respective protocol minor versions,
/// are treated adjacently to this enum.
///
/// This way, this type can be used for all of the following:
/// - protocol registry
/// - message type deserialization
/// - discover features protocol
///
/// The way a message kind (e.g: `request`) is bound to a [`Protocol`] (e.g: `https://didcomm.org/connections/1.0`)
/// is through our internal [`messages_macros::MessageType`] proc_macro. See the docs for that for
/// more info.
#[derive(Clone, Copy, Debug, From, TryInto, PartialEq, Deserialize)]
#[serde(try_from = "&str")]
pub enum Protocol {
    RoutingProtocol(RoutingProtocol),
    ConnectionProtocol(ConnectionProtocol),
    RevocationProtocol(RevocationProtocol),
    CredentialIssuanceProtocol(CredentialIssuanceProtocol),
    ReportProblemProtocol(ReportProblemProtocol),
    PresentProofProtocol(PresentProofProtocol),
    TrustPingProtocol(TrustPingProtocol),
    DiscoverFeaturesProtocol(DiscoverFeaturesProtocol),
    BasicMessageProtocol(BasicMessageProtocol),
    OutOfBandProtocol(OutOfBandProtocol),
    NotificationProtocol(NotificationProtocol),
}

/// Utility macro to avoid harder to read and error prone calling
/// of the version resolution method on the correct type.
macro_rules! match_protocol {
    ($type:ident, $protocol:expr, $major:expr, $minor:expr) => {
        if $protocol == $type::PROTOCOL {
            return Ok(Self::$type($type::try_from_version_parts($major, $minor)?));
        }
    };
}

impl Protocol {
    pub const DID_COM_ORG_PREFIX: &'static str = "https://didcomm.org";
    pub const DID_SOV_PREFIX: &'static str = "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec";

    /// Tried to construct a [`Protocol`] from parts.
    ///
    /// # Errors:
    ///
    /// An error is returned if a [`Protocol`] could not be constructed
    /// from the provided parts.
    pub fn from_parts(protocol: &str, major: u8, minor: u8) -> MsgTypeResult<Self> {
        match_protocol!(RoutingProtocol, protocol, major, minor);
        match_protocol!(ConnectionProtocol, protocol, major, minor);
        match_protocol!(RevocationProtocol, protocol, major, minor);
        match_protocol!(CredentialIssuanceProtocol, protocol, major, minor);
        match_protocol!(ReportProblemProtocol, protocol, major, minor);
        match_protocol!(PresentProofProtocol, protocol, major, minor);
        match_protocol!(TrustPingProtocol, protocol, major, minor);
        match_protocol!(DiscoverFeaturesProtocol, protocol, major, minor);
        match_protocol!(BasicMessageProtocol, protocol, major, minor);
        match_protocol!(OutOfBandProtocol, protocol, major, minor);
        match_protocol!(NotificationProtocol, protocol, major, minor);

        Err(MsgTypeError::unknown_protocol(protocol.to_owned()))
    }

    /// Returns the parts that this [`Protocol`] is comprised of.
    pub fn as_parts(&self) -> (&'static str, u8, u8) {
        match &self {
            Self::RoutingProtocol(v) => v.as_protocol_parts(),
            Self::ConnectionProtocol(v) => v.as_protocol_parts(),
            Self::RevocationProtocol(v) => v.as_protocol_parts(),
            Self::CredentialIssuanceProtocol(v) => v.as_protocol_parts(),
            Self::ReportProblemProtocol(v) => v.as_protocol_parts(),
            Self::PresentProofProtocol(v) => v.as_protocol_parts(),
            Self::TrustPingProtocol(v) => v.as_protocol_parts(),
            Self::DiscoverFeaturesProtocol(v) => v.as_protocol_parts(),
            Self::BasicMessageProtocol(v) => v.as_protocol_parts(),
            Self::OutOfBandProtocol(v) => v.as_protocol_parts(),
            Self::NotificationProtocol(v) => v.as_protocol_parts(),
        }
    }

    /// Steps the provided iterator of parts and returns the string slice element.
    ///
    /// # Errors:
    ///
    /// Will return an error if the iterator returns [`None`].
    pub(crate) fn next_part<'a, I>(iter: &mut I, name: &'static str) -> MsgTypeResult<&'a str>
    where
        I: Iterator<Item = &'a str>,
    {
        iter.next().ok_or_else(|| MsgTypeError::not_found(name))
    }
}

impl Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = Self::DID_COM_ORG_PREFIX;
        let (protocol, major, minor) = self.as_parts();
        write!(f, "{prefix}/{protocol}/{major}.{minor}")
    }
}

impl FromStr for Protocol {
    type Err = MsgTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // The type is segmented by forward slashes, but the HTTPS
        // prefix includes two as well (https://), so we'll accommodate that
        // when we skip elements from the split string.
        //
        // We always skip at least one element, the prefix itself.
        let skip_slash = match s {
            _ if s.starts_with(Self::DID_COM_ORG_PREFIX) => Ok(3),
            _ if s.starts_with(Self::DID_SOV_PREFIX) => Ok(1),
            _ => Err(MsgTypeError::unknown_prefix(s.to_owned())),
        }?;

        // We'll get the next components in order
        let mut iter = s.split('/').skip(skip_slash);

        let protocol_name = Protocol::next_part(&mut iter, "protocol name")?;
        let version = Protocol::next_part(&mut iter, "protocol version")?;

        // We'll parse the version to its major and minor parts
        let mut version_iter = version.split('.');

        let major = Protocol::next_part(&mut version_iter, "protocol major version")?.parse()?;
        let minor = Protocol::next_part(&mut version_iter, "protocol minor version")?.parse()?;

        Protocol::from_parts(protocol_name, major, minor)
    }
}

impl<'a> TryFrom<&'a str> for Protocol {
    type Error = MsgTypeError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl Serialize for Protocol {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let prefix = Self::DID_COM_ORG_PREFIX;
        let (protocol, major, minor) = self.as_parts();
        format_args!("{prefix}/{protocol}/{major}.{minor}").serialize(serializer)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_old_prefix() {
        let value_old = json!("did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0");
        let value_new = json!("https://didcomm.org/connections/1.0");

        let protocol_old = Protocol::deserialize(&value_old).unwrap();
        let protocol_new = Protocol::deserialize(&value_new).unwrap();

        assert_eq!(protocol_old, protocol_new);
    }
}
