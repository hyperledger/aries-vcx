use crate::{a2a::MessageId, errors::error::prelude::*};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct Forward {
    #[serde(rename = "@id")]
    pub id: MessageId,
    pub to: String,
    #[serde(rename = "msg")]
    pub msg: serde_json::Value,
}

impl Forward {
    pub fn new(to: String, msg: Vec<u8>) -> MessagesResult<Forward> {
        let msg = serde_json::from_slice(msg.as_slice())
            .map_err(|err| MessagesError::from_msg(MessagesErrorKind::InvalidState, err))?;

        Ok(Forward {
            id: MessageId::new(),
            to,
            msg,
        })
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use super::*;
    use crate::concepts::ack::test_utils::*;

    fn _to() -> String {
        String::from("GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL")
    }

    fn _msg() -> serde_json::Value {
        json!(_ack())
    }

    fn _forward() -> Forward {
        Forward {
            id: MessageId::default(),
            to: _to(),
            msg: _msg(),
        }
    }

    #[test]
    fn test_forward_build_works() {
        let message = serde_json::to_vec(&_ack()).unwrap();
        let forward: Forward = Forward::new(_to(), message).unwrap();
        assert_eq!(_forward(), forward);
    }
}
