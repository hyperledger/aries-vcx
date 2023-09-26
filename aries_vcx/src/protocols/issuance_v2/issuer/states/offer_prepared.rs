use messages::msg_fields::protocols::cred_issuance::v2::offer_credential::OfferCredentialV2;

use crate::protocols::issuance_v2::formats::issuer::IssuerCredentialIssuanceFormat;

pub struct OfferPrepared<T: IssuerCredentialIssuanceFormat> {
    pub offer_metadata: T::CreatedOfferMetadata,
    pub offer: OfferCredentialV2,
}
