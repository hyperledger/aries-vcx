use crate::{
    a2a::{A2AMessage, MessageId},
    concepts::{thread::Thread, timing::Timing},
    timing_optional,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PingResponse {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

impl PingResponse {
    pub fn create() -> PingResponse {
        PingResponse::default()
    }

    pub fn set_comment(mut self, comment: String) -> PingResponse {
        self.comment = Some(comment);
        self
    }
}

timing_optional!(PingResponse);
threadlike!(PingResponse);
a2a_message!(PingResponse);

#[cfg(feature = "general_test")]
pub mod unit_tests {
    use super::*;
    use crate::protocols::connection::response::test_utils::*;

    fn _comment() -> String {
        String::from("comment")
    }

    pub fn _ping_response() -> PingResponse {
        PingResponse {
            id: MessageId::id(),
            thread: _thread(),
            comment: Some(_comment()),
            timing: None,
        }
    }

    #[test]
    fn test_ping_response_build_works() {
        let ping_response: PingResponse = PingResponse::default()
            .set_comment(_comment())
            .set_thread_id(&_thread_id());

        assert_eq!(_ping_response(), ping_response);
    }
}
