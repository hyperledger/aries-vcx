pub mod mime_type;
pub mod nothing;
pub(crate) mod utils;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub mod test_utils {
    use serde::{Deserialize, Serialize};
    use serde_json::{json, Value};

    use crate::{
        message::Message,
        msg_types::{types::traits::MessageKind, MessageType, Protocol},
        protocols::traits::MessageContent,
        AriesMessage,
    };

    pub fn test_protocol<T>(protocol_str: &str, protocol_type: T)
    where
        Protocol: From<T>,
    {
        let quoted = format!("\"{protocol_str}\"");
        let deserialized = serde_json::from_str(&quoted).unwrap();
        let expected = Protocol::from(protocol_type);
        assert_eq!(expected, deserialized)
    }

    pub fn test_msg_type<T>(protocol_str: &str, kind_str: &str, protocol_type: T)
    where
        Protocol: From<T>,
    {
        let s = format!("\"{protocol_str}/{kind_str}\"");
        let deserialized = serde_json::from_str(&s).unwrap();
        let expected = MessageType {
            protocol: Protocol::from(protocol_type),
            kind: kind_str,
        };
        assert_eq!(expected, deserialized)
    }

    pub fn test_msg<V, T, U>(content: T, decorators: U, mut json: Value)
    where
        AriesMessage: From<Message<T, U>>,
        V: MessageContent,
        V::Kind: MessageKind,
        Protocol: From<<V::Kind as MessageKind>::Parent>,
    {
        let id = "test".to_owned();
        let msg_type = build_msg_type::<V>();

        let obj = json.as_object_mut().expect("JSON object");
        obj.insert("@id".to_owned(), json!(id));
        obj.insert("@type".to_owned(), json!(msg_type));

        let msg = Message::with_decorators(id, content, decorators);
        let msg = AriesMessage::from(msg);

        test_serde(msg, json);
    }

    pub fn test_serde<T>(value: T, json: Value)
    where
        T: for<'de> Deserialize<'de> + Serialize + std::fmt::Debug + PartialEq,
    {
        let deserialized = T::deserialize(&json).unwrap();

        assert_eq!(serde_json::to_value(&value).unwrap(), json);
        assert_eq!(deserialized, value);
    }

    pub fn build_msg_type<T>() -> String
    where
        T: MessageContent,
        T::Kind: MessageKind,
        Protocol: From<<T::Kind as MessageKind>::Parent>,
    {
        let kind = T::kind();
        let kind = kind.as_ref();
        let protocol: Protocol = <T::Kind as MessageKind>::parent().into();
        format!("{protocol}/{kind}")
    }
}
