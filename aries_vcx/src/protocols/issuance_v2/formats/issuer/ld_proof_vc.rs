// TODO - delete, this is a mock

use async_trait::async_trait;

use crate::{errors::error::VcxResult, protocols::issuance_v2::messages::RequestCredentialV2};

use super::IssuerCredentialIssuanceFormat;

pub struct LdProofIssuerCredentialIssuanceFormat;

#[async_trait]
impl IssuerCredentialIssuanceFormat for LdProofIssuerCredentialIssuanceFormat {
    type CreateOfferInput = ();
    type CreatedOfferMetadata = ();

    type CreateCredentialInput = ();
    type CreatedCredentialMetadata = ();

    fn supports_request_independent_of_offer() -> bool {
        true
    }
    fn supports_multi_credential_issuance() -> bool {
        true
    }

    fn get_offer_attachment_format() -> String {
        String::from("aries/ld-proof-vc-detail@v1.0")
    }
    fn get_credential_attachment_format() -> String {
        String::from("aries/ld-proof-vc@v1.0")
    }

    async fn create_offer_attachment_content(_: &Self::CreateOfferInput) -> VcxResult<(Vec<u8>, ())> {
        Ok(("mock data".into(), ()))
    }

    async fn create_credential_attachment_content(
        _offer_metadata: &(),
        _request_message: &RequestCredentialV2,
        _data: &Self::CreateCredentialInput,
    ) -> VcxResult<(Vec<u8>, ())> {
        Ok(("mock data".into(), ()))
    }

    async fn create_credential_attachment_content_independent_of_offer(
        _request_message: &RequestCredentialV2,
        _data: &Self::CreateCredentialInput,
    ) -> VcxResult<(Vec<u8>, ())> {
        Ok(("mock data".into(), ()))
    }
}
