use messages::msg_fields::protocols::cred_issuance::v2::request_credential::RequestCredentialV2;

use crate::protocols::issuance_v2::formats::issuer::IssuerCredentialIssuanceFormat;

pub struct RequestReceived<T: IssuerCredentialIssuanceFormat> {
    pub(crate) from_offer_metadata: Option<T::CreatedOfferMetadata>,
    pub(crate) request: RequestCredentialV2,
}

impl<T: IssuerCredentialIssuanceFormat> RequestReceived<T> {
    pub fn new(
        from_offer_metadata: Option<T::CreatedOfferMetadata>,
        request: RequestCredentialV2,
    ) -> Self {
        Self {
            from_offer_metadata,
            request,
        }
    }

    pub fn get_from_offer_metadata(&self) -> Option<&T::CreatedOfferMetadata> {
        self.from_offer_metadata.as_ref()
    }

    pub fn get_request(&self) -> &RequestCredentialV2 {
        &self.request
    }
}
