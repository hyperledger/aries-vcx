use messages::{
    decorators::thread::Thread,
    msg_fields::protocols::report_problem::{
        Description, ProblemReport, ProblemReportContent, ProblemReportDecorators,
    },
};
use uuid::Uuid;

pub fn build_problem_report_msg(comment: Option<String>, thread_id: &str) -> ProblemReport {
    let id = Uuid::new_v4().to_string();
    let content = ProblemReportContent::builder()
        .description(
            Description::builder()
                .code(comment.unwrap_or_default())
                .build(),
        )
        .build();

    let decorators = ProblemReportDecorators::builder()
        .thread(Thread::builder().thid(thread_id.to_owned()).build())
        .build();

    ProblemReport::builder()
        .id(id)
        .content(content)
        .decorators(decorators)
        .build()
}

// #[cfg(test)]
// mod test {
//     use crate::protocols::common::build_problem_report_msg;
//     use crate::utils::devsetup::{was_in_past, SetupMocks};

//     #[test]
//     fn test_holder_build_problem_report_msg() {
//         let _setup = SetupMocks::init();
//         let msg = build_problem_report_msg(Some("foo".into()), "12345");

//         assert_eq!(msg.id, "test");
//         assert_eq!(msg.decorators.thread.unwrap().thid, "12345");
//         assert!(was_in_past(
//             &msg.decorators.timing.unwrap().out_time.unwrap(),
//             chrono::Duration::milliseconds(100),
//         )
//         .unwrap());
//     }
// }
