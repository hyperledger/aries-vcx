use async_trait::async_trait;

use super::HolderCredentialIssuanceFormat;
use crate::{
    errors::error::VcxResult,
    protocols::issuance_v2::messages::{IssueCredentialV2, OfferCredentialV2},
};

// TODO - delete, this is just a mock
pub struct LdProofHolderCredentialIssuanceFormat;

#[async_trait]
impl HolderCredentialIssuanceFormat for LdProofHolderCredentialIssuanceFormat {
    type CreateProposalInput = ();

    type CreateRequestInput = ();
    type CreatedRequestMetadata = ();

    type StoreCredentialInput = ();
    type StoredCredentialMetadata = ();

    fn supports_request_independent_of_offer() -> bool {
        true
    }

    fn get_proposal_attachment_format() -> String {
        String::from("aries/ld-proof-vc-detail@v1.0")
    }
    fn get_request_attachment_format() -> String {
        String::from("aries/ld-proof-vc-detail@v1.0")
    }

    async fn create_proposal_attachment_content(
        _data: &Self::CreateProposalInput,
    ) -> VcxResult<Vec<u8>> {
        Ok("mock".to_owned().into())
    }

    async fn create_request_attachment_content(
        _offer_message: &OfferCredentialV2,
        _data: &Self::CreateRequestInput,
    ) -> VcxResult<(Vec<u8>, Self::CreatedRequestMetadata)> {
        Ok(("mock".to_owned().into(), ()))
    }

    async fn create_request_attachment_content_independent_of_offer(
        _data: &Self::CreateRequestInput,
    ) -> VcxResult<(Vec<u8>, Self::CreatedRequestMetadata)> {
        Ok(("mock".to_owned().into(), ()))
    }

    async fn process_and_store_credential(
        _issue_credential_message: &IssueCredentialV2,
        _user_input: &Self::StoreCredentialInput,
        _request_metadata: Self::CreatedRequestMetadata,
    ) -> VcxResult<Self::StoredCredentialMetadata> {
        Ok(())
    }
}
