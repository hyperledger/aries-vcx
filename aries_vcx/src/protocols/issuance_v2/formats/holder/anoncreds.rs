use std::{marker::PhantomData, sync::Arc};

use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds, ledger::base_ledger::AnoncredsLedgerRead,
};
use async_trait::async_trait;
use messages::msg_fields::protocols::cred_issuance::v2::{
    issue_credential::IssueCredentialV2, offer_credential::OfferCredentialV2,
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

// TODO - rebrand this all to "hyperledger" handler
pub struct AnoncredsHolderCredentialIssuanceFormat<'a> {
    _data: &'a PhantomData<()>,
}

pub struct AnoncredsCreateProposalInput {
    pub cred_filter: AnoncredsCredentialFilter,
}

// TODO - rebrand this all to "hyperledger" handler
#[derive(Default, Serialize, Deserialize)]
pub struct AnoncredsCredentialFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_issuer_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cred_def_id: Option<String>,
}

pub struct AnoncredsCreateRequestInput<'a> {
    pub entropy: String,
    pub ledger: &'a Arc<dyn AnoncredsLedgerRead>,
    pub anoncreds: &'a Arc<dyn BaseAnonCreds>,
}

pub struct AnoncredsCreatedRequestMetadata {
    credential_request_metadata: String,
    credential_def_json: String,
}

pub struct AnoncredsStoreCredentialInput<'a> {
    pub ledger: &'a Arc<dyn AnoncredsLedgerRead>,
    pub anoncreds: &'a Arc<dyn BaseAnonCreds>,
}

pub struct AnoncredsStoredCredentialMetadata {
    pub credential_id: String,
}

#[async_trait]
impl<'a> HolderCredentialIssuanceFormat for AnoncredsHolderCredentialIssuanceFormat<'a> {
    type CreateProposalInput = AnoncredsCreateProposalInput;

    type CreateRequestInput = AnoncredsCreateRequestInput<'a>;
    type CreatedRequestMetadata = AnoncredsCreatedRequestMetadata;

    type StoreCredentialInput = AnoncredsStoreCredentialInput<'a>;
    type StoredCredentialMetadata = AnoncredsStoredCredentialMetadata;

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

    async fn create_proposal_attachment_content(
        data: &AnoncredsCreateProposalInput,
    ) -> VcxResult<Vec<u8>> {
        let filter_bytes = serde_json::to_vec(&data.cred_filter)?;

        Ok(filter_bytes)
    }

    async fn create_request_attachment_content(
        offer_message: &OfferCredentialV2,
        data: &AnoncredsCreateRequestInput,
    ) -> VcxResult<(Vec<u8>, AnoncredsCreatedRequestMetadata)> {
        // extract first "anoncreds/credential-offer@v1.0" attachment from `offer_message`, or fail
        _ = offer_message;
        let offer_payload: String = String::from("TODO - extract from offer_message");

        let cred_def_id = parse_cred_def_id_from_cred_offer(&offer_payload)?;
        let entropy = &data.entropy;
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
            AnoncredsCreatedRequestMetadata {
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
        user_input: &AnoncredsStoreCredentialInput,
        request_metadata: AnoncredsCreatedRequestMetadata,
    ) -> VcxResult<AnoncredsStoredCredentialMetadata> {
        _ = issue_credential_message;
        let credential_payload: String =
            String::from("TODO - extract from issue_credential_message");

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

        Ok(AnoncredsStoredCredentialMetadata {
            credential_id: cred_id,
        })
    }
}
