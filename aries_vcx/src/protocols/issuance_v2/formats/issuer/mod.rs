pub mod anoncreds;

use async_trait::async_trait;

use crate::{
    errors::error::VcxResult,
    protocols::issuance_v2::messages::{OfferCredentialV2, RequestCredentialV2},
};

#[async_trait]
pub trait IssuerCredentialIssuanceFormat {
    type CreateOfferInput;

    type CreateCredentialInput;

    fn supports_request_independent_of_offer() -> bool;

    fn get_offer_attachment_format() -> String;
    fn get_credential_attachment_format() -> String;

    async fn create_offer_attachment_content(data: &Self::CreateOfferInput) -> VcxResult<Vec<u8>>;

    async fn create_credential_attachment_content(
        offer_message: &OfferCredentialV2,
        request_message: &RequestCredentialV2,
        data: &Self::CreateCredentialInput,
    ) -> VcxResult<Vec<u8>>;

    async fn create_credential_attachment_content_independent_of_offer(
        request_message: &RequestCredentialV2,
        data: &Self::CreateCredentialInput,
    ) -> VcxResult<Vec<u8>>;
}
