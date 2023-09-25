// TODO - delete, this is a mock

use async_trait::async_trait;
use messages::msg_fields::protocols::cred_issuance::v2::{
    issue_credential::IssueCredentialAttachmentFormatType,
    offer_credential::OfferCredentialAttachmentFormatType, request_credential::RequestCredentialV2,
};
use shared_vcx::maybe_known::MaybeKnown;

use super::IssuerCredentialIssuanceFormat;
use crate::errors::error::VcxResult;

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

    fn get_offer_attachment_format() -> MaybeKnown<OfferCredentialAttachmentFormatType> {
        MaybeKnown::Known(OfferCredentialAttachmentFormatType::AriesLdProofVcDetail1_0)
    }
    fn get_credential_attachment_format() -> MaybeKnown<IssueCredentialAttachmentFormatType> {
        MaybeKnown::Known(IssueCredentialAttachmentFormatType::AriesLdProofVcDetail1_0)
    }

    async fn create_offer_attachment_content(
        _: &Self::CreateOfferInput,
    ) -> VcxResult<(Vec<u8>, ())> {
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
