pub mod anoncreds;
pub mod ld_proof_vc;

use async_trait::async_trait;

use crate::{errors::error::VcxResult, protocols::issuance_v2::messages::RequestCredentialV2};

#[async_trait]
pub trait IssuerCredentialIssuanceFormat {
    type CreateOfferInput;
    type CreatedOfferMetadata;

    type CreateCredentialInput;
    type CreatedCredentialMetadata;

    fn supports_request_independent_of_offer() -> bool;
    fn supports_multi_credential_issuance() -> bool;

    fn get_offer_attachment_format() -> String;
    fn get_credential_attachment_format() -> String;

    async fn create_offer_attachment_content(
        data: &Self::CreateOfferInput,
    ) -> VcxResult<(Vec<u8>, Self::CreatedOfferMetadata)>;

    async fn create_credential_attachment_content(
        offer_metadata: &Self::CreatedOfferMetadata,
        request_message: &RequestCredentialV2,
        data: &Self::CreateCredentialInput,
    ) -> VcxResult<(Vec<u8>, Self::CreatedCredentialMetadata)>;

    async fn create_credential_attachment_content_independent_of_offer(
        request_message: &RequestCredentialV2,
        data: &Self::CreateCredentialInput,
    ) -> VcxResult<(Vec<u8>, Self::CreatedCredentialMetadata)>;
}
