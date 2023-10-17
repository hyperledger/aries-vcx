pub mod states;

use std::{error::Error, marker::PhantomData};

use messages::{
    decorators::thread::Thread,
    msg_fields::protocols::{
        cred_issuance::v2::{
            ack::AckCredentialV2, issue_credential::IssueCredentialV2,
            offer_credential::OfferCredentialV2, problem_report::CredIssuanceProblemReportV2,
            propose_credential::ProposeCredentialV2, request_credential::RequestCredentialV2,
            CredentialPreviewV2,
        },
        report_problem::{Description, ProblemReportContent, ProblemReportDecorators},
    },
};
use uuid::Uuid;

use self::states::{
    completed::Completed, credential_prepared::CredentialPrepared, failed::Failed,
    offer_prepared::OfferPrepared, proposal_received::ProposalReceived,
    request_received::RequestReceived,
};
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    handlers::util::{get_thread_id_or_message_id, matches_thread_id},
    protocols::issuance_v2::{
        formats::issuer::IssuerCredentialIssuanceFormat,
        processing::issuer::{
            create_credential_message_from_attachments, create_offer_message_from_attachments,
        },
        unmatched_thread_id_error, RecoveredSMError, VcxSMTransitionResult,
    },
};

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

    /// Respond to a proposal by preparing a new offer. This API can be used repeatedly to negotiate
    /// the offer with the holder until an agreement is reached.
    ///
    /// An offer is prepared in the format of [IssuerCredentialIssuanceFormat], using the provided
    /// input data to create it. Additionally, a [CredentialPreviewV2] is attached to give further
    /// details to the holder about the offer.
    ///
    /// In the event of failure, an error is returned which contains the reason for failure
    /// and the state machine before any transitions. Consumers should decide whether the failure
    /// is terminal, in which case they should prepare a problem report.
    pub async fn prepare_offer(
        self,
        input_data: &T::CreateOfferInput,
        preview: CredentialPreviewV2,
        replacement_id: Option<String>,
    ) -> VcxSMTransitionResult<(IssuerV2<OfferPrepared<T>>, OfferCredentialV2), Self> {
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
        let attachments_format_and_data = vec![(T::get_offer_attachment_format(), attachment_data)];
        let offer = create_offer_message_from_attachments(
            attachments_format_and_data,
            preview,
            replacement_id,
            Some(self.thread_id.clone()),
        );

        let new_state = OfferPrepared { offer_metadata };

        let issuer = IssuerV2 {
            state: new_state,
            thread_id: self.thread_id,
        };

        Ok((issuer, offer))
    }
}

impl<T: IssuerCredentialIssuanceFormat> IssuerV2<OfferPrepared<T>> {
    /// Initiate a new [IssuerV2] by preparing a offer message from the provided input for
    /// creating a offer with the choosen [IssuerCredentialIssuanceFormat].
    ///
    /// Additionally, a [CredentialPreviewV2] is provided to attach more credential information
    /// in the offer message payload.
    pub async fn with_offer(
        input_data: &T::CreateOfferInput,
        preview: CredentialPreviewV2,
        replacement_id: Option<String>,
    ) -> VcxResult<(Self, OfferCredentialV2)> {
        let (attachment_data, offer_metadata) =
            T::create_offer_attachment_content(input_data).await?;

        let attachments_format_and_data = vec![(T::get_offer_attachment_format(), attachment_data)];
        let offer = create_offer_message_from_attachments(
            attachments_format_and_data,
            preview,
            replacement_id,
            None,
        );

        let thread_id = get_thread_id_or_message_id!(offer);

        let new_state = OfferPrepared { offer_metadata };

        let issuer = IssuerV2 {
            state: new_state,
            thread_id,
        };

        Ok((issuer, offer))
    }

    /// Receive an incoming [ProposeCredentialV2] message for this protocol. On success, the
    /// [IssuerV2] transitions into the [ProposalReceived] state.
    ///
    /// This API should only be used for proposals which are in response to an ongoing [IssuerV2]
    /// protocol thread. New proposals should be received via [IssuerV2::from_proposal].
    ///
    /// In the event of failure, an error is returned which contains the reason for failure
    /// and the state machine before any transitions. Consumers should decide whether the failure
    /// is terminal, in which case they should prepare a problem report.
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

    /// Receive a request in response to an offer that was sent to the holder.
    ///
    /// This API should only be used for requests that are in response to an ongoing [IssuerV2]
    /// protocol thread. To receive new standalone requests, [IssuerV2::from_request] should be
    /// used.
    ///
    /// In the event of failure, an error is returned which contains the reason for failure
    /// and the state machine before any transitions. Consumers should decide whether the failure
    /// is terminal, in which case they should prepare a problem report.
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
    /// Initialize an [IssuerV2] by receiving a standalone request message from a holder. This API
    /// should only be used for standalone requests not in response to an ongoing protocol thread.
    ///
    /// To receive a request in response to an ongoing protocol thread, the
    /// [IssuerV2::receive_request] method should be used.
    ///
    /// The request should contain an attachment in the suitable [IssuerCredentialIssuanceFormat]
    /// format, and the [IssuerCredentialIssuanceFormat] MUST support receiving standalone requests
    /// for this function to succeed. Some formats (such as hlindy or anoncreds) do not
    /// support this.
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

    /// Prepare a credential message in response to a received request. The prepared credential will
    /// be in the [IssuerCredentialIssuanceFormat] format, and will be created using the associated
    /// input data.
    ///
    /// Additionally other flags can be attached to the prepared message for the holder. Notably:
    /// * `please_ack` - whether the holder should acknowledge that they receive the credential
    /// * `replacement_id` - a unique ID which can be used across credential issuances to indicate
    ///   that this credential should effectively 'replace' the last credential that this issuer
    ///   issued to them with the same `replacement_id`.
    ///
    /// In the event of failure, an error is returned which contains the reason for failure
    /// and the state machine before any transitions. Consumers should decide whether the failure
    /// is terminal, in which case they should prepare a problem report.
    pub async fn prepare_credential(
        self,
        input_data: &T::CreateCredentialInput,
        please_ack: Option<bool>, // defaults to false
        replacement_id: Option<String>,
    ) -> VcxSMTransitionResult<(IssuerV2<CredentialPrepared<T>>, IssueCredentialV2), Self> {
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
        let attachments_format_and_data =
            vec![(T::get_credential_attachment_format(), attachment_data)];

        let please_ack = please_ack.unwrap_or(false);
        let credential = create_credential_message_from_attachments(
            attachments_format_and_data,
            please_ack,
            self.thread_id.clone(),
            replacement_id,
        );

        let new_state = CredentialPrepared {
            from_offer_metadata: self.state.from_offer_metadata,
            credential_metadata: cred_metadata,
            please_ack,
        };

        let issuer = IssuerV2 {
            state: new_state,
            thread_id: self.thread_id,
        };

        Ok((issuer, credential))
    }
}

impl<T: IssuerCredentialIssuanceFormat> IssuerV2<CredentialPrepared<T>> {
    /// Get details about the credential that was prepared.
    /// The details are specific to the [IssuerCredentialIssuanceFormat] being used.
    pub fn get_credential_creation_metadata(&self) -> &T::CreatedCredentialMetadata {
        &self.state.credential_metadata
    }

    /// Whether or not this [IssuerV2] is expecting an Ack message to complete.
    pub fn is_expecting_ack(&self) -> bool {
        self.state.please_ack
    }

    /// Transition into a completed state without receiving an ack message from the holder.
    ///
    /// In the case where the [IssuerV2] was expecting an ack, this method will fail.
    ///
    /// In the event of failure, an error is returned which contains the reason for failure
    /// and the state machine before any transitions. Consumers should decide whether the failure
    /// is terminal, in which case they should prepare a problem report.
    pub fn complete_without_ack(self) -> VcxSMTransitionResult<IssuerV2<Completed<T>>, Self> {
        if self.is_expecting_ack() {
            return Err(RecoveredSMError {
                error: AriesVcxError::from_msg(
                    AriesVcxErrorKind::ActionNotSupported,
                    "Cannot transition until ACK is received",
                ),
                state_machine: self,
            });
        }

        let new_state = Completed {
            ack: None,
            _marker: PhantomData,
        };

        Ok(IssuerV2 {
            state: new_state,
            thread_id: self.thread_id,
        })
    }

    /// Transition into a completed state by receiving an incoming ack message from the holder.
    ///
    /// In the event of failure, an error is returned which contains the reason for failure
    /// and the state machine before any transitions. Consumers should decide whether the failure
    /// is terminal, in which case they should prepare a problem report.
    pub fn complete_with_ack(
        self,
        ack: AckCredentialV2,
    ) -> VcxSMTransitionResult<IssuerV2<Completed<T>>, Self> {
        let is_match = matches_thread_id!(ack, self.thread_id.as_str());
        if !is_match {
            return Err(RecoveredSMError {
                error: unmatched_thread_id_error(ack.into(), &self.thread_id),
                state_machine: self,
            });
        }

        let new_state = Completed {
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
    /// Get the prepared [CredIssuanceProblemReportV2] to be sent to the holder to report a failure.
    pub fn get_problem_report(&self) -> &CredIssuanceProblemReportV2 {
        &self.state.problem_report
    }
}

impl<S> IssuerV2<S> {
    /// Transition into the [Failed] state by preparing a problem report message for the holder.
    /// The problem report message is generated by using details from the provided [Error].
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
