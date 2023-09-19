use std::{marker::PhantomData, sync::Arc};

use ::messages::decorators::attachment::{Attachment, AttachmentData, AttachmentType};
use aries_vcx_core::{anoncreds::base_anoncreds::BaseAnonCreds, ledger::base_ledger::AnoncredsLedgerRead};
use async_trait::async_trait;

use crate::{
    errors::error::VcxResult,
    protocols::issuance::holder::state_machine::{
        _parse_rev_reg_id_from_credential, create_anoncreds_credential_request, parse_cred_def_id_from_cred_offer,
    },
};

use self::{
    messages::{IssueCredentialV2, OfferCredentialV2, ProposeCredentialV2},
    states::*,
};

mod messages {
    pub struct ProposeCredentialV2;
    pub struct OfferCredentialV2;
    pub struct RequestCredentialV2;
    pub struct IssueCredentialV2;
}

mod states {
    use super::{messages::OfferCredentialV2, CredentialIssuanceFormatHandler};

    pub struct ProposalPrepared;
    pub struct OfferReceived {
        pub offer: OfferCredentialV2,
    }
    pub struct RequestPrepared<T: CredentialIssuanceFormatHandler> {
        pub request_preparation_metadata: T::CreatedRequestMetadata,
    }
    pub struct CredentialReceived;
    pub struct Completed;
}

pub struct HolderV2<S> {
    state: S,
    thread_id: String,
}

impl HolderV2<ProposalPrepared> {
    pub fn with_proposal(proposal: ProposeCredentialV2) -> Self {
        todo!()
    }

    pub fn receive_offer(self, offer: OfferCredentialV2) -> VcxResult<HolderV2<OfferReceived>> {
        // verify thread ID?
        todo!()
    }
}

impl HolderV2<OfferReceived> {
    pub fn from_offer(offer: OfferCredentialV2) -> Self {
        todo!()
    }

    pub async fn prepare_credential_request<T: CredentialIssuanceFormatHandler>(
        self,
        format_data: &T::CreateRequestInput,
    ) -> VcxResult<HolderV2<RequestPrepared<T>>> {
        let offer_message = self.state.offer;

        let (attachment_data, output_metadata) =
            T::create_request_attachment_content(&offer_message, format_data).await?;
        let attachment_content = AttachmentType::Base64(base64::encode(&attachment_data));
        let attachment_id = uuid::Uuid::new_v4().to_string();
        let attachment = Attachment::builder()
            .id(attachment_id)
            .mime_type(::messages::misc::MimeType::Json)
            .data(AttachmentData::builder().content(attachment_content).build())
            .build();

        let request_attachment_format = T::get_request_attachment_format();

        // create formats array, of { attach_id: attachment_id, format: request_attachment_format }

        let new_state = RequestPrepared {
            request_preparation_metadata: output_metadata,
        };

        Ok(HolderV2 {
            state: new_state,
            thread_id: String::new(),
        })
    }
}

impl<T: CredentialIssuanceFormatHandler> HolderV2<RequestPrepared<T>> {
    pub async fn receive_credential(
        self,
        credential: IssueCredentialV2,
        format_data: &T::StoreCredentialInput,
    ) -> VcxResult<HolderV2<CredentialReceived>> {
        let res = T::process_and_store_credential(&credential, self.state.request_preparation_metadata, format_data)
            .await
            .unwrap();
        todo!()
    }
}

impl HolderV2<CredentialReceived> {}

impl HolderV2<Completed> {}

#[async_trait]
pub trait CredentialIssuanceFormatHandler {
    type CreateRequestInput;
    type CreatedRequestMetadata;

    type StoreCredentialInput;

    fn get_request_attachment_format() -> String;
    async fn create_request_attachment_content(
        offer_message: &OfferCredentialV2,
        data: &Self::CreateRequestInput,
    ) -> VcxResult<(Vec<u8>, Self::CreatedRequestMetadata)>;

    async fn process_and_store_credential(
        issue_credential_message: &IssueCredentialV2,
        request_metadata: Self::CreatedRequestMetadata,
        user_input: &Self::StoreCredentialInput,
    ) -> VcxResult<()>;
}

pub struct AnoncredsCredentialIssuanceFormatHandler<'a> {
    _data: &'a PhantomData<()>,
}

pub struct AnoncredsCreateRequestInput<'a> {
    entropy: String,
    ledger: &'a Arc<dyn AnoncredsLedgerRead>,
    anoncreds: &'a Arc<dyn BaseAnonCreds>,
}

pub struct AnoncredsCreatedRequestMetadata {
    credential_request_metadata: String,
    credential_def_json: String,
}

pub struct AnoncredsStoreCredentialInput<'a> {
    ledger: &'a Arc<dyn AnoncredsLedgerRead>,
    anoncreds: &'a Arc<dyn BaseAnonCreds>,
}

#[async_trait]
impl<'a> CredentialIssuanceFormatHandler for AnoncredsCredentialIssuanceFormatHandler<'a> {
    type CreateRequestInput = AnoncredsCreateRequestInput<'a>;
    type CreatedRequestMetadata = AnoncredsCreatedRequestMetadata;

    type StoreCredentialInput = AnoncredsStoreCredentialInput<'a>;

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
    ) -> VcxResult<()> {
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

        todo!()
    }
}

#[cfg(test)]
pub mod demo_test {
    use crate::{
        core::profile::profile::Profile,
        protocols::issuance_v2::holder::{
            messages::{IssueCredentialV2, OfferCredentialV2},
            AnoncredsCreateRequestInput, AnoncredsCredentialIssuanceFormatHandler, AnoncredsStoreCredentialInput,
            HolderV2,
        },
        utils::mockdata::profile::mock_profile::MockProfile,
    };

    #[tokio::test]
    async fn demo_test() {
        let profile = MockProfile;
        let anoncreds = profile.inject_anoncreds();
        let ledger_read = profile.inject_anoncreds_ledger_read();

        let offer_message = OfferCredentialV2;

        let holder = HolderV2::from_offer(offer_message);

        let prep_request_data = AnoncredsCreateRequestInput {
            entropy: String::from("blah-blah-blah"),
            ledger: &ledger_read,
            anoncreds: &anoncreds,
        };
        let holder = holder
            .prepare_credential_request::<AnoncredsCredentialIssuanceFormatHandler>(&prep_request_data)
            .await
            .unwrap();

        let issue_message = IssueCredentialV2;

        let receive_cred_data = AnoncredsStoreCredentialInput {
            ledger: &ledger_read,
            anoncreds: &anoncreds,
        };
        let holder = holder
            .receive_credential(issue_message, &receive_cred_data)
            .await
            .unwrap();
    }
}
