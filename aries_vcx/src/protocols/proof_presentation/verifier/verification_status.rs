use serde::{Deserialize, Deserializer};

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub enum PresentationVerificationStatus {
    Valid,
    Invalid,
    Unavailable(),
}

impl<'de> Deserialize<'de> for PresentationVerificationStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.as_str() {
            "Valid" | "NonRevoked" => Ok(PresentationVerificationStatus::Valid),
            "Invalid" | "Revoked" => Ok(PresentationVerificationStatus::Invalid),
            "Unavailable" => Ok(PresentationVerificationStatus::Unavailable()),
            _ => Err(serde::de::Error::custom(format!("unexpected value for Color: {}", s))),
        }
    }
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
            PresentationVerificationStatus::Unavailable(),
            serde_json::from_str("\"Unavailable\"").unwrap()
        );
    }
}
