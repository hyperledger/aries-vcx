use async_trait::async_trait;

use crate::errors::error::VcxResult;

use super::messages::{IssueCredentialV2, OfferCredentialV2};

pub mod anoncreds;

#[async_trait]
pub trait HolderCredentialIssuanceFormat {
    type CreateProposalInput;

    type CreateRequestInput;
    type CreatedRequestMetadata;

    type StoreCredentialInput;
    type StoredCredentialMetadata;
    
    fn supports_request_independent_of_offer() -> bool;
    
    fn get_proposal_attachment_format() -> String;
    fn get_request_attachment_format() -> String;


    async fn create_proposal_attachment_content(data: &Self::CreateProposalInput) -> VcxResult<Vec<u8>>;

    async fn create_request_attachment_content(
        offer_message: &OfferCredentialV2,
        data: &Self::CreateRequestInput,
    ) -> VcxResult<(Vec<u8>, Self::CreatedRequestMetadata)>;

    async fn create_request_attachment_content_independent_of_offer(
        data: &Self::CreateRequestInput
    ) -> VcxResult<(Vec<u8>, Self::CreatedRequestMetadata)>;

    async fn process_and_store_credential(
        issue_credential_message: &IssueCredentialV2,
        user_input: &Self::StoreCredentialInput,
        request_metadata: Self::CreatedRequestMetadata,
    ) -> VcxResult<Self::StoredCredentialMetadata>;
}
