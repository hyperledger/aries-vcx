pub mod actor;
pub mod message_family;
mod prefix;
pub mod registry;

use std::{fmt::Display, str::FromStr};

use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use crate::error::MsgTypeError;

use self::prefix::Prefix;

pub use self::message_family::MessageFamily;

#[derive(Clone, Debug, PartialEq)]
pub struct MessageType {
    prefix: Prefix,
    pub family: MessageFamily,
}

impl MessageType {
    pub fn new(prefix: Prefix, family: MessageFamily) -> Self {
        Self { prefix, family }
    }
}

impl Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix_str = self.prefix.as_ref();
        let (family, major, minor, kind) = self.family.as_parts();
        write!(f, "{prefix_str}/{family}/{major}.{minor}/{kind}")
    }
}

impl From<MessageFamily> for MessageType {
    fn from(value: MessageFamily) -> Self {
        Self::new(Prefix::default(), value)
    }
}

impl FromStr for MessageType {
    type Err = MsgTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // The type is segmented by forward slashes, but the HTTPS
        // prefix includes two as well (https://), so we'll accommodate that
        // when we skip elements from the split string.
        //
        // We always skip at least one element, the prefix itself.
        //
        // While we're at it we can also store the Prefix itself.
        let (prefix, skip_slash) = match s {
            _ if s.starts_with(Prefix::DID_COM_ORG_PREFIX) => Ok((Prefix::DidCommOrg, 3)),
            _ if s.starts_with(Prefix::DID_SOV_PREFIX) => Ok((Prefix::DidSov, 1)),
            _ => Err(MsgTypeError::unknown_prefix(s.to_owned())),
        }?;

        // We'll get the next components in order
        let mut iter = s.split('/').skip(skip_slash);

        let family = MessageFamily::next_part(&mut iter, "family")?;
        let version = MessageFamily::next_part(&mut iter, "protocol version")?;
        let kind = MessageFamily::next_part(&mut iter, "message kind")?;

        // We'll parse the version to its major and minor parts
        let mut version_iter = version.split('.');

        let major = MessageFamily::next_part(&mut version_iter, "protocol major version")?.parse()?;
        let minor = MessageFamily::next_part(&mut version_iter, "protocol minor version")?.parse()?;

        let msg_family = MessageFamily::from_parts(family, major, minor, kind)?;
        let msg_type = MessageType::new(prefix, msg_family);

        Ok(msg_type)
    }
}

impl<'de> Deserialize<'de> for MessageType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let type_str = <&str>::deserialize(deserializer)?;
        type_str
            .parse()
            .map_err(|e| D::Error::custom(format!("Message type {type_str}; {e}")))
    }
}

impl Serialize for MessageType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::message_type::message_family::basic_message::BasicMessageV1_0;
    use crate::message_type::message_family::connection::ConnectionV1_0;

    #[test]
    fn from_string_basicmessage() {
        // This From conversion is generated using transitive_from! macro
        let msg_type: MessageType = MessageFamily::into(BasicMessageV1_0::Message.into());
        assert_eq!(msg_type.to_string(), "https://didcomm.org/basicmessage/1.0/message");
    }

    #[test]
    fn from_string_connections() {
        let msg_type: MessageType = MessageFamily::into(ConnectionV1_0::Invitation.into());
        assert_eq!(msg_type.to_string(), "https://didcomm.org/connections/1.0/invitation");
        let msg_type: MessageType = MessageFamily::into(ConnectionV1_0::Request.into());
        assert_eq!(msg_type.to_string(), "https://didcomm.org/connections/1.0/request");
        let msg_type: MessageType = MessageFamily::into(ConnectionV1_0::Response.into());
        assert_eq!(msg_type.to_string(), "https://didcomm.org/connections/1.0/response");
        let msg_type: MessageType = MessageFamily::into(ConnectionV1_0::ProblemReport.into());
        assert_eq!(
            msg_type.to_string(),
            "https://didcomm.org/connections/1.0/problem_report"
        );
    }
}
