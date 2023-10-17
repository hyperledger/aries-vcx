use crate::protocols::issuance_v2::formats::issuer::IssuerCredentialIssuanceFormat;

pub struct OfferPrepared<T: IssuerCredentialIssuanceFormat> {
    offer_metadata: T::CreatedOfferMetadata,
}

impl<T: IssuerCredentialIssuanceFormat> OfferPrepared<T> {
    pub fn new(offer_metadata: T::CreatedOfferMetadata) -> Self {
        Self { offer_metadata }
    }

    pub fn into_parts(self) -> T::CreatedOfferMetadata {
        self.offer_metadata
    }

    pub fn get_offer_metadata(&self) -> &T::CreatedOfferMetadata {
        &self.offer_metadata
    }
}
