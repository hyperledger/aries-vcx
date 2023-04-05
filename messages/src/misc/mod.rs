mod mime_type;
mod no_decorators;
pub(crate) mod utils;

pub use mime_type::MimeType;
pub use no_decorators::NoDecorators;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub mod test_utils {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use serde_json::{json, Value};

    use crate::{
        misc::utils::CowStr,
        msg_parts::MsgParts,
        msg_types::{traits::MessageKind, MessageType, Protocol},
        AriesMessage,
    };

    use super::utils;

    pub struct DateTimeRfc3339<'a>(pub &'a DateTime<Utc>);

    impl<'a> Serialize for DateTimeRfc3339<'a> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            utils::serialize_datetime(self.0, serializer)
        }
    }

    pub struct OptDateTimeRfc3339<'a>(pub &'a Option<DateTime<Utc>>);

    impl<'a> Serialize for OptDateTimeRfc3339<'a> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            utils::serialize_opt_datetime(self.0, serializer)
        }
    }

    pub fn test_msg_type<T>(protocol_str: &str, kind_str: &str, protocol_type: T)
    where
        Protocol: From<T>,
    {
        let s = format!("\"{protocol_str}/{kind_str}\"");
        let protocol = Protocol::from(protocol_type);
        let deserialized: CowStr = serde_json::from_str(&s).unwrap();
        let deserialized = MessageType::try_from(deserialized.0.as_ref()).unwrap();

        let expected = MessageType {
            protocol,
            kind: kind_str,
        };

        let serialized = serde_json::to_string(&format_args!("{protocol}/{kind_str}")).unwrap();

        assert_eq!(expected, deserialized);
        assert_eq!(s, serialized);
    }

    pub fn test_msg_type_resolution<T>(protocol_str: &str, protocol_type: T)
    where
        Protocol: From<T>,
    {
        let quoted = format!("\"{protocol_str}\"");
        let deserialized = serde_json::from_str(&quoted).unwrap();
        assert_eq!(Protocol::from(protocol_type), deserialized)
    }

    pub fn test_msg<T, U, V>(content: T, decorators: U, msg_kind: V, mut expected: Value)
    where
        AriesMessage: From<MsgParts<T, U>>,
        V: MessageKind,
        Protocol: From<V::Parent>,
    {
        let id = "test".to_owned();
        let msg_type = build_msg_type(msg_kind);

        let obj = expected.as_object_mut().expect("JSON object");
        obj.insert("@id".to_owned(), json!(id));
        obj.insert("@type".to_owned(), json!(msg_type));

        let msg = MsgParts::with_decorators(id, content, decorators);
        let msg = AriesMessage::from(msg);

        test_serde(msg, expected);
    }

    pub fn test_serde<T>(value: T, expected: Value)
    where
        T: for<'de> Deserialize<'de> + Serialize + std::fmt::Debug + PartialEq,
    {
        // Test serialization
        assert_eq!(serde_json::to_value(&value).unwrap(), expected);

        // Test deserialization from deserializer that owns data:
        let deserialized = T::deserialize(expected.clone()).unwrap();
        assert_eq!(deserialized, value);

        // Test deserialization from deserialized that borrows data:
        let deserialized = T::deserialize(&expected).unwrap();
        assert_eq!(deserialized, value);
    }

    fn build_msg_type<T>(msg_kind: T) -> String
    where
        T: MessageKind,
        Protocol: From<T::Parent>,
    {
        let kind = msg_kind.as_ref();
        let protocol: Protocol = T::parent().into();
        format!("{protocol}/{kind}")
    }
}
