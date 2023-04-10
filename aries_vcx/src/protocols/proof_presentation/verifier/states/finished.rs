use serde::{Deserialize, Deserializer};

use messages::concepts::problem_report::ProblemReport;
use messages::protocols::proof_presentation::presentation::Presentation;
use messages::protocols::proof_presentation::presentation_request::PresentationRequest;
use messages::status::Status;

use crate::protocols::proof_presentation::verifier::verification_status::PresentationVerificationStatus;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinishedState {
    pub presentation_request: Option<PresentationRequest>,
    pub presentation: Option<Presentation>,
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
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use std::str::FromStr;

    use messages::protocols::proof_presentation::presentation::test_utils::{_presentation, _presentation_1};
    use messages::protocols::proof_presentation::presentation_proposal::test_utils::_presentation_proposal;
    use messages::protocols::proof_presentation::presentation_request::test_utils::_presentation_request;
    use messages::protocols::proof_presentation::test_utils::{_ack, _problem_report};

    use crate::common::test_utils::mock_profile;
    use crate::test::source_id;
    use crate::utils::devsetup::{SetupEmpty, SetupMocks};

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
        let expected =
            r#"{"presentation_request":null,"presentation":null,"status":"Success","verification_status":"Valid"}"#;
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
        let expected =
            r#"{"presentation_request":null,"presentation":null,"status":"Success","verification_status":"Invalid"}"#;
        assert_eq!(serialized, expected);
        let deserialized: FinishedState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(state, deserialized)
    }
}
