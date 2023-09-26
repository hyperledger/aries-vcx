use messages::msg_fields::protocols::cred_issuance::v2::request_credential::RequestCredentialV2;

use crate::protocols::issuance_v2::formats::holder::HolderCredentialIssuanceFormat;

pub struct RequestPrepared<T: HolderCredentialIssuanceFormat> {
    pub request: RequestCredentialV2,
    pub request_preparation_metadata: T::CreatedRequestMetadata,
}
