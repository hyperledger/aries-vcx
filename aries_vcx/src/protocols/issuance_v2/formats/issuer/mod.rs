pub mod anoncreds;
pub mod ld_proof_vc;

use async_trait::async_trait;
use messages::msg_fields::protocols::cred_issuance::v2::{
    issue_credential::IssueCredentialAttachmentFormatType,
    offer_credential::OfferCredentialAttachmentFormatType, request_credential::RequestCredentialV2,
};
use shared_vcx::maybe_known::MaybeKnown;

use crate::errors::error::VcxResult;

#[async_trait]
pub trait IssuerCredentialIssuanceFormat {
    type CreateOfferInput;
    type CreatedOfferMetadata;

    type CreateCredentialInput;
    type CreatedCredentialMetadata;

    fn supports_request_independent_of_offer() -> bool;
    fn supports_multi_credential_issuance() -> bool;

    fn get_offer_attachment_format() -> MaybeKnown<OfferCredentialAttachmentFormatType>;
    fn get_credential_attachment_format() -> MaybeKnown<IssueCredentialAttachmentFormatType>;

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
