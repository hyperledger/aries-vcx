use messages::msg_fields::protocols::cred_issuance::v2::request_credential::RequestCredentialV2;

use crate::protocols::issuance_v2::formats::holder::HolderCredentialIssuanceFormat;

pub struct RequestPrepared<T: HolderCredentialIssuanceFormat> {
    pub(crate) request: RequestCredentialV2,
    pub(crate) request_preparation_metadata: T::CreatedRequestMetadata,
}

impl<T: HolderCredentialIssuanceFormat> RequestPrepared<T> {
    pub fn new(
        request: RequestCredentialV2,
        request_preparation_metadata: T::CreatedRequestMetadata,
    ) -> Self {
        Self {
            request,
            request_preparation_metadata,
        }
    }

    pub fn get_request(&self) -> &RequestCredentialV2 {
        &self.request
    }

    pub fn get_request_preparation_metadata(&self) -> &T::CreatedRequestMetadata {
        &self.request_preparation_metadata
    }
}
