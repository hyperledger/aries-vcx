use messages::msg_fields::protocols::cred_issuance::v2::issue_credential::IssueCredentialV2;

use crate::protocols::issuance_v2::formats::holder::HolderCredentialIssuanceFormat;

pub struct CredentialReceived<T: HolderCredentialIssuanceFormat> {
    #[allow(dead_code)] // `credential` may become used in future
    pub(crate) credential: IssueCredentialV2,
    pub(crate) stored_credential_metadata: T::StoredCredentialMetadata,
}

impl<T: HolderCredentialIssuanceFormat> CredentialReceived<T> {
    pub fn new(
        credential: IssueCredentialV2,
        stored_credential_metadata: T::StoredCredentialMetadata,
    ) -> Self {
        Self {
            credential,
            stored_credential_metadata,
        }
    }

    pub fn get_credential(&self) -> &IssueCredentialV2 {
        &self.credential
    }

    pub fn get_stored_credential_metadata(&self) -> &T::StoredCredentialMetadata {
        &self.stored_credential_metadata
    }
}
