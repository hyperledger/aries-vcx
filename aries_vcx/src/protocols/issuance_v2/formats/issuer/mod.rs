pub mod hyperledger_indy;
pub mod ld_proof_vc;

use async_trait::async_trait;
use messages::msg_fields::protocols::cred_issuance::v2::{
    issue_credential::IssueCredentialAttachmentFormatType,
    offer_credential::OfferCredentialAttachmentFormatType,
    propose_credential::{ProposeCredentialAttachmentFormatType, ProposeCredentialV2},
    request_credential::{RequestCredentialAttachmentFormatType, RequestCredentialV2},
};
use shared_vcx::maybe_known::MaybeKnown;

use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    handlers::util::{extract_attachment_data, get_attachment_with_id},
};

/// Trait representing some issue-credential-v2 format family, containing methods required by an
/// issuer of this format to create attachments of this format.
#[async_trait]
pub trait IssuerCredentialIssuanceFormat {
    type ProposalDetails;

    type CreateOfferInput;
    type CreatedOfferMetadata;

    type CreateCredentialInput;
    type CreatedCredentialMetadata;

    fn supports_request_independent_of_offer() -> bool;

    /// Retrieve the format type that an implementation uses/expects for credential proposal
    /// attachments.
    ///
    /// See formats here: https://github.com/hyperledger/aries-rfcs/blob/main/features/0453-issue-credential-v2/README.md#propose-attachment-registry
    fn get_proposal_attachment_format() -> MaybeKnown<ProposeCredentialAttachmentFormatType>;

    /// Retrieve the format type that an implementation uses/expects for credential offer
    /// attachments.
    ///
    /// See formats here: https://github.com/hyperledger/aries-rfcs/blob/main/features/0453-issue-credential-v2/README.md#offer-attachment-registry
    fn get_offer_attachment_format() -> MaybeKnown<OfferCredentialAttachmentFormatType>;

    /// Retrieve the format type that an implementation uses/expects for credential request
    /// attachments.
    ///
    /// See formats here: https://github.com/hyperledger/aries-rfcs/blob/main/features/0453-issue-credential-v2/README.md#request-attachment-registry
    fn get_request_attachment_format() -> MaybeKnown<RequestCredentialAttachmentFormatType>;

    /// Retrieve the format type that an implementation uses/expects for credential attachments.
    ///
    /// See formats here: https://github.com/hyperledger/aries-rfcs/blob/main/features/0453-issue-credential-v2/README.md#credentials-attachment-registry
    fn get_credential_attachment_format() -> MaybeKnown<IssueCredentialAttachmentFormatType>;

    fn extract_proposal_attachment_content(
        proposal_message: &ProposeCredentialV2,
    ) -> VcxResult<Vec<u8>> {
        let attachment_id = proposal_message
            .content
            .formats
            .iter()
            .find_map(|format| {
                (format.format == Self::get_proposal_attachment_format())
                    .then_some(&format.attach_id)
            })
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidMessageFormat,
                "Message does not containing an attachment with the expected format.",
            ))?;

        let attachment =
            get_attachment_with_id(&proposal_message.content.filters_attach, attachment_id)?;

        extract_attachment_data(attachment)
    }

    fn extract_proposal_details(
        proposal_message: &ProposeCredentialV2,
    ) -> VcxResult<Self::ProposalDetails>;

    async fn create_offer_attachment_content(
        data: &Self::CreateOfferInput,
    ) -> VcxResult<(Vec<u8>, Self::CreatedOfferMetadata)>;

    fn extract_request_attachment_content(
        request_message: &RequestCredentialV2,
    ) -> VcxResult<Vec<u8>> {
        let attachment_id = request_message
            .content
            .formats
            .iter()
            .find_map(|format| {
                (format.format == Self::get_request_attachment_format())
                    .then_some(&format.attach_id)
            })
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidMessageFormat,
                "Message does not containing an attachment with the expected format.",
            ))?;

        let attachment =
            get_attachment_with_id(&request_message.content.requests_attach, attachment_id)?;

        extract_attachment_data(attachment)
    }

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
