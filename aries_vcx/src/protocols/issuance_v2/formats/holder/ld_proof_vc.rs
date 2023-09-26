use async_trait::async_trait;
use messages::msg_fields::protocols::cred_issuance::v2::{
    issue_credential::{IssueCredentialAttachmentFormatType, IssueCredentialV2},
    offer_credential::{OfferCredentialAttachmentFormatType, OfferCredentialV2},
    propose_credential::ProposeCredentialAttachmentFormatType,
    request_credential::RequestCredentialAttachmentFormatType,
};
use shared_vcx::maybe_known::MaybeKnown;

use super::HolderCredentialIssuanceFormat;
use crate::errors::error::VcxResult;

// TODO - delete, this is just a mock
pub struct LdProofHolderCredentialIssuanceFormat;

#[async_trait]
impl HolderCredentialIssuanceFormat for LdProofHolderCredentialIssuanceFormat {
    type CreateProposalInput = ();

    type OfferDetails = ();

    type CreateRequestInput = ();
    type CreatedRequestMetadata = ();

    type StoreCredentialInput = ();
    type StoredCredentialMetadata = ();

    fn supports_request_independent_of_offer() -> bool {
        true
    }

    fn get_proposal_attachment_format() -> MaybeKnown<ProposeCredentialAttachmentFormatType> {
        MaybeKnown::Known(ProposeCredentialAttachmentFormatType::AriesLdProofVcDetail1_0)
    }
    fn get_request_attachment_format() -> MaybeKnown<RequestCredentialAttachmentFormatType> {
        MaybeKnown::Known(RequestCredentialAttachmentFormatType::AriesLdProofVcDetail1_0)
    }
    fn get_offer_attachment_format() -> MaybeKnown<OfferCredentialAttachmentFormatType> {
        MaybeKnown::Known(OfferCredentialAttachmentFormatType::AriesLdProofVcDetail1_0)
    }
    fn get_credential_attachment_format() -> MaybeKnown<IssueCredentialAttachmentFormatType> {
        MaybeKnown::Known(IssueCredentialAttachmentFormatType::AriesLdProofVc1_0)
    }

    async fn create_proposal_attachment_content(
        _data: &Self::CreateProposalInput,
    ) -> VcxResult<Vec<u8>> {
        Ok("mock".to_owned().into())
    }

    fn extract_offer_details(
        _: &OfferCredentialV2,
    ) -> VcxResult<Self::OfferDetails> {
        Ok(())
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
