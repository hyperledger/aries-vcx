use messages::msg_fields::protocols::cred_issuance::request_credential::RequestCredential;

#[derive(Debug)]
pub struct RequestPrepared {
    pub(crate) credential_request_message: RequestCredential,
    pub(crate) credential_request_metadata: String,
    pub(crate) credential_definition: String,
}

impl RequestPrepared {
    pub fn new(
        credential_request_message: RequestCredential,
        credential_request_metadata: String,
        credential_definition: String,
    ) -> Self {
        RequestPrepared {
            credential_request_message,
            credential_request_metadata,
            credential_definition,
        }
    }
}
