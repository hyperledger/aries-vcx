pub mod states;

use std::{error::Error, marker::PhantomData};

use messages::{
    decorators::{
        attachment::{Attachment, AttachmentData, AttachmentType},
        please_ack::{AckOn, PleaseAck},
        thread::Thread,
    },
    misc::MimeType,
    msg_fields::protocols::{
        cred_issuance::v2::{
            ack::AckCredentialV2,
            issue_credential::{
                IssueCredentialV2, IssueCredentialV2Content, IssueCredentialV2Decorators,
            },
            offer_credential::{
                OfferCredentialV2, OfferCredentialV2Content, OfferCredentialV2Decorators,
            },
            problem_report::CredIssuanceProblemReportV2,
            propose_credential::ProposeCredentialV2,
            request_credential::RequestCredentialV2,
            AttachmentFormatSpecifier, CredentialPreviewV2,
        },
        report_problem::{Description, ProblemReportContent, ProblemReportDecorators},
    },
};
use uuid::Uuid;

use self::states::{
    complete::Complete, credential_prepared::CredentialPrepared, failed::Failed,
    offer_prepared::OfferPrepared, proposal_received::ProposalReceived,
    request_received::RequestReceived,
};
use super::{
    formats::issuer::IssuerCredentialIssuanceFormat, unmatched_thread_id_error,
    VcxSMTransitionResult,
};
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    handlers::util::{get_thread_id_or_message_id, matches_thread_id},
    protocols::issuance_v2::RecoveredSMError,
};

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

/// Represents a type-state machine which walks through issue-credential-v2 from the Issuer
/// perspective. https://github.com/hyperledger/aries-rfcs/blob/main/features/0453-issue-credential-v2/README.md
///
/// States in the [IssuerV2] APIs require knowledge of the credential format being used. As such,
/// this API only supports usage of a single credential format being used throughout a single
/// protocol flow.
///
/// To indicate which credential format should be used by [IssuerV2], an implementation of
/// [IssuerCredentialIssuanceFormat] should be used as the generic argument when required.
///
/// For instance, the following will bootstrap a [IssuerV2] into the [ProposalPrepared] state,
/// with the `HyperledgerIndyIssuerCredentialIssuanceFormat` format.
///
/// ```no_run
/// let issuer =
///     IssuerV2::<OfferPrepared<HyperledgerIndyIssuerCredentialIssuanceFormat>>::with_offer(
///         &offer_data,
///         offer_preview,
///         None,
///     )
///     .await
///     .unwrap();
/// ```
///
/// For more information about formats, see [IssuerCredentialIssuanceFormat] documentation.
pub struct IssuerV2<S> {
    state: S,
    thread_id: String,
}

impl<S> IssuerV2<S> {
    pub fn from_parts(thread_id: String, state: S) -> Self {
        Self { state, thread_id }
    }

    pub fn into_parts(self) -> (String, S) {
        (self.thread_id, self.state)
    }

    /// Get the thread ID that is being used for this protocol instance.
    pub fn get_thread_id(&self) -> &str {
        &self.thread_id
    }

    pub fn get_state(&self) -> &S {
        &self.state
    }
}

impl<T: IssuerCredentialIssuanceFormat> IssuerV2<ProposalReceived<T>> {
    /// Initialize a new [IssuerV2] by receiving an incoming [ProposeCredentialV2] message from a
    /// holder.
    ///
    /// The [IssuerCredentialIssuanceFormat] used during initialization should be suitable
    /// for the attachments within the [ProposeCredentialV2] message, or else the [IssuerV2] will
    /// not be able to transition forward without failure.
    ///
    /// This API should only be used for standalone proposals that aren't apart of an existing
    /// protocol thread. Proposals in response to an ongoing thread should be handled via
    /// [HolderV2::receive_proposal].
    pub fn from_proposal(proposal: ProposeCredentialV2) -> Self {
        IssuerV2 {
            thread_id: get_thread_id_or_message_id!(proposal),
            state: ProposalReceived {
                proposal,
                _marker: PhantomData,
            },
        }
    }

    /// Get the details and credential preview (if any) of the proposal that was received. The
    /// returned [IssuerCredentialIssuanceFormat::ProposalDetails] data will contain data
    /// specific to the format being used.
    pub fn get_proposal_details(
        &self,
    ) -> VcxResult<(T::ProposalDetails, Option<&CredentialPreviewV2>)> {
        let details = T::extract_proposal_details(&self.state.proposal)?;
        let preview = self.state.proposal.content.credential_preview.as_ref();

        Ok((details, preview))
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

    pub fn receive_proposal(
        self,
        proposal: ProposeCredentialV2,
    ) -> VcxSMTransitionResult<IssuerV2<ProposalReceived<T>>, Self> {
        let is_match = proposal
            .decorators
            .thread
            .as_ref()
            .map_or(false, |t| t.thid == self.thread_id);
        if !is_match {
            return Err(RecoveredSMError {
                error: unmatched_thread_id_error(proposal.into(), &self.thread_id),
                state_machine: self,
            });
        }

        let new_state = ProposalReceived {
            proposal,
            _marker: PhantomData,
        };

        Ok(IssuerV2 {
            state: new_state,
            thread_id: self.thread_id,
        })
    }

    pub fn receive_request(
        self,
        request: RequestCredentialV2,
    ) -> VcxSMTransitionResult<IssuerV2<RequestReceived<T>>, Self> {
        let is_match = request
            .decorators
            .thread
            .as_ref()
            .map_or(false, |t| t.thid == self.thread_id);
        if !is_match {
            return Err(RecoveredSMError {
                error: unmatched_thread_id_error(request.into(), &self.thread_id),
                state_machine: self,
            });
        }

        let new_state = RequestReceived {
            from_offer_metadata: Some(self.state.offer_metadata),
            request,
        };

        Ok(IssuerV2 {
            state: new_state,
            thread_id: self.thread_id,
        })
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

    pub fn complete_with_ack(
        self,
        ack: AckCredentialV2,
    ) -> VcxSMTransitionResult<IssuerV2<Complete<T>>, Self> {
        let is_match = matches_thread_id!(ack, self.thread_id.as_str());
        if !is_match {
            return Err(RecoveredSMError {
                error: unmatched_thread_id_error(ack.into(), &self.thread_id),
                state_machine: self,
            });
        }

        let new_state = Complete {
            ack: Some(ack),
            _marker: PhantomData,
        };

        Ok(IssuerV2 {
            state: new_state,
            thread_id: self.thread_id,
        })
    }
}

impl IssuerV2<Failed> {
    pub fn get_problem_report(&self) -> &CredIssuanceProblemReportV2 {
        &self.state.problem_report
    }
}

impl<S> IssuerV2<S> {
    pub fn prepare_problem_report_with_error<E>(self, err: &E) -> IssuerV2<Failed>
    where
        E: Error,
    {
        let content = ProblemReportContent::builder()
            .description(Description::builder().code(err.to_string()).build())
            .build();

        let decorators = ProblemReportDecorators::builder()
            .thread(Thread::builder().thid(self.thread_id.clone()).build())
            .build();

        let report = CredIssuanceProblemReportV2::builder()
            .id(Uuid::new_v4().to_string())
            .content(content)
            .decorators(decorators)
            .build();

        let new_state = Failed {
            problem_report: report,
        };

        IssuerV2 {
            state: new_state,
            thread_id: self.thread_id,
        }
    }
}
