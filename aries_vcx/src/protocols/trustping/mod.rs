use crate::handlers::trust_ping::TrustPingSender;
use crate::messages::a2a::MessageId;
use crate::messages::trust_ping::ping::Ping;
use crate::messages::trust_ping::ping_response::PingResponse;
use crate::utils::uuid;

pub fn build_ping(request_response: bool, comment: Option<String>) -> Ping {
    // todo : Remove different Default implementation for MessageId in tests, then we can remove this override
    Ping::create(MessageId(uuid::uuid()))
        .set_out_time()
        .set_request_response(request_response)
        .set_comment(comment)
}

pub fn build_ping_response(ping: &Ping) -> PingResponse {
    let thread_id = ping
        .thread
        .as_ref()
        .and_then(|thread| thread.thid.clone())
        .unwrap_or(ping.id.0.clone());
    PingResponse::create().set_out_time().set_thread_id(&thread_id)
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use crate::messages::trust_ping::ping::unit_tests::{_ping, _ping_no_thread};
    use crate::messages::trust_ping::ping_response::unit_tests::_ping_response;

    use super::*;

    #[test]
    fn test_build_ping_response_works() {
        assert_eq!(
            build_ping_response(&_ping()).get_thread_id(),
            _ping_response().get_thread_id()
        );
        assert_eq!(
            build_ping_response(&_ping_no_thread()).get_thread_id(),
            _ping_response().get_thread_id()
        );
        assert_eq!(
            build_ping_response(&_ping_no_thread()).get_thread_id(),
            _ping_no_thread().id.0
        );
    }
}
