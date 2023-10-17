use messages::msg_fields::protocols::cred_issuance::v2::issue_credential::IssueCredentialV2;

use crate::protocols::issuance_v2::formats::issuer::IssuerCredentialIssuanceFormat;

pub struct CredentialPrepared<T: IssuerCredentialIssuanceFormat> {
    pub(crate) from_offer_metadata: Option<T::CreatedOfferMetadata>,
    pub(crate) credential_metadata: T::CreatedCredentialMetadata,
    pub(crate) credential: IssueCredentialV2,
    pub(crate) please_ack: bool,
}

impl<T: IssuerCredentialIssuanceFormat> CredentialPrepared<T> {
    pub fn new(
        from_offer_metadata: Option<T::CreatedOfferMetadata>,
        credential_metadata: T::CreatedCredentialMetadata,
        credential: IssueCredentialV2,
        please_ack: bool,
    ) -> Self {
        Self {
            from_offer_metadata,
            credential_metadata,
            credential,
            please_ack,
        }
    }

    pub fn get_from_offer_metadata(&self) -> Option<&T::CreatedOfferMetadata> {
        self.from_offer_metadata.as_ref()
    }

    pub fn get_credential_metadata(&self) -> &T::CreatedCredentialMetadata {
        &self.credential_metadata
    }

    pub fn get_credential(&self) -> &IssueCredentialV2 {
        &self.credential
    }

    pub fn get_please_ack(&self) -> bool {
        self.please_ack
    }
}
