use std::marker::PhantomData;

use messages::msg_fields::protocols::cred_issuance::v2::problem_report::CredIssuanceProblemReportV2;

use crate::protocols::issuance_v2::formats::issuer::IssuerCredentialIssuanceFormat;

pub struct Failed<T: IssuerCredentialIssuanceFormat> {
    pub problem_report: CredIssuanceProblemReportV2,
    pub _marker: PhantomData<T>,
}
