use ::uuid::Uuid;
use chrono::Utc;
use messages::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::trust_ping::{
        ping::{Ping, PingContent, PingDecorators},
        ping_response::{PingResponse, PingResponseDecorators},
    },
    AriesMessage,
};

pub fn build_ping(request_response: bool, comment: Option<String>) -> Ping {
    let content = PingContent::builder().response_requested(request_response);
    let content = match comment {
        None => content.build(),
        Some(comment) => content.comment(comment).build(),
    };

    let decorators = PingDecorators::builder()
        .timing(Timing::builder().out_time(Utc::now()).build())
        .build();

    Ping::builder()
        .id(Uuid::new_v4().to_string())
        .content(content)
        .decorators(decorators)
        .build()
}

pub fn build_ping_response(ping: &Ping) -> PingResponse {
    let thread_id = ping
        .decorators
        .thread
        .as_ref()
        .map(|t| t.thid.as_str())
        .unwrap_or(ping.id.as_str())
        .to_owned();

    let decorators = PingResponseDecorators::builder()
        .thread(Thread::builder().thid(thread_id).build())
        .timing(Timing::builder().out_time(Utc::now()).build())
        .build();

    PingResponse::builder()
        .id(Uuid::new_v4().to_string())
        .decorators(decorators)
        .build()
}

pub fn build_ping_response_msg(ping: &Ping) -> AriesMessage {
    build_ping_response(ping).into()
}

// #[cfg(test)]
// pub mod unit_tests {
//     use messages::protocols::trust_ping::ping::unit_tests::{_ping, _ping_no_thread};
//     use messages::protocols::trust_ping::ping_response::unit_tests::_ping_response;

//     use super::*;

//     #[test]
//     fn test_build_ping_response_works() {
//         assert_eq!(
//             build_ping_response(&_ping()).get_thread_id(),
//             _ping_response().get_thread_id()
//         );
//         assert_eq!(
//             build_ping_response(&_ping_no_thread()).get_thread_id(),
//             _ping_response().get_thread_id()
//         );
//         assert_eq!(
//             build_ping_response(&_ping_no_thread()).get_thread_id(),
//             _ping_no_thread().id.0
//         );
//     }
// }
