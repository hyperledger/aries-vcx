pub mod mime_type;
pub mod nothing;
pub(crate) mod utils;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub mod test_utils {
    use serde::Deserialize;
    use serde_json::{json, Value};

    use crate::{
        msg_types::{types::traits::MessageKind, MessageType, Protocol},
        protocols::traits::ConcreteMessage,
        AriesMessage, Message,
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

    pub fn test_msg<T, U>(content: T, decorators: U, mut json: Value)
    where
        AriesMessage: From<Message<T, U>>,
    {
        let id = "test".to_owned();

        let msg = Message::with_decorators(id.clone(), content, decorators);
        let msg = AriesMessage::from(msg);

        json.as_object_mut()
            .expect("JSON object")
            .insert("@id".to_owned(), json!(id));

        let deserialized = AriesMessage::deserialize(&json).unwrap();

        assert_eq!(serde_json::to_value(&msg).unwrap(), json);
        assert_eq!(deserialized, msg);
    }

    pub fn build_msg_type<T>() -> String
    where
        T: ConcreteMessage,
        T::Kind: MessageKind,
        Protocol: From<<T::Kind as MessageKind>::Parent>,
    {
        let kind = T::kind();
        let kind = kind.as_ref();
        let protocol: Protocol = <T::Kind as MessageKind>::parent().into();
        format!("{protocol}/{kind}")
    }
}
