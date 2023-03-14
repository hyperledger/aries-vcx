#[macro_use]
pub mod please_ack;

use crate::{
    a2a::{A2AMessage, MessageId},
    concepts::{thread::Thread, timing::Timing},
    timing_optional,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Ack {
    #[serde(rename = "@id")]
    pub id: MessageId,
    pub status: AckStatus,
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

threadlike!(Ack);
a2a_message!(Ack);
timing_optional!(Ack);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AckStatus {
    #[serde(rename = "OK")]
    Ok,
    #[serde(rename = "FAIL")]
    Fail,
    #[serde(rename = "PENDING")]
    Pending,
}

impl Default for AckStatus {
    fn default() -> AckStatus {
        AckStatus::Ok
    }
}

impl Ack {
    pub fn create() -> Ack {
        Ack::default()
    }

    pub fn set_status(mut self, status: AckStatus) -> Ack {
        self.status = status;
        self
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use super::*;
    use crate::protocols::connection::response::test_utils::{_thread, _thread_1, _thread_random};

    pub fn _ack() -> Ack {
        Ack {
            id: MessageId::id(),
            status: AckStatus::Fail,
            thread: _thread(),
            timing: None,
        }
    }

    pub fn _ack_random_thread() -> Ack {
        Ack {
            id: MessageId::id(),
            status: AckStatus::Ok,
            thread: _thread_random(),
            timing: None,
        }
    }

    pub fn _ack_1() -> Ack {
        Ack {
            id: MessageId::id(),
            status: AckStatus::Fail,
            thread: _thread_1(),
            timing: None,
        }
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use super::*;
    use crate::{concepts::ack::test_utils::_ack, protocols::connection::response::test_utils::_thread_id};

    #[test]
    fn test_ack_build_works() {
        let ack: Ack = Ack::default().set_status(AckStatus::Fail).set_thread_id(&_thread_id());

        assert_eq!(_ack(), ack);
    }
}
