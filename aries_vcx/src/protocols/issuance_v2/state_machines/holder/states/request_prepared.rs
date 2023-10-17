use crate::protocols::issuance_v2::formats::holder::HolderCredentialIssuanceFormat;

pub struct RequestPrepared<T: HolderCredentialIssuanceFormat> {
    pub(crate) request_preparation_metadata: T::CreatedRequestMetadata,
}

impl<T: HolderCredentialIssuanceFormat> RequestPrepared<T> {
    pub fn new(request_preparation_metadata: T::CreatedRequestMetadata) -> Self {
        Self {
            request_preparation_metadata,
        }
    }

    pub fn get_request_preparation_metadata(&self) -> &T::CreatedRequestMetadata {
        &self.request_preparation_metadata
    }
}
