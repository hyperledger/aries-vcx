use messages::msg_fields::protocols::{
    present_proof::v1::{present::PresentationV1, request::RequestPresentationV1},
    report_problem::ProblemReport,
};
use serde::Deserialize;

use crate::{
    handlers::util::Status,
    protocols::proof_presentation::verifier::verification_status::PresentationVerificationStatus,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinishedState {
    pub presentation_request: Option<RequestPresentationV1>,
    pub presentation: Option<PresentationV1>,
    pub status: Status,
    pub verification_status: PresentationVerificationStatus,
}

impl FinishedState {
    pub fn declined(problem_report: ProblemReport) -> Self {
        trace!("transit state to FinishedState due to a rejection");
        FinishedState {
            presentation_request: None,
            presentation: None,
            status: Status::Declined(problem_report),
            verification_status: PresentationVerificationStatus::Unavailable,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub mod unit_tests {
    use super::*;

    #[test]
    fn test_verifier_state_finished_ser_deser_valid() {
        let state = FinishedState {
            presentation_request: None,
            presentation: None,
            status: Status::Success,
            verification_status: PresentationVerificationStatus::Valid,
        };
        let serialized = serde_json::to_string(&state).unwrap();
        let expected = r#"{"presentation_request":null,"presentation":null,"status":"Success","verification_status":"Valid"}"#;
        assert_eq!(serialized, expected);
        let deserialized: FinishedState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(state, deserialized)
    }

    #[test]
    fn test_verifier_state_finished_ser_deser_unavailable() {
        let state = FinishedState {
            presentation_request: None,
            presentation: None,
            status: Status::Success,
            verification_status: PresentationVerificationStatus::Unavailable,
        };
        let serialized = serde_json::to_string(&state).unwrap();
        let expected = r#"{"presentation_request":null,"presentation":null,"status":"Success","verification_status":"Unavailable"}"#;
        assert_eq!(serialized, expected);
        let deserialized: FinishedState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(state, deserialized)
    }

    #[test]
    fn test_verifier_state_finished_ser_deser_invalid() {
        let state = FinishedState {
            presentation_request: None,
            presentation: None,
            status: Status::Success,
            verification_status: PresentationVerificationStatus::Invalid,
        };
        let serialized = serde_json::to_string(&state).unwrap();
        let expected = r#"{"presentation_request":null,"presentation":null,"status":"Success","verification_status":"Invalid"}"#;
        assert_eq!(serialized, expected);
        let deserialized: FinishedState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(state, deserialized)
    }
}
