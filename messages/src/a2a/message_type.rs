use regex::{Match, Regex};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::a2a::message_family::MessageFamilies;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MessageType {
    pub prefix: String,
    pub family: MessageFamilies,
    pub version: String,
    pub msg_type: String,
}

impl MessageType {
    pub fn build(family: MessageFamilies, name: &str) -> MessageType {
        MessageType {
            prefix: MessageFamilies::ARIES_CORE_PREFIX.to_string(),
            version: family.version().to_string(),
            family,
            msg_type: name.to_string(),
        }
    }
}

impl<'de> Deserialize<'de> for MessageType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;

        match value.as_str() {
            Some(type_) => {
                if let Some(msg_type) = parse_message_type_legacy(type_) {
                    Ok(msg_type)
                } else if let Some(msg_type) = parse_message_type(type_) {
                    Ok(msg_type)
                } else {
                    Err(de::Error::custom("Unexpected @type field structure."))
                }
            }
            _ => Err(de::Error::custom("Unexpected @type field structure.")),
        }
    }
}

impl Serialize for MessageType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = Value::String(self.to_string());
        value.serialize(serializer)
    }
}

impl std::string::ToString for MessageType {
    fn to_string(&self) -> String {
        if self.family == MessageFamilies::Routing {
            // vcxagency-node only supports legacy format right now
            format!(
                "{};spec/{}/{}/{}",
                MessageFamilies::DID_PREFIX,
                self.family.to_string(),
                self.version,
                self.msg_type
            )
        } else {
            format!(
                "{}/{}/{}/{}",
                self.prefix,
                self.family.to_string(),
                self.version,
                self.msg_type
            )
        }
    }
}

pub fn parse_message_type_legacy(message_type: &str) -> Option<MessageType> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"(?x)
            (?P<did>[\d\w:]*);
            (?P<spec>.*)/
            (?P<family>.*)/
            (?P<version>.*)/
            (?P<type>.*)"
        )
        .unwrap();
    }

    RE.captures(message_type).and_then(|cap| {
        let did = cap.name("did").as_ref().map(Match::as_str);
        let family = cap.name("family").as_ref().map(Match::as_str);
        let version = cap.name("version").as_ref().map(Match::as_str);
        let msg_type = cap.name("type").as_ref().map(Match::as_str);

        match (did, family, version, msg_type) {
            (Some(did), Some(family), Some(version), Some(msg_type)) => Some(MessageType {
                prefix: did.to_string(),
                family: MessageFamilies::from(family.to_string()),
                version: version.to_string(),
                msg_type: msg_type.to_string(),
            }),
            _ => {
                panic!("Message type regex captures, but failed to map it onto MessageType structure.")
            }
        }
    })
}

pub fn parse_message_type(message_type: &str) -> Option<MessageType> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"(?x)
            (?P<prefix>.+)/  # https://didcomm.org/
            (?P<family>.+)/  # connections/
            (?P<version>.+)/ # 1.0/
            (?P<type>.+)     # request
            "
        )
        .unwrap();
    }

    RE.captures(message_type).and_then(|cap| {
        let prefix = cap.name("prefix").as_ref().map(Match::as_str);
        let family = cap.name("family").as_ref().map(Match::as_str);
        let version = cap.name("version").as_ref().map(Match::as_str);
        let msg_type = cap.name("type").as_ref().map(Match::as_str);

        match (prefix, family, version, msg_type) {
            (Some(prefix), Some(family), Some(version), Some(msg_type)) => Some(MessageType {
                prefix: prefix.to_string(),
                family: MessageFamilies::from(family.to_string()),
                version: version.to_string(),
                msg_type: msg_type.to_string(),
            }),
            _ => {
                panic!("Message type regex captures, but failed to map it onto MessageType structure.")
            }
        }
    })
}
