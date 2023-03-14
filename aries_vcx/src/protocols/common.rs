use messages::concepts::problem_report::ProblemReport;

pub fn build_problem_report_msg(comment: Option<String>, thread_id: &str) -> ProblemReport {
    ProblemReport::create()
        .set_out_time()
        .set_comment(comment)
        .set_thread_id(thread_id)
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod test {
    use messages::a2a::MessageId;

    use crate::{
        protocols::common::build_problem_report_msg,
        utils::devsetup::{was_in_past, SetupMocks},
    };

    #[test]
    #[cfg(feature = "general_test")]
    fn test_holder_build_problem_report_msg() {
        let _setup = SetupMocks::init();
        let msg = build_problem_report_msg(Some("foo".into()), "12345");

        assert_eq!(msg.id, MessageId::default());
        assert_eq!(msg.thread.unwrap().thid.unwrap(), "12345");
        assert!(was_in_past(
            &msg.timing.unwrap().out_time.unwrap(),
            chrono::Duration::milliseconds(100),
        )
        .unwrap());
    }
}
