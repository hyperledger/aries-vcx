use crate::protocols::issuance_v2::formats::holder::HolderCredentialIssuanceFormat;

pub struct CredentialReceived<T: HolderCredentialIssuanceFormat> {
    stored_credential_metadata: T::StoredCredentialMetadata,
    should_ack: bool,
}

impl<T: HolderCredentialIssuanceFormat> CredentialReceived<T> {
    pub fn new(stored_credential_metadata: T::StoredCredentialMetadata, should_ack: bool) -> Self {
        Self {
            stored_credential_metadata,
            should_ack,
        }
    }

    pub fn get_stored_credential_metadata(&self) -> &T::StoredCredentialMetadata {
        &self.stored_credential_metadata
    }

    pub fn get_should_ack(&self) -> bool {
        self.should_ack
    }
}
