use async_trait::async_trait;
use messages::msg_fields::protocols::cred_issuance::v2::{
    issue_credential::IssueCredentialV2, offer_credential::OfferCredentialV2,
    propose_credential::ProposeCredentialAttachmentFormatType,
    request_credential::RequestCredentialAttachmentFormatType,
};
use shared_vcx::maybe_known::MaybeKnown;

use crate::errors::error::VcxResult;

pub mod anoncreds;
pub mod ld_proof_vc;

#[async_trait]
pub trait HolderCredentialIssuanceFormat {
    type CreateProposalInput;

    type CreateRequestInput;
    type CreatedRequestMetadata;

    type StoreCredentialInput;
    type StoredCredentialMetadata;

    fn supports_request_independent_of_offer() -> bool;

    fn get_proposal_attachment_format() -> MaybeKnown<ProposeCredentialAttachmentFormatType>;
    fn get_request_attachment_format() -> MaybeKnown<RequestCredentialAttachmentFormatType>;

    async fn create_proposal_attachment_content(
        data: &Self::CreateProposalInput,
    ) -> VcxResult<Vec<u8>>;

    async fn create_request_attachment_content(
        offer_message: &OfferCredentialV2,
        data: &Self::CreateRequestInput,
    ) -> VcxResult<(Vec<u8>, Self::CreatedRequestMetadata)>;

    async fn create_request_attachment_content_independent_of_offer(
        data: &Self::CreateRequestInput,
    ) -> VcxResult<(Vec<u8>, Self::CreatedRequestMetadata)>;

    async fn process_and_store_credential(
        issue_credential_message: &IssueCredentialV2,
        data: &Self::StoreCredentialInput,
        request_metadata: Self::CreatedRequestMetadata,
    ) -> VcxResult<Self::StoredCredentialMetadata>;
}
