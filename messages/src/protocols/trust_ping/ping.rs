use crate::{
    a2a::{A2AMessage, MessageId},
    concepts::{thread::Thread, timing::Timing},
    timing_optional,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Ping {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(default)]
    pub response_requested: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

impl Ping {
    pub fn create(thread_id: MessageId) -> Ping {
        Ping {
            id: thread_id,
            ..Default::default()
        }
    }

    pub fn set_comment(mut self, comment: Option<String>) -> Ping {
        self.comment = comment;
        self
    }

    pub fn set_request_response(mut self, request_response: bool) -> Ping {
        self.response_requested = request_response;
        self
    }
}

timing_optional!(Ping);
threadlike_optional!(Ping);
a2a_message!(Ping);

#[cfg(feature = "general_test")]
pub mod unit_tests {
    use super::*;
    use crate::protocols::connection::response::test_utils::*;

    fn _comment() -> String {
        String::from("comment")
    }

    pub fn _ping() -> Ping {
        Ping {
            id: MessageId::id(),
            response_requested: false,
            thread: Some(_thread()),
            comment: Some(_comment()),
            timing: None,
        }
    }

    pub fn _ping_no_thread() -> Ping {
        Ping {
            id: MessageId::id(),
            response_requested: false,
            thread: None,
            comment: Some(_comment()),
            timing: None,
        }
    }

    #[test]
    fn test_ping_build_works() {
        let ping: Ping = Ping::default()
            .set_comment(Some(_comment()))
            .set_thread_id(&_thread_id());

        assert_eq!(_ping(), ping);
    }
}
