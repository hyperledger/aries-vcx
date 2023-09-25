use std::marker::PhantomData;

use messages::{
    decorators::{
        attachment::{Attachment, AttachmentData, AttachmentType},
        please_ack::{AckOn, PleaseAck},
        thread::Thread,
    },
    misc::MimeType,
    msg_fields::protocols::{
        cred_issuance::v2::{
            issue_credential::{
                IssueCredentialV2, IssueCredentialV2Content, IssueCredentialV2Decorators,
            },
            offer_credential::{
                OfferCredentialV2, OfferCredentialV2Content, OfferCredentialV2Decorators,
            },
            propose_credential::ProposeCredentialV2,
            request_credential::RequestCredentialV2,
            AttachmentFormatSpecifier, CredentialPreviewV2,
        },
        notification::ack::Ack,
    },
};
use uuid::Uuid;

use self::states::{
    Complete, CredentialPrepared, OfferPrepared, ProposalReceived, RequestReceived,
};
use super::{formats::issuer::IssuerCredentialIssuanceFormat, VcxSMTransitionResult};
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    handlers::util::get_thread_id_or_message_id,
    protocols::issuance_v2::RecoveredSMError,
};

pub mod states {
    use std::marker::PhantomData;

    use messages::msg_fields::protocols::{
        cred_issuance::v2::{
            issue_credential::IssueCredentialV2, offer_credential::OfferCredentialV2,
            propose_credential::ProposeCredentialV2, request_credential::RequestCredentialV2,
        },
        notification::ack::Ack,
    };

    use crate::protocols::issuance_v2::formats::issuer::IssuerCredentialIssuanceFormat;

    pub struct ProposalReceived<T: IssuerCredentialIssuanceFormat> {
        pub proposal: ProposeCredentialV2,
        pub _marker: PhantomData<T>,
    }

    pub struct OfferPrepared<T: IssuerCredentialIssuanceFormat> {
        pub offer_metadata: T::CreatedOfferMetadata,
        pub offer: OfferCredentialV2,
    }

    pub struct RequestReceived<T: IssuerCredentialIssuanceFormat> {
        pub from_offer_metadata: Option<T::CreatedOfferMetadata>,
        pub request: RequestCredentialV2,
    }

    pub struct CredentialPrepared<T: IssuerCredentialIssuanceFormat> {
        pub from_offer_metadata: Option<T::CreatedOfferMetadata>,
        pub credential_metadata: T::CreatedCredentialMetadata,
        pub credential: IssueCredentialV2,
        pub please_ack: bool,
    }

    pub struct Complete<T: IssuerCredentialIssuanceFormat> {
        pub ack: Option<Ack>,
        pub _marker: PhantomData<T>,
    }
}

fn create_offer_message_from_attachment<T: IssuerCredentialIssuanceFormat>(
    attachment_data: Vec<u8>,
    preview: CredentialPreviewV2,
    replacement_id: Option<String>,
    thread_id: Option<String>,
) -> OfferCredentialV2 {
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

    let content = OfferCredentialV2Content::builder()
        .credential_preview(preview)
        .formats(vec![AttachmentFormatSpecifier::builder()
            .attach_id(attach_id)
            .format(T::get_offer_attachment_format())
            .build()])
        .offers_attach(vec![attachment]);

    let content = if let Some(id) = replacement_id {
        content.replacement_id(id).build()
    } else {
        content.build()
    };

    let decorators = if let Some(id) = thread_id {
        OfferCredentialV2Decorators::builder()
            .thread(Thread::builder().thid(id).build())
            .build()
    } else {
        OfferCredentialV2Decorators::builder().build()
    };

    OfferCredentialV2::builder()
        .id(Uuid::new_v4().to_string())
        .content(content)
        .decorators(decorators)
        .build()
}

fn create_credential_message_from_attachment<T: IssuerCredentialIssuanceFormat>(
    attachment_data: Vec<u8>,
    please_ack: bool,
    thread_id: String,
    replacement_id: Option<String>,
) -> IssueCredentialV2 {
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

    let content = IssueCredentialV2Content::builder()
        .formats(vec![AttachmentFormatSpecifier::builder()
            .attach_id(attach_id)
            .format(T::get_credential_attachment_format())
            .build()])
        .credentials_attach(vec![attachment]);

    let content = if let Some(id) = replacement_id {
        content.replacement_id(id).build()
    } else {
        content.build()
    };

    let decorators =
        IssueCredentialV2Decorators::builder().thread(Thread::builder().thid(thread_id).build());
    let decorators = match please_ack {
        true => decorators
            .please_ack(PleaseAck::builder().on(vec![AckOn::Outcome]).build())
            .build(),
        false => decorators.build(),
    };

    IssueCredentialV2::builder()
        .id(Uuid::new_v4().to_string())
        .content(content)
        .decorators(decorators)
        .build()
}

pub struct IssuerV2<S> {
    state: S,
    thread_id: String,
}

impl<T: IssuerCredentialIssuanceFormat> IssuerV2<ProposalReceived<T>> {
    pub fn from_proposal(proposal: ProposeCredentialV2) -> Self {
        IssuerV2 {
            thread_id: get_thread_id_or_message_id!(proposal),
            state: ProposalReceived {
                proposal,
                _marker: PhantomData,
            },
        }
    }

    pub async fn prepare_offer(
        self,
        input_data: &T::CreateOfferInput,
        preview: CredentialPreviewV2,
        replacement_id: Option<String>,
    ) -> VcxSMTransitionResult<IssuerV2<OfferPrepared<T>>, Self> {
        let (attachment_data, offer_metadata) =
            match T::create_offer_attachment_content(input_data).await {
                Ok(data) => data,
                Err(error) => {
                    return Err(RecoveredSMError {
                        error,
                        state_machine: self,
                    })
                }
            };

        let offer = create_offer_message_from_attachment::<T>(
            attachment_data,
            preview,
            replacement_id,
            Some(self.thread_id.clone()),
        );

        let new_state = OfferPrepared {
            offer_metadata,
            offer,
        };

        Ok(IssuerV2 {
            state: new_state,
            thread_id: self.thread_id,
        })
    }

    // TODO - helpers so that consumers can understand what proposal they received?
}

impl<T: IssuerCredentialIssuanceFormat> IssuerV2<OfferPrepared<T>> {
    pub async fn with_offer(
        input_data: &T::CreateOfferInput,
        preview: CredentialPreviewV2,
        replacement_id: Option<String>,
    ) -> VcxResult<Self> {
        let (attachment_data, offer_metadata) =
            T::create_offer_attachment_content(input_data).await?;

        let offer = create_offer_message_from_attachment::<T>(
            attachment_data,
            preview,
            replacement_id,
            None,
        );

        let thread_id = get_thread_id_or_message_id!(offer);

        let new_state = OfferPrepared {
            offer_metadata,
            offer,
        };

        Ok(IssuerV2 {
            state: new_state,
            thread_id,
        })
    }

    pub fn get_offer(&self) -> &OfferCredentialV2 {
        &self.state.offer
    }

    pub fn receive_proposal(self, proposal: ProposeCredentialV2) -> IssuerV2<ProposalReceived<T>> {
        let new_state = ProposalReceived {
            proposal,
            _marker: PhantomData,
        };

        IssuerV2 {
            state: new_state,
            thread_id: self.thread_id,
        }
    }

    pub fn receive_request(self, request: RequestCredentialV2) -> IssuerV2<RequestReceived<T>> {
        let new_state = RequestReceived {
            from_offer_metadata: Some(self.state.offer_metadata),
            request,
        };

        IssuerV2 {
            state: new_state,
            thread_id: self.thread_id,
        }
    }
}

impl<T: IssuerCredentialIssuanceFormat> IssuerV2<RequestReceived<T>> {
    pub fn from_request(request: RequestCredentialV2) -> VcxResult<Self> {
        if !T::supports_request_independent_of_offer() {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::ActionNotSupported,
                "Receiving a request independent of an offer is unsupported for this format",
            ));
        }

        let thread_id = get_thread_id_or_message_id!(request);

        let new_state = RequestReceived {
            from_offer_metadata: None,
            request,
        };

        Ok(Self {
            state: new_state,
            thread_id,
        })
    }

    pub async fn prepare_credential(
        self,
        input_data: &T::CreateCredentialInput,
        please_ack: Option<bool>, // defaults to false
        replacement_id: Option<String>,
    ) -> VcxSMTransitionResult<IssuerV2<CredentialPrepared<T>>, Self> {
        let request = &self.state.request;

        let res = match &self.state.from_offer_metadata {
            Some(offer) => {
                T::create_credential_attachment_content(offer, request, input_data).await
            }
            None => {
                T::create_credential_attachment_content_independent_of_offer(request, input_data)
                    .await
            }
        };

        let (attachment_data, cred_metadata) = match res {
            Ok(data) => data,
            Err(error) => {
                return Err(RecoveredSMError {
                    error,
                    state_machine: self,
                })
            }
        };

        let please_ack = please_ack.unwrap_or(false);
        let credential = create_credential_message_from_attachment::<T>(
            attachment_data,
            please_ack,
            self.thread_id.clone(),
            replacement_id,
        );

        let new_state = CredentialPrepared {
            from_offer_metadata: self.state.from_offer_metadata,
            credential_metadata: cred_metadata,
            credential,
            please_ack,
        };

        Ok(IssuerV2 {
            state: new_state,
            thread_id: self.thread_id,
        })
    }
}

impl<T: IssuerCredentialIssuanceFormat> IssuerV2<CredentialPrepared<T>> {
    pub fn get_credential(&self) -> &IssueCredentialV2 {
        &self.state.credential
    }

    pub fn get_credential_creation_metadata(&self) -> &T::CreatedCredentialMetadata {
        &self.state.credential_metadata
    }

    pub fn is_expecting_ack(&self) -> bool {
        self.state.please_ack
    }

    pub fn complete_without_ack(self) -> VcxSMTransitionResult<IssuerV2<Complete<T>>, Self> {
        if self.is_expecting_ack() {
            return Err(RecoveredSMError {
                error: AriesVcxError::from_msg(
                    AriesVcxErrorKind::ActionNotSupported,
                    "Cannot transition until ACK is received",
                ),
                state_machine: self,
            });
        }

        let new_state = Complete {
            ack: None,
            _marker: PhantomData,
        };

        Ok(IssuerV2 {
            state: new_state,
            thread_id: self.thread_id,
        })
    }

    pub fn complete_with_ack(self, ack: Ack) -> IssuerV2<Complete<T>> {
        let new_state = Complete {
            ack: Some(ack),
            _marker: PhantomData,
        };

        IssuerV2 {
            state: new_state,
            thread_id: self.thread_id,
        }
    }
}
