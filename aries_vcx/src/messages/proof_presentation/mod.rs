pub mod presentation;
pub mod presentation_ack;
pub mod presentation_proposal;
pub mod presentation_request;
pub mod presentation_request_internal;

#[cfg(test)]
#[cfg(feature = "test_utils")]
pub mod test_utils {
    use crate::messages::ack;
    use crate::messages::error;
    use crate::messages::proof_presentation::presentation_request::test_utils::_presentation_request;

    pub fn _ack() -> ack::Ack {
        ack::test_utils::_ack().set_thread_id(&_presentation_request().id.0)
    }

    pub fn _problem_report() -> error::ProblemReport {
        error::test_utils::_problem_report().set_thread_id(&_presentation_request().id.0)
    }
}
