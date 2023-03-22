use std::marker::PhantomData;

use derive_more::{From, TryInto};
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use super::Protocol;
use crate::msg_types::role::Role;

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, MessageType)]
#[msg_type(protocol = "issue-credential")]
pub enum CredentialIssuance {
    V1(CredentialIssuanceV1),
}

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(CredentialIssuance, Protocol))]
#[msg_type(major = 1)]
pub enum CredentialIssuanceV1 {
    #[msg_type(minor = 0, roles = "Role::Holder, Role::Issuer")]
    V1_0(PhantomData<fn() -> CredentialIssuanceV1_0>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum CredentialIssuanceV1_0 {
    OfferCredential,
    ProposeCredential,
    RequestCredential,
    IssueCredential,
    CredentialPreview,
    Ack,
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::CredentialIssuanceV1;
    use crate::misc::test_utils;

    const PROTOCOL: &str = "https://didcomm.org/issue-credential/1.0";
    const VERSION_RESOLUTION_PROTOCOL: &str = "https://didcomm.org/issue-credential/1.255";
    const UNSUPPORTED_VERSION_PROTOCOL: &str = "https://didcomm.org/issue-credential/2.0";

    const KIND_OFFER: &str = "offer-credential";
    const KIND_PROPOSE: &str = "propose-credential";
    const KIND_REQUEST: &str = "request-credential";
    const KIND_ISSUE: &str = "issue-credential";
    const KIND_PREVIEW: &str = "credential-preview";
    const KIND_ACK: &str = "ack";

    #[test]
    fn test_protocol_issue_credential() {
        test_utils::test_protocol(PROTOCOL, CredentialIssuanceV1::V1_0(PhantomData))
    }

    #[test]
    fn test_version_resolution_issue_credential() {
        test_utils::test_protocol(VERSION_RESOLUTION_PROTOCOL, CredentialIssuanceV1::V1_0(PhantomData))
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_issue_credential() {
        test_utils::test_protocol(UNSUPPORTED_VERSION_PROTOCOL, CredentialIssuanceV1::V1_0(PhantomData))
    }

    #[test]
    fn test_msg_type_offer() {
        test_utils::test_msg_type(PROTOCOL, KIND_OFFER, CredentialIssuanceV1::V1_0(PhantomData))
    }

    #[test]
    fn test_msg_type_propose() {
        test_utils::test_msg_type(PROTOCOL, KIND_PROPOSE, CredentialIssuanceV1::V1_0(PhantomData))
    }

    #[test]
    fn test_msg_type_request() {
        test_utils::test_msg_type(PROTOCOL, KIND_REQUEST, CredentialIssuanceV1::V1_0(PhantomData))
    }

    #[test]
    fn test_msg_type_issue() {
        test_utils::test_msg_type(PROTOCOL, KIND_ISSUE, CredentialIssuanceV1::V1_0(PhantomData))
    }

    #[test]
    fn test_msg_type_preview() {
        test_utils::test_msg_type(PROTOCOL, KIND_PREVIEW, CredentialIssuanceV1::V1_0(PhantomData))
    }

    #[test]
    fn test_msg_type_ack() {
        test_utils::test_msg_type(PROTOCOL, KIND_ACK, CredentialIssuanceV1::V1_0(PhantomData))
    }
}
