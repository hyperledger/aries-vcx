use messages::msg_fields::protocols::cred_issuance::v2::request_credential::RequestCredentialV2;

use crate::protocols::issuance_v2::formats::issuer::IssuerCredentialIssuanceFormat;

pub struct RequestReceived<T: IssuerCredentialIssuanceFormat> {
    pub from_offer_metadata: Option<T::CreatedOfferMetadata>,
    pub request: RequestCredentialV2,
}
