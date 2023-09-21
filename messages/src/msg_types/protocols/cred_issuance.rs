use derive_more::{From, TryInto};
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::Transitive;

use super::Protocol;
use crate::msg_types::{role::Role, MsgKindType};

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, MessageType)]
#[msg_type(protocol = "issue-credential")]
pub enum CredentialIssuanceType {
    V1(CredentialIssuanceTypeV1),
    V2(CredentialIssuanceTypeV2),
}

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, Transitive, MessageType)]
#[transitive(into(CredentialIssuanceType, Protocol))]
#[msg_type(major = 1)]
pub enum CredentialIssuanceTypeV1 {
    #[msg_type(minor = 0, roles = "Role::Holder, Role::Issuer")]
    V1_0(MsgKindType<CredentialIssuanceTypeV1_0>),
}

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, Transitive, MessageType)]
#[transitive(into(CredentialIssuanceType, Protocol))]
#[msg_type(major = 2)]
pub enum CredentialIssuanceTypeV2 {
    #[msg_type(minor = 0, roles = "Role::Holder, Role::Issuer")]
    V2_0(MsgKindType<CredentialIssuanceTypeV2_0>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum CredentialIssuanceTypeV1_0 {
    OfferCredential,
    ProposeCredential,
    RequestCredential,
    IssueCredential,
    CredentialPreview,
    Ack,
    ProblemReport,
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum CredentialIssuanceTypeV2_0 {
    OfferCredential,
    ProposeCredential,
    RequestCredential,
    IssueCredential,
    CredentialPreview,
    Ack,
    ProblemReport,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    #[test]
    fn test_protocol_issue_credential_v1() {
        test_utils::test_serde(
            Protocol::from(CredentialIssuanceTypeV1::new_v1_0()),
            json!("https://didcomm.org/issue-credential/1.0"),
        )
    }

    #[test]
    fn test_version_resolution_issue_credential_v1() {
        test_utils::test_msg_type_resolution(
            "https://didcomm.org/issue-credential/1.255",
            CredentialIssuanceTypeV1::new_v1_0(),
        )
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_issue_credential_v1() {
        test_utils::test_serde(
            Protocol::from(CredentialIssuanceTypeV1::new_v1_0()),
            json!("https://didcomm.org/issue-credential/2.0"),
        )
    }

    #[test]
    fn test_msg_type_offer_v1() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/1.0",
            "offer-credential",
            CredentialIssuanceTypeV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_propose_v1() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/1.0",
            "propose-credential",
            CredentialIssuanceTypeV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_request_v1() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/1.0",
            "request-credential",
            CredentialIssuanceTypeV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_issue_v1() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/1.0",
            "issue-credential",
            CredentialIssuanceTypeV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_preview_v1() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/1.0",
            "credential-preview",
            CredentialIssuanceTypeV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_ack_v1() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/1.0",
            "ack",
            CredentialIssuanceTypeV1::new_v1_0(),
        )
    }

    #[test]
    fn test_protocol_issue_credential_v2() {
        test_utils::test_serde(
            Protocol::from(CredentialIssuanceTypeV2::new_v2_0()),
            json!("https://didcomm.org/issue-credential/2.0"),
        )
    }

    #[test]
    fn test_version_resolution_issue_credential_v2() {
        test_utils::test_msg_type_resolution(
            "https://didcomm.org/issue-credential/2.255",
            CredentialIssuanceTypeV2::new_v2_0(),
        )
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_issue_credential_v2() {
        test_utils::test_serde(
            Protocol::from(CredentialIssuanceTypeV2::new_v2_0()),
            json!("https://didcomm.org/issue-credential/1.0"),
        )
    }

    #[test]
    fn test_msg_type_offer_v2() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/2.0",
            "offer-credential",
            CredentialIssuanceTypeV2::new_v2_0(),
        )
    }

    #[test]
    fn test_msg_type_propose_v2() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/2.0",
            "propose-credential",
            CredentialIssuanceTypeV2::new_v2_0(),
        )
    }

    #[test]
    fn test_msg_type_request_v2() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/2.0",
            "request-credential",
            CredentialIssuanceTypeV2::new_v2_0(),
        )
    }

    #[test]
    fn test_msg_type_issue_v2() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/2.0",
            "issue-credential",
            CredentialIssuanceTypeV2::new_v2_0(),
        )
    }

    #[test]
    fn test_msg_type_preview_v2() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/2.0",
            "credential-preview",
            CredentialIssuanceTypeV2::new_v2_0(),
        )
    }

    #[test]
    fn test_msg_type_ack_v2() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/2.0",
            "ack",
            CredentialIssuanceTypeV2::new_v2_0(),
        )
    }
}
