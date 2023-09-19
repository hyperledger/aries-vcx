use async_trait::async_trait;

use crate::errors::error::VcxResult;

use super::messages::{OfferCredentialV2, IssueCredentialV2};

pub mod anoncreds;

#[async_trait]
pub trait HolderCredentialIssuanceFormatHandler {
    type CreateRequestInput;
    type CreatedRequestMetadata;

    type StoreCredentialInput;
    type StoredCredentialMetadata;

    fn get_request_attachment_format() -> String;

    async fn create_request_attachment_content(
        offer_message: &OfferCredentialV2,
        data: &Self::CreateRequestInput,
    ) -> VcxResult<(Vec<u8>, Self::CreatedRequestMetadata)>;

    async fn process_and_store_credential(
        issue_credential_message: &IssueCredentialV2,
        user_input: &Self::StoreCredentialInput,
        request_metadata: Self::CreatedRequestMetadata,
    ) -> VcxResult<Self::StoredCredentialMetadata>;
}
