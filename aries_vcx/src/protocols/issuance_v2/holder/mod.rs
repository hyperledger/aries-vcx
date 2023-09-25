use std::marker::PhantomData;

use ::messages::decorators::attachment::{Attachment, AttachmentData, AttachmentType};
use messages::{
    decorators::thread::Thread,
    misc::MimeType,
    msg_fields::protocols::{
        cred_issuance::v2::{
            issue_credential::IssueCredentialV2,
            offer_credential::OfferCredentialV2,
            propose_credential::{
                ProposeCredentialV2, ProposeCredentialV2Content, ProposeCredentialV2Decorators,
            },
            request_credential::{
                RequestCredentialV2, RequestCredentialV2Content, RequestCredentialV2Decorators,
            },
            AttachmentFormatSpecifier, CredentialPreviewV2,
        },
        notification::ack::{Ack, AckContent, AckDecorators, AckStatus},
    },
};
use uuid::Uuid;

use self::states::*;
use super::{
    formats::holder::HolderCredentialIssuanceFormat, RecoveredSMError, VcxSMTransitionResult,
};
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    handlers::util::get_thread_id_or_message_id,
};

pub mod states {
    use std::marker::PhantomData;

    use messages::msg_fields::protocols::{
        cred_issuance::v2::{
            issue_credential::IssueCredentialV2, offer_credential::OfferCredentialV2,
            propose_credential::ProposeCredentialV2, request_credential::RequestCredentialV2,
        },
        notification::ack::Ack,
        report_problem::ProblemReport,
    };

    use super::HolderCredentialIssuanceFormat;

    pub struct ProposalPrepared<T: HolderCredentialIssuanceFormat> {
        pub proposal: ProposeCredentialV2,
        pub _marker: PhantomData<T>,
    }

    pub struct OfferReceived<T: HolderCredentialIssuanceFormat> {
        pub offer: OfferCredentialV2,
        pub _marker: PhantomData<T>,
    }

    pub struct RequestPrepared<T: HolderCredentialIssuanceFormat> {
        pub request: RequestCredentialV2,
        pub request_preparation_metadata: T::CreatedRequestMetadata,
    }

    pub struct CredentialReceived<T: HolderCredentialIssuanceFormat> {
        pub credential: IssueCredentialV2,
        pub credential_received_metadata: T::StoredCredentialMetadata,
    }

    pub struct Complete<T: HolderCredentialIssuanceFormat> {
        pub ack: Option<Ack>,
        pub _marker: PhantomData<T>,
    }

    pub struct Failed<T: HolderCredentialIssuanceFormat> {
        pub problem_report: ProblemReport,
        pub _marker: PhantomData<T>,
    }
}

fn create_proposal_message_from_attachment<T: HolderCredentialIssuanceFormat>(
    attachment_data: Vec<u8>,
    preview: Option<CredentialPreviewV2>,
    thread_id: Option<String>,
) -> ProposeCredentialV2 {
    let attachment_content = AttachmentType::Base64(base64::encode(&attachment_data));
    let attach_id = Uuid::new_v4().to_string();
    let attachment = Attachment::builder()
        .id(attach_id.clone())
        .mime_type(MimeType::Json)
        .data(
            AttachmentData::builder()
                .content(attachment_content)
                .build(),
        )
        .build();

    let content = ProposeCredentialV2Content::builder()
        .formats(vec![AttachmentFormatSpecifier::builder()
            .attach_id(attach_id)
            .format(T::get_proposal_attachment_format())
            .build()])
        .filters_attach(vec![attachment]);

    let content = if let Some(preview) = preview {
        content.credential_preview(preview).build()
    } else {
        content.build()
    };

    let decorators = if let Some(id) = thread_id {
        ProposeCredentialV2Decorators::builder()
            .thread(Thread::builder().thid(id).build())
            .build()
    } else {
        ProposeCredentialV2Decorators::builder().build()
    };

    ProposeCredentialV2::builder()
        .id(Uuid::new_v4().to_string())
        .content(content)
        .decorators(decorators)
        .build()
}

fn create_request_message_from_attachment<T: HolderCredentialIssuanceFormat>(
    attachment_data: Vec<u8>,
    thread_id: Option<String>,
) -> RequestCredentialV2 {
    let attachment_content = AttachmentType::Base64(base64::encode(&attachment_data));
    let attach_id = uuid::Uuid::new_v4().to_string();
    let attachment = Attachment::builder()
        .id(attach_id.clone())
        .mime_type(MimeType::Json)
        .data(
            AttachmentData::builder()
                .content(attachment_content)
                .build(),
        )
        .build();

    let content = RequestCredentialV2Content::builder()
        .formats(vec![AttachmentFormatSpecifier::builder()
            .attach_id(attach_id)
            .format(T::get_request_attachment_format())
            .build()])
        .requests_attach(vec![attachment])
        .build();

    let decorators = if let Some(id) = thread_id {
        RequestCredentialV2Decorators::builder()
            .thread(Thread::builder().thid(id).build())
            .build()
    } else {
        RequestCredentialV2Decorators::builder().build()
    };
    RequestCredentialV2::builder()
        .id(Uuid::new_v4().to_string())
        .content(content)
        .decorators(decorators)
        .build()
}

pub struct HolderV2<S> {
    state: S,
    thread_id: String,
}

impl<T: HolderCredentialIssuanceFormat> HolderV2<ProposalPrepared<T>> {
    // initiate by creating a proposal message
    pub async fn with_proposal(
        input_data: &T::CreateProposalInput,
        preview: Option<CredentialPreviewV2>,
    ) -> VcxResult<Self> {
        let attachment_data = T::create_proposal_attachment_content(input_data).await?;
        let proposal = create_proposal_message_from_attachment::<T>(attachment_data, preview, None);

        Ok(HolderV2 {
            thread_id: get_thread_id_or_message_id!(proposal),
            state: ProposalPrepared {
                proposal,
                _marker: PhantomData,
            },
        })
    }

    // get prepared proposal message
    pub fn get_proposal(&self) -> &ProposeCredentialV2 {
        &self.state.proposal
    }

    // receive an offer in response to the proposal
    pub fn receive_offer(
        self,
        offer: OfferCredentialV2,
    ) -> VcxSMTransitionResult<HolderV2<OfferReceived<T>>, Self> {
        // TODO - verify thread ID?
        // verify_thread_id(&self.thread_id, offer.into())

        let new_state = OfferReceived {
            offer,
            _marker: PhantomData,
        };

        Ok(HolderV2 {
            state: new_state,
            thread_id: self.thread_id,
        })
    }
}

impl<T: HolderCredentialIssuanceFormat> HolderV2<OfferReceived<T>> {
    // initiate by receiving an offer
    pub fn from_offer(offer: OfferCredentialV2) -> Self {
        Self {
            thread_id: get_thread_id_or_message_id!(offer),
            state: OfferReceived {
                offer,
                _marker: PhantomData,
            },
        }
    }

    // TODO - helpers so that consumers can understand what offer they received? (is cred preview
    // garuanteed?)

    // TODO - helper function to give consumers a clue about what format is being used

    // respond to offer by preparing a proposal
    pub async fn prepare_proposal(
        self,
        input_data: &T::CreateProposalInput,
        preview: Option<CredentialPreviewV2>,
    ) -> VcxSMTransitionResult<HolderV2<ProposalPrepared<T>>, Self> {
        let attachment_data = match T::create_proposal_attachment_content(input_data).await {
            Ok(msg) => msg,
            Err(error) => {
                return Err(RecoveredSMError {
                    error,
                    state_machine: self,
                })
            }
        };
        let proposal = create_proposal_message_from_attachment::<T>(
            attachment_data,
            preview,
            Some(self.thread_id.clone()),
        );

        Ok(HolderV2 {
            state: ProposalPrepared {
                proposal,
                _marker: PhantomData,
            },
            thread_id: self.thread_id,
        })
    }

    // respond to offer by preparing a request
    pub async fn prepare_credential_request(
        self,
        input_data: &T::CreateRequestInput,
    ) -> VcxSMTransitionResult<HolderV2<RequestPrepared<T>>, Self> {
        let offer_message = &self.state.offer;

        let (attachment_data, output_metadata) =
            match T::create_request_attachment_content(offer_message, input_data).await {
                Ok((data, meta)) => (data, meta),
                Err(error) => {
                    return Err(RecoveredSMError {
                        error,
                        state_machine: self,
                    })
                }
            };

        let request = create_request_message_from_attachment::<T>(
            attachment_data,
            Some(self.thread_id.clone()),
        );

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

impl<T: HolderCredentialIssuanceFormat> HolderV2<RequestPrepared<T>> {
    // TODO - better name; this is "begin with request as a holder"
    // initiate by creating a request
    pub async fn with_request(
        input_data: &T::CreateRequestInput,
    ) -> VcxResult<HolderV2<RequestPrepared<T>>> {
        let (attachment_data, output_metadata) =
            T::create_request_attachment_content_independent_of_offer(input_data).await?;

        let request = create_request_message_from_attachment::<T>(attachment_data, None);

        let thread_id = get_thread_id_or_message_id!(request);

        let new_state = RequestPrepared {
            request_preparation_metadata: output_metadata,
            request,
        };

        Ok(HolderV2 {
            thread_id,
            state: new_state,
        })
    }

    // get prepared request message to be sent
    pub fn get_request(&self) -> &RequestCredentialV2 {
        &self.state.request
    }

    // receive process and store a credential sent by the issuer in respond to the request
    pub async fn receive_credential(
        self,
        credential: IssueCredentialV2,
        input_data: &T::StoreCredentialInput,
    ) -> VcxResult<HolderV2<CredentialReceived<T>>> {
        let credential_received_metadata = T::process_and_store_credential(
            &credential,
            input_data,
            self.state.request_preparation_metadata,
        )
        .await?;

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

impl<T: HolderCredentialIssuanceFormat> HolderV2<CredentialReceived<T>> {
    // indiciates if the issuer intends to issue more credentials
    pub fn is_more_credential_available(&self) -> bool {
        // check more_available > 0
        true
    }

    // prepare a request for the next credential if the issuer indicated there is more
    pub async fn prepare_request_for_next_credential(
        self,
        input_data: &T::CreateRequestInput,
    ) -> VcxSMTransitionResult<HolderV2<RequestPrepared<T>>, Self> {
        if !self.is_more_credential_available() {
            return Err(RecoveredSMError {
                error: AriesVcxError::from_msg(
                    AriesVcxErrorKind::ActionNotSupported,
                    "No more credentials to accept",
                ),
                state_machine: self,
            });
        }

        let (attachment_data, output_metadata) =
            match T::create_request_attachment_content_independent_of_offer(input_data).await {
                Ok((data, meta)) => (data, meta),
                Err(error) => {
                    return Err(RecoveredSMError {
                        error,
                        state_machine: self,
                    })
                }
            };

        let request = create_request_message_from_attachment::<T>(
            attachment_data,
            Some(self.thread_id.clone()),
        );

        let new_state = RequestPrepared {
            request_preparation_metadata: output_metadata,
            request,
        };

        Ok(HolderV2 {
            state: new_state,
            thread_id: self.thread_id,
        })
    }

    // prepare a problem report to refuse any more credentials and end the protocol
    pub fn prepare_refusal_to_more_credentials(self) -> HolderV2<Failed<T>> {
        todo!()
    }

    // transition to complete and prepare an ack message if the issuer requires one
    // TODO - consider enum variants for (HolderV2<AckPrepared>, HoldverV2<Completed>)
    pub fn prepare_ack_if_required(self) -> HolderV2<Complete<T>> {
        // if more_available: error?? as they should either problem report, or get more

        // if please_ack: else None
        let ack = Ack::builder()
            .id(uuid::Uuid::new_v4().to_string())
            .content(AckContent::builder().status(AckStatus::Ok).build())
            .decorators(
                AckDecorators::builder()
                    .thread(Thread::builder().thid(self.thread_id.clone()).build())
                    .build(),
            )
            .build();
        HolderV2 {
            state: Complete {
                ack: Some(ack),
                _marker: PhantomData,
            },
            thread_id: self.thread_id,
        }
    }
}

impl<T: HolderCredentialIssuanceFormat> HolderV2<Complete<T>> {
    // get the prepared ack message (if the issuer indiciated they want an ack)
    pub fn get_ack(&self) -> Option<&Ack> {
        self.state.ack.as_ref()
    }
}
