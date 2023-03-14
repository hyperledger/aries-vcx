use crate::{
    a2a::{A2AMessage, MessageId},
    actors::Actors,
    concepts::{thread::Thread, timing::Timing},
    timing_optional,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Disclose {
    #[serde(rename = "@id")]
    pub id: MessageId,
    pub protocols: Vec<ProtocolDescriptor>,
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

threadlike_optional!(Disclose);
timing_optional!(Disclose);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProtocolDescriptor {
    pub pid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<Actors>>,
}

impl Disclose {
    pub fn create() -> Disclose {
        Disclose::default()
    }

    pub fn set_protocols(mut self, protocols: Vec<ProtocolDescriptor>) -> Self {
        self.protocols = protocols;
        self
    }

    pub fn add_protocol(&mut self, protocol: ProtocolDescriptor) {
        self.protocols.push(protocol);
    }

    pub fn to_a2a_message(&self) -> A2AMessage {
        A2AMessage::Disclose(self.clone()) // TODO: THINK how to avoid clone
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use super::*;
    use crate::protocols::connection::response::test_utils::*;

    pub fn _protocol_descriptor() -> ProtocolDescriptor {
        ProtocolDescriptor {
            pid: String::from("https://didcomm.org/"),
            roles: None,
        }
    }

    pub fn _disclose() -> Disclose {
        Disclose {
            id: MessageId::id(),
            protocols: vec![_protocol_descriptor()],
            thread: Some(_thread()),
            timing: None,
        }
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use super::*;
    use crate::protocols::{
        connection::response::test_utils::*,
        discovery::disclose::test_utils::{_disclose, _protocol_descriptor},
    };

    #[test]
    fn test_disclose_build_works() {
        let mut disclose: Disclose = Disclose::default().set_thread_id(&_thread_id());

        disclose.add_protocol(_protocol_descriptor());

        assert_eq!(_disclose(), disclose);
    }
}
