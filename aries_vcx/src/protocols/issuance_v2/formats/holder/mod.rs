use async_trait::async_trait;
use messages::msg_fields::protocols::cred_issuance::v2::{
    issue_credential::{IssueCredentialAttachmentFormatType, IssueCredentialV2},
    offer_credential::{OfferCredentialAttachmentFormatType, OfferCredentialV2},
    propose_credential::ProposeCredentialAttachmentFormatType,
    request_credential::RequestCredentialAttachmentFormatType,
};
use shared_vcx::maybe_known::MaybeKnown;

use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    handlers::util::{extract_attachment_as_base64, get_attachment_with_id},
};

pub mod hyperledger_indy;
pub mod ld_proof_vc;

/// Trait representing some issue-credential-v2 format family, containing methods required by an
/// holder of this format to create attachments of this format.
#[async_trait]
pub trait HolderCredentialIssuanceFormat {
    type CreateProposalInput;

    type OfferDetails;

    type CreateRequestInput;
    type CreatedRequestMetadata;

    type StoreCredentialInput;
    type StoredCredentialMetadata;

    fn supports_request_independent_of_offer() -> bool;

    fn get_proposal_attachment_format() -> MaybeKnown<ProposeCredentialAttachmentFormatType>;
    fn get_offer_attachment_format() -> MaybeKnown<OfferCredentialAttachmentFormatType>;
    fn get_request_attachment_format() -> MaybeKnown<RequestCredentialAttachmentFormatType>;
    fn get_credential_attachment_format() -> MaybeKnown<IssueCredentialAttachmentFormatType>;

    async fn create_proposal_attachment_content(
        data: &Self::CreateProposalInput,
    ) -> VcxResult<Vec<u8>>;

    fn extract_offer_attachment_content(offer_message: &OfferCredentialV2) -> VcxResult<Vec<u8>> {
        let attachment_id = offer_message
            .content
            .formats
            .iter()
            .find_map(|format| {
                (format.format == Self::get_offer_attachment_format()).then_some(&format.attach_id)
            })
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidMessageFormat,
                "Message does not containing an attachment with the expected format.",
            ))?;

        let attachment =
            get_attachment_with_id(&offer_message.content.offers_attach, attachment_id)?;

        extract_attachment_as_base64(attachment)
    }

    fn extract_offer_details(offer_message: &OfferCredentialV2) -> VcxResult<Self::OfferDetails>;

    async fn create_request_attachment_content(
        offer_message: &OfferCredentialV2,
        data: &Self::CreateRequestInput,
    ) -> VcxResult<(Vec<u8>, Self::CreatedRequestMetadata)>;

    async fn create_request_attachment_content_independent_of_offer(
        data: &Self::CreateRequestInput,
    ) -> VcxResult<(Vec<u8>, Self::CreatedRequestMetadata)>;

    fn extract_credential_attachment_content(
        issue_credential_message: &IssueCredentialV2,
    ) -> VcxResult<Vec<u8>> {
        let attachment_id = issue_credential_message
            .content
            .formats
            .iter()
            .find_map(|format| {
                (format.format == Self::get_credential_attachment_format())
                    .then_some(&format.attach_id)
            })
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidMessageFormat,
                "Message does not containing an attachment with the expected format.",
            ))?;

        let attachment = get_attachment_with_id(
            &issue_credential_message.content.credentials_attach,
            attachment_id,
        )?;

        extract_attachment_as_base64(attachment)
    }

    async fn process_and_store_credential(
        issue_credential_message: &IssueCredentialV2,
        data: &Self::StoreCredentialInput,
        request_metadata: &Self::CreatedRequestMetadata,
    ) -> VcxResult<Self::StoredCredentialMetadata>;
}

#[cfg(test)]
pub(crate) mod mocks {
    use async_trait::async_trait;
    use messages::msg_fields::protocols::cred_issuance::v2::{
        issue_credential::{IssueCredentialAttachmentFormatType, IssueCredentialV2},
        offer_credential::{OfferCredentialAttachmentFormatType, OfferCredentialV2},
        propose_credential::ProposeCredentialAttachmentFormatType,
        request_credential::RequestCredentialAttachmentFormatType,
    };
    use mockall::mock;
    use shared_vcx::maybe_known::MaybeKnown;

    use super::HolderCredentialIssuanceFormat;
    use crate::errors::error::VcxResult;

    mock! {
        pub HolderCredentialIssuanceFormat {}
        #[async_trait]
        impl HolderCredentialIssuanceFormat for HolderCredentialIssuanceFormat {
            type CreateProposalInput = String;

            type OfferDetails = String;

            type CreateRequestInput = String;
            type CreatedRequestMetadata = String;

            type StoreCredentialInput = String;
            type StoredCredentialMetadata = String;

            fn supports_request_independent_of_offer() -> bool;

            fn get_proposal_attachment_format() -> MaybeKnown<ProposeCredentialAttachmentFormatType>;
            fn get_offer_attachment_format() -> MaybeKnown<OfferCredentialAttachmentFormatType>;
            fn get_request_attachment_format() -> MaybeKnown<RequestCredentialAttachmentFormatType>;
            fn get_credential_attachment_format() -> MaybeKnown<IssueCredentialAttachmentFormatType>;

            async fn create_proposal_attachment_content(
                data: &String,
            ) -> VcxResult<Vec<u8>>;

            fn extract_offer_details(
                offer_message: &OfferCredentialV2,
            ) -> VcxResult<String>;

            async fn create_request_attachment_content(
                offer_message: &OfferCredentialV2,
                data: &String,
            ) -> VcxResult<(Vec<u8>, String)>;

            async fn create_request_attachment_content_independent_of_offer(
                data: &String,
            ) -> VcxResult<(Vec<u8>, String)>;

            async fn process_and_store_credential(
                issue_credential_message: &IssueCredentialV2,
                data: &String,
                request_metadata: &String,
            ) -> VcxResult<String>;
        }
    }
}
