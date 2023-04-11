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
