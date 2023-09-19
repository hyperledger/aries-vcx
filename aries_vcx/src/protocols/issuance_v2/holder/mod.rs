use ::messages::decorators::attachment::{Attachment, AttachmentData, AttachmentType};

use crate::errors::error::VcxResult;

use self::{
    super::messages::{IssueCredentialV2, OfferCredentialV2, ProposeCredentialV2},
    states::*,
};

use super::{formats::HolderCredentialIssuanceFormatHandler, messages::RequestCredentialV2};

mod states {
    use super::{
        super::messages::{IssueCredentialV2, OfferCredentialV2, ProposeCredentialV2, RequestCredentialV2},
        HolderCredentialIssuanceFormatHandler,
    };

    pub struct ProposalPrepared {
        pub proposal: ProposeCredentialV2,
    }

    pub struct OfferReceived {
        pub offer: OfferCredentialV2,
    }

    pub struct RequestPrepared<T: HolderCredentialIssuanceFormatHandler> {
        pub request: RequestCredentialV2,
        pub request_preparation_metadata: T::CreatedRequestMetadata,
    }

    pub struct CredentialReceived<T: HolderCredentialIssuanceFormatHandler> {
        pub credential: IssueCredentialV2,
        pub credential_received_metadata: T::StoredCredentialMetadata,
    }

    pub struct Completed;
}

pub struct HolderV2<S> {
    state: S,
    thread_id: String,
}

impl HolderV2<ProposalPrepared> {
    pub fn with_proposal(_proposal: ProposeCredentialV2) -> Self {
        todo!()
    }

    pub fn receive_offer(self, _offer: OfferCredentialV2) -> VcxResult<HolderV2<OfferReceived>> {
        // verify thread ID?
        todo!()
    }
}

impl HolderV2<OfferReceived> {
    pub fn from_offer(offer: OfferCredentialV2) -> Self {
        Self {
            state: OfferReceived { offer },
            thread_id: String::new(),
        }
    }

    pub fn propose(self, proposal: ProposeCredentialV2) -> HolderV2<ProposalPrepared> {
        HolderV2 {
            state: ProposalPrepared { proposal },
            thread_id: self.thread_id,
        }
    }

    // TODO - helper function to give consumers a clue about what format is being used

    pub async fn prepare_credential_request<T: HolderCredentialIssuanceFormatHandler>(
        self,
        input_data: &T::CreateRequestInput,
    ) -> VcxResult<HolderV2<RequestPrepared<T>>> {
        let offer_message = self.state.offer;

        let (attachment_data, output_metadata) =
            T::create_request_attachment_content(&offer_message, input_data).await?;

        let attachment_content = AttachmentType::Base64(base64::encode(&attachment_data));
        let attachment_id = uuid::Uuid::new_v4().to_string();
        let attachment = Attachment::builder()
            .id(attachment_id.clone())
            .mime_type(::messages::misc::MimeType::Json)
            .data(AttachmentData::builder().content(attachment_content).build())
            .build();

        let request_attachment_format = T::get_request_attachment_format();
        let _formats = json!([{ "attach_id": attachment_id, "format": request_attachment_format }]);
        let _attachments = vec![attachment];
        // TODO - create request message

        let request = RequestCredentialV2;

        let new_state = RequestPrepared {
            request_preparation_metadata: output_metadata,
            request,
        };

        Ok(HolderV2 {
            state: new_state,
            thread_id: self.thread_id,
        })
    }
}

impl<T: HolderCredentialIssuanceFormatHandler> HolderV2<RequestPrepared<T>> {
    pub async fn receive_credential(
        self,
        credential: IssueCredentialV2,
        input_data: &T::StoreCredentialInput,
    ) -> VcxResult<HolderV2<CredentialReceived<T>>> {
        let credential_received_metadata =
            T::process_and_store_credential(&credential, input_data, self.state.request_preparation_metadata).await?;

        let new_state = CredentialReceived {
            credential,
            credential_received_metadata,
        };
        Ok(HolderV2 {
            state: new_state,
            thread_id: self.thread_id,
        })
    }
}

impl<T: HolderCredentialIssuanceFormatHandler> HolderV2<CredentialReceived<T>> {
    // TODO - handle multi creds??
}

impl HolderV2<Completed> {}

#[cfg(test)]
pub mod demo_test {
    use crate::{
        core::profile::profile::Profile,
        protocols::issuance_v2::{
            formats::anoncreds::{
                AnoncredsCreateRequestInput, AnoncredsHolderCredentialIssuanceFormatHandler, AnoncredsStoreCredentialInput,
            },
            holder::HolderV2,
            messages::{IssueCredentialV2, OfferCredentialV2},
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
            .prepare_credential_request::<AnoncredsHolderCredentialIssuanceFormatHandler>(&prep_request_data)
            .await
            .unwrap();

        let issue_message = IssueCredentialV2;

        let receive_cred_data = AnoncredsStoreCredentialInput {
            ledger: &ledger_read,
            anoncreds: &anoncreds,
        };
        let _holder = holder
            .receive_credential(issue_message, &receive_cred_data)
            .await
            .unwrap();
    }
}
