use messages::msg_fields::protocols::cred_issuance::v2::problem_report::CredIssuanceProblemReportV2;

pub struct Failed {
    pub(crate) problem_report: CredIssuanceProblemReportV2,
}

impl Failed {
    pub fn new(problem_report: CredIssuanceProblemReportV2) -> Self {
        Self { problem_report }
    }

    pub fn get_problem_report(&self) -> &CredIssuanceProblemReportV2 {
        &self.problem_report
    }
}
