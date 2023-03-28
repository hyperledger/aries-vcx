#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum PresentationVerificationStatus {
    #[serde(alias = "NonRevoked")]
    // todo: to be removed in 0.54.0, supports legacy serialization when the enum had values "Revoked" and "NotRevoked"
    Valid,
    #[serde(alias = "Revoked")]
    Invalid,
    Unavailable,
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use std::str::FromStr;

    use crate::common::proofs::proof_request::test_utils::_presentation_request_data;
    use crate::common::test_utils::mock_profile;
    use crate::test::source_id;
    use crate::utils::devsetup::{SetupEmpty, SetupMocks};
    use messages::protocols::proof_presentation::presentation::test_utils::{_presentation, _presentation_1};
    use messages::protocols::proof_presentation::presentation_proposal::test_utils::_presentation_proposal;
    use messages::protocols::proof_presentation::presentation_request::test_utils::_presentation_request;
    use messages::protocols::proof_presentation::test_utils::{_ack, _problem_report};

    use super::*;

    #[test]
    fn test_presentation_status_ser_deser() {
        assert_eq!(
            PresentationVerificationStatus::Valid,
            serde_json::from_str("\"Valid\"").unwrap()
        );
        assert_eq!(
            PresentationVerificationStatus::Valid,
            serde_json::from_str("\"NonRevoked\"").unwrap()
        );
        assert_eq!(
            PresentationVerificationStatus::Invalid,
            serde_json::from_str("\"Invalid\"").unwrap()
        );
        assert_eq!(
            PresentationVerificationStatus::Invalid,
            serde_json::from_str("\"Revoked\"").unwrap()
        );
        assert_eq!(
            PresentationVerificationStatus::Unavailable,
            serde_json::from_str("\"Unavailable\"").unwrap()
        );
    }
}
