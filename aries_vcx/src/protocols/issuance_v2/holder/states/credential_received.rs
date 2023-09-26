use messages::msg_fields::protocols::cred_issuance::v2::issue_credential::IssueCredentialV2;

use crate::protocols::issuance_v2::formats::holder::HolderCredentialIssuanceFormat;

pub struct CredentialReceived<T: HolderCredentialIssuanceFormat> {
    pub credential: IssueCredentialV2,
    pub stored_credential_metadata: T::StoredCredentialMetadata,
}
