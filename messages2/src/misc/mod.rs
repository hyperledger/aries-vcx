pub mod mime_type;
pub mod nothing;
pub(crate) mod utils;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub mod test_utils {
    use crate::msg_types::{MessageType, Protocol};

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
}
