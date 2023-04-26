#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum PresentationVerificationStatus {
    Valid,
    Invalid,
    Unavailable,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub mod unit_tests {
    use super::*;

    #[test]
    fn test_presentation_status_ser_deser() {
        assert_eq!(
            PresentationVerificationStatus::Valid,
            serde_json::from_str("\"Valid\"").unwrap()
        );
        assert_eq!(
            PresentationVerificationStatus::Invalid,
            serde_json::from_str("\"Invalid\"").unwrap()
        );
        assert_eq!(
            PresentationVerificationStatus::Unavailable,
            serde_json::from_str("\"Unavailable\"").unwrap()
        );
    }
}
