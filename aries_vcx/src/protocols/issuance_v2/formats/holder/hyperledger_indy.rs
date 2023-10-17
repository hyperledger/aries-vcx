use std::marker::PhantomData;

use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds, ledger::base_ledger::AnoncredsLedgerRead,
};
use async_trait::async_trait;
use messages::msg_fields::protocols::cred_issuance::v2::{
    issue_credential::{IssueCredentialAttachmentFormatType, IssueCredentialV2},
    offer_credential::{OfferCredentialAttachmentFormatType, OfferCredentialV2},
    propose_credential::ProposeCredentialAttachmentFormatType,
    request_credential::RequestCredentialAttachmentFormatType,
};
use shared_vcx::maybe_known::MaybeKnown;

use super::HolderCredentialIssuanceFormat;
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    protocols::issuance::holder::state_machine::{
        _parse_rev_reg_id_from_credential, create_anoncreds_credential_request,
        parse_cred_def_id_from_cred_offer,
    },
};

/// Structure which implements [HolderCredentialIssuanceFormat] functionality for the `hlindy/...`
/// family of issue-credential-v2 attachment formats.
///
/// This implementation expects and creates attachments of the following types:
/// * [ProposeCredentialAttachmentFormatType::HyperledgerIndyCredentialFilter2_0]
/// * [RequestCredentialAttachmentFormatType::HyperledgerIndyCredentialRequest2_0]
/// * [OfferCredentialAttachmentFormatType::HyperledgerIndyCredentialAbstract2_0]
/// * [IssueCredentialAttachmentFormatType::HyperledgerIndyCredential2_0]
///
/// This is done in accordance to the Aries RFC 0592 Spec:
///
/// https://github.com/hyperledger/aries-rfcs/blob/b3a3942ef052039e73cd23d847f42947f8287da2/features/0592-indy-attachments/README.md
pub struct HyperledgerIndyHolderCredentialIssuanceFormat<'a, R, A>
where
    R: AnoncredsLedgerRead,
    A: BaseAnonCreds,
{
    _data: &'a PhantomData<()>,
    _ledger_read: PhantomData<R>,
    _anoncreds: PhantomData<A>,
}

pub struct HyperledgerIndyCreateProposalInput {
    pub cred_filter: HyperledgerIndyCredentialFilter,
}

#[derive(Default, Clone, PartialEq, Debug, Serialize, Deserialize, Builder)]
#[builder(setter(into, strip_option), default)]
pub struct HyperledgerIndyCredentialFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_issuer_did: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer_did: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cred_def_id: Option<String>,
}

// Simplified cred abstract, for purpose of easy viewing for consumer
// https://github.com/hyperledger/aries-rfcs/blob/main/features/0592-indy-attachments/README.md#cred-abstract-format
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HyperledgerIndyOfferDetails {
    pub schema_id: String,
    pub cred_def_id: String,
}

pub struct HyperledgerIndyCreateRequestInput<'a, R, A>
where
    R: AnoncredsLedgerRead,
    A: BaseAnonCreds,
{
    pub entropy_did: String,
    pub ledger: &'a R,
    pub anoncreds: &'a A,
}

#[derive(Clone, Debug)]
pub struct HyperledgerIndyCreatedRequestMetadata {
    credential_request_metadata: String,
    credential_def_json: String,
}

pub struct HyperledgerIndyStoreCredentialInput<'a, R, A>
where
    R: AnoncredsLedgerRead,
    A: BaseAnonCreds,
{
    pub ledger: &'a R,
    pub anoncreds: &'a A,
}

#[derive(Clone, Debug)]
pub struct HyperledgerIndyStoredCredentialMetadata {
    pub credential_id: String,
}

#[async_trait]
impl<'a, R, A> HolderCredentialIssuanceFormat
    for HyperledgerIndyHolderCredentialIssuanceFormat<'a, R, A>
where
    R: AnoncredsLedgerRead + 'a,
    A: BaseAnonCreds + 'a,
{
    type CreateProposalInput = HyperledgerIndyCreateProposalInput;

    type OfferDetails = HyperledgerIndyOfferDetails;

    type CreateRequestInput = HyperledgerIndyCreateRequestInput<'a, R, A>;
    type CreatedRequestMetadata = HyperledgerIndyCreatedRequestMetadata;

    type StoreCredentialInput = HyperledgerIndyStoreCredentialInput<'a, R, A>;
    type StoredCredentialMetadata = HyperledgerIndyStoredCredentialMetadata;

    fn supports_request_independent_of_offer() -> bool {
        false
    }

    fn get_proposal_attachment_format() -> MaybeKnown<ProposeCredentialAttachmentFormatType> {
        MaybeKnown::Known(ProposeCredentialAttachmentFormatType::HyperledgerIndyCredentialFilter2_0)
    }
    fn get_request_attachment_format() -> MaybeKnown<RequestCredentialAttachmentFormatType> {
        MaybeKnown::Known(
            RequestCredentialAttachmentFormatType::HyperledgerIndyCredentialRequest2_0,
        )
    }
    fn get_offer_attachment_format() -> MaybeKnown<OfferCredentialAttachmentFormatType> {
        MaybeKnown::Known(OfferCredentialAttachmentFormatType::HyperledgerIndyCredentialAbstract2_0)
    }
    fn get_credential_attachment_format() -> MaybeKnown<IssueCredentialAttachmentFormatType> {
        MaybeKnown::Known(IssueCredentialAttachmentFormatType::HyperledgerIndyCredential2_0)
    }

    async fn create_proposal_attachment_content(
        data: &HyperledgerIndyCreateProposalInput,
    ) -> VcxResult<Vec<u8>> {
        let filter_bytes = serde_json::to_vec(&data.cred_filter)?;

        Ok(filter_bytes)
    }

    fn extract_offer_details(
        offer_message: &OfferCredentialV2,
    ) -> VcxResult<HyperledgerIndyOfferDetails> {
        let attachment = Self::extract_offer_attachment_content(offer_message)?;

        Ok(serde_json::from_slice(&attachment)?)
    }

    async fn create_request_attachment_content(
        offer_message: &OfferCredentialV2,
        data: &Self::CreateRequestInput,
    ) -> VcxResult<(Vec<u8>, HyperledgerIndyCreatedRequestMetadata)> {
        let offer_bytes = Self::extract_offer_attachment_content(&offer_message)?;
        let offer_payload = String::from_utf8(offer_bytes).map_err(|_| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::EncodeError,
                "Expected payload to be a utf8 string",
            )
        })?;

        let cred_def_id = parse_cred_def_id_from_cred_offer(&offer_payload)?;
        let entropy = &data.entropy_did;
        let ledger = data.ledger;
        let anoncreds = data.anoncreds;

        let (credential_request, credential_request_metadata, _, credential_def_json) =
            create_anoncreds_credential_request(
                ledger,
                anoncreds,
                &cred_def_id,
                &entropy,
                &offer_payload,
            )
            .await?;

        Ok((
            credential_request.into(),
            HyperledgerIndyCreatedRequestMetadata {
                credential_request_metadata,
                credential_def_json,
            },
        ))
    }

    async fn create_request_attachment_content_independent_of_offer(
        _: &Self::CreateRequestInput,
    ) -> VcxResult<(Vec<u8>, Self::CreatedRequestMetadata)> {
        Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::ActionNotSupported,
            "Anoncreds cannot create request payload independent of an offer",
        ))
    }

    async fn process_and_store_credential(
        issue_credential_message: &IssueCredentialV2,
        user_input: &HyperledgerIndyStoreCredentialInput<R, A>,
        request_metadata: &HyperledgerIndyCreatedRequestMetadata,
    ) -> VcxResult<HyperledgerIndyStoredCredentialMetadata> {
        let cred_bytes = Self::extract_credential_attachment_content(&issue_credential_message)?;
        let credential_payload = String::from_utf8(cred_bytes).map_err(|_| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::EncodeError,
                "Expected payload to be a utf8 string",
            )
        })?;

        let ledger = user_input.ledger;
        let anoncreds = user_input.anoncreds;

        let rev_reg_id = _parse_rev_reg_id_from_credential(&credential_payload)?;
        let rev_reg_def_json = if let Some(rev_reg_id) = rev_reg_id {
            let json = ledger.get_rev_reg_def_json(&rev_reg_id).await?;
            Some(json)
        } else {
            None
        };

        let cred_id = anoncreds
            .prover_store_credential(
                None,
                &request_metadata.credential_request_metadata,
                &credential_payload,
                &request_metadata.credential_def_json,
                rev_reg_def_json.as_deref(),
            )
            .await?;

        Ok(HyperledgerIndyStoredCredentialMetadata {
            credential_id: cred_id,
        })
    }
}
