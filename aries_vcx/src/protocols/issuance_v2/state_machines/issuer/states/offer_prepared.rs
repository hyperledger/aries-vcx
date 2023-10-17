use messages::msg_fields::protocols::cred_issuance::v2::offer_credential::OfferCredentialV2;

use crate::protocols::issuance_v2::formats::issuer::IssuerCredentialIssuanceFormat;

pub struct OfferPrepared<T: IssuerCredentialIssuanceFormat> {
    pub(crate) offer_metadata: T::CreatedOfferMetadata,
    pub(crate) offer: OfferCredentialV2,
}

impl<T: IssuerCredentialIssuanceFormat> OfferPrepared<T> {
    pub fn new(offer_metadata: T::CreatedOfferMetadata, offer: OfferCredentialV2) -> Self {
        Self {
            offer_metadata,
            offer,
        }
    }

    pub fn get_offer_metadata(&self) -> &T::CreatedOfferMetadata {
        &self.offer_metadata
    }

    pub fn get_offer(&self) -> &OfferCredentialV2 {
        &self.offer
    }
}
