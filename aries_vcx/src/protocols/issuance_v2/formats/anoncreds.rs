use std::{marker::PhantomData, sync::Arc};

use aries_vcx_core::{anoncreds::base_anoncreds::BaseAnonCreds, ledger::base_ledger::AnoncredsLedgerRead};
use async_trait::async_trait;

use crate::{
    errors::error::VcxResult,
    protocols::{
        issuance::holder::state_machine::{
            _parse_rev_reg_id_from_credential, create_anoncreds_credential_request, parse_cred_def_id_from_cred_offer,
        },
        issuance_v2::messages::{IssueCredentialV2, OfferCredentialV2},
    },
};

use super::CredentialIssuanceFormatHandler;

pub struct AnoncredsCredentialIssuanceFormatHandler<'a> {
    _data: &'a PhantomData<()>,
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
    credential_id: String,
}

#[async_trait]
impl<'a> CredentialIssuanceFormatHandler for AnoncredsCredentialIssuanceFormatHandler<'a> {
    type CreateRequestInput = AnoncredsCreateRequestInput<'a>;
    type CreatedRequestMetadata = AnoncredsCreatedRequestMetadata;

    type StoreCredentialInput = AnoncredsStoreCredentialInput<'a>;
    type StoredCredentialMetadata = AnoncredsStoredCredentialMetadata;

    fn get_request_attachment_format() -> String {
        String::from("anoncreds/credential-request@v1.0")
    }

    async fn create_request_attachment_content(
        offer_message: &OfferCredentialV2,
        data: &AnoncredsCreateRequestInput,
    ) -> VcxResult<(Vec<u8>, AnoncredsCreatedRequestMetadata)> {
        // extract first "anoncreds/credential-offer@v1.0" attachment from `offer_message`, or fail
        let offer_payload: String = String::from("TODO - extract from offer_message");

        let cred_def_id = parse_cred_def_id_from_cred_offer(&offer_payload)?;
        let entropy = &data.entropy;
        let ledger = data.ledger;
        let anoncreds = data.anoncreds;

        let (credential_request, credential_request_metadata, _, credential_def_json) =
            create_anoncreds_credential_request(ledger, anoncreds, &cred_def_id, &entropy, &offer_payload).await?;

        Ok((
            credential_request.into(),
            AnoncredsCreatedRequestMetadata {
                credential_request_metadata,
                credential_def_json,
            },
        ))
    }

    async fn process_and_store_credential(
        issue_credential_message: &IssueCredentialV2,
        request_metadata: AnoncredsCreatedRequestMetadata,
        user_input: &AnoncredsStoreCredentialInput,
    ) -> VcxResult<AnoncredsStoredCredentialMetadata> {
        let credential_payload: String = String::from("TODO - extract from issue_credential_message");

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

        Ok(AnoncredsStoredCredentialMetadata { credential_id: cred_id })
    }
}
