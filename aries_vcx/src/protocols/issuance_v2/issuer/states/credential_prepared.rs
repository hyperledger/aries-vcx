use messages::msg_fields::protocols::cred_issuance::v2::issue_credential::IssueCredentialV2;

use crate::protocols::issuance_v2::formats::issuer::IssuerCredentialIssuanceFormat;

pub struct CredentialPrepared<T: IssuerCredentialIssuanceFormat> {
    pub from_offer_metadata: Option<T::CreatedOfferMetadata>,
    pub credential_metadata: T::CreatedCredentialMetadata,
    pub credential: IssueCredentialV2,
    pub please_ack: bool,
}
