use messages::msg_fields::protocols::{
    present_proof::{present::Presentation, request::RequestPresentation},
    report_problem::ProblemReport,
};

use crate::handlers::util::Status;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinishedState {
    pub presentation_request: Option<RequestPresentation>,
    pub presentation: Option<Presentation>,
    pub status: Status,
}

impl FinishedState {
    pub fn declined(problem_report: ProblemReport) -> Self {
        trace!("transit state to FinishedState due to a rejection");
        FinishedState {
            presentation_request: None,
            presentation: None,
            status: Status::Declined(problem_report),
        }
    }
}
