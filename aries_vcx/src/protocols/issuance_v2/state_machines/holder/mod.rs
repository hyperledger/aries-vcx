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
        notification::ack::{AckContent, AckDecorators, AckStatus},
        report_problem::{Description, ProblemReportContent, ProblemReportDecorators},
    },
};
use uuid::Uuid;

use self::states::{
    completed::Completed, credential_received::CredentialReceived, failed::Failed,
    offer_received::OfferReceived, proposal_prepared::ProposalPrepared,
    request_prepared::RequestPrepared,
};
use crate::{
    errors::error::VcxResult,
    handlers::util::{get_thread_id_or_message_id, matches_thread_id},
    protocols::issuance_v2::{
        formats::holder::HolderCredentialIssuanceFormat,
        processing::holder::{
            create_proposal_message_from_attachments, create_request_message_from_attachments,
        },
        unmatched_thread_id_error, RecoveredSMError, VcxSMTransitionResult,
    },
};

/// Represents a type-state machine which walks through issue-credential-v2 from the Holder
/// perspective. https://github.com/hyperledger/aries-rfcs/blob/main/features/0453-issue-credential-v2/README.md
///
/// States in the HolderV2 APIs require knowledge of the credential format being used. As such, this
/// API only supports usage of a single credential format being used throughout a single protocol
/// flow.
///
/// To indicate which credential format should be used by [HolderV2], an implementation of
/// [HolderCredentialIssuanceFormat] should be used as the generic argument when required.
///
/// For instance, the following will bootstrap a [HolderV2] into the [ProposalPrepared] state,
/// with the `HyperledgerIndyHolderCredentialIssuanceFormat` format.
///
/// ```no_run
/// let holder =
///     HolderV2::<ProposalPrepared<HyperledgerIndyHolderCredentialIssuanceFormat>>::with_proposal(
///         &proposal_input,
///         Some(proposal_preview.clone()),
///     )
///     .await
///     .unwrap();
/// ```
///
/// For more information about formats, see [HolderCredentialIssuanceFormat] documentation.
pub struct HolderV2<S> {
    state: S,
    thread_id: String,
}

impl<S> HolderV2<S> {
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

impl<T: HolderCredentialIssuanceFormat> HolderV2<ProposalPrepared<T>> {
    /// Initiate a new [HolderV2] by preparing a proposal message from the provided input for
    /// creating a proposal with the choosen [HolderCredentialIssuanceFormat].
    ///
    /// Additionally, a [CredentialPreviewV2] can be provided to attach more proposal information
    /// in the proposal message payload.
    pub async fn with_proposal(
        input_data: &T::CreateProposalInput,
        preview: Option<CredentialPreviewV2>,
    ) -> VcxResult<(Self, ProposeCredentialV2)> {
        let attachment_data = T::create_proposal_attachment_content(input_data).await?;
        let attachments_format_and_data =
            vec![(T::get_proposal_attachment_format(), attachment_data)];
        let proposal =
            create_proposal_message_from_attachments(attachments_format_and_data, preview, None);

        let holder = HolderV2 {
            thread_id: get_thread_id_or_message_id!(proposal),
            state: ProposalPrepared {
                _marker: PhantomData,
            },
        };

        Ok((holder, proposal))
    }

    /// Receive an incoming [OfferCredentialV2] message for this protocol. On success, the
    /// [HolderV2] transitions into the [OfferReceived] state.
    ///
    /// This API should only be used for offers which are in response to an ongoing [HolderV2]
    /// protocol thread. New offers should be received via [HolderV2::from_offer].
    ///
    /// In the event of failure, an error is returned which contains the reason for failure
    /// and the state machine before any transitions. Consumers should decide whether the failure
    /// is terminal, in which case they should prepare a problem report.
    pub fn receive_offer(
        self,
        offer: OfferCredentialV2,
    ) -> VcxSMTransitionResult<HolderV2<OfferReceived<T>>, Self> {
        let is_match = offer
            .decorators
            .thread
            .as_ref()
            .map_or(false, |t| t.thid == self.thread_id);
        if !is_match {
            return Err(RecoveredSMError {
                error: unmatched_thread_id_error(offer.into(), &self.thread_id),
                state_machine: self,
            });
        }

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
    /// Initialize a [HolderV2] protocol from a new incoming [OfferCredentialV2] message.
    ///
    /// The [HolderCredentialIssuanceFormat] used during initialization should be suitable for
    /// the attachments within the [OfferCredentialV2] message, or else the [HolderV2] will not
    /// be able to transition forward without failure.
    ///
    /// This API should only be used for offers which are initializing a NEW issue-credential-v2
    /// thread. [OfferCredentialV2] messages which are in response to an ongoing protocol thread
    /// should be handled via [HolderV2::receive_offer].
    pub fn from_offer(offer: OfferCredentialV2) -> Self {
        Self {
            thread_id: get_thread_id_or_message_id!(offer),
            state: OfferReceived {
                offer,
                _marker: PhantomData,
            },
        }
    }

    /// Get the details and credential preview of the offer that was received. The returned
    /// [HolderCredentialIssuanceFormat::OfferDetails] data will contain data specific to the
    /// format being used.
    pub fn get_offer_details(&self) -> VcxResult<(T::OfferDetails, &CredentialPreviewV2)> {
        let details = T::extract_offer_details(&self.state.offer)?;
        let preview = &self.state.offer.content.credential_preview;

        Ok((details, preview))
    }

    /// Respond to an offer by preparing a new proposal. This API can be used repeatedly to
    /// negotiate the offer with the issuer until an agreement is reached.
    ///
    /// A proposal is prepared in the format of [HolderCredentialIssuanceFormat], using the provided
    /// input data to create it. Additionally, a [CredentialPreviewV2] can be attached to give
    /// further details to the issuer about the proposal.
    ///
    /// In the event of failure, an error is returned which contains the reason for failure
    /// and the state machine before any transitions. Consumers should decide whether the failure
    /// is terminal, in which case they should prepare a problem report.
    pub async fn prepare_proposal(
        self,
        input_data: &T::CreateProposalInput,
        preview: Option<CredentialPreviewV2>,
    ) -> VcxSMTransitionResult<(HolderV2<ProposalPrepared<T>>, ProposeCredentialV2), Self> {
        let attachment_data = match T::create_proposal_attachment_content(input_data).await {
            Ok(msg) => msg,
            Err(error) => {
                return Err(RecoveredSMError {
                    error,
                    state_machine: self,
                })
            }
        };
        let attachments_format_and_data =
            vec![(T::get_proposal_attachment_format(), attachment_data)];
        let proposal = create_proposal_message_from_attachments(
            attachments_format_and_data,
            preview,
            Some(self.thread_id.clone()),
        );

        let holder = HolderV2 {
            state: ProposalPrepared {
                _marker: PhantomData,
            },
            thread_id: self.thread_id,
        };

        Ok((holder, proposal))
    }

    /// Respond to an offer by preparing a request (to accept the offer). The request is prepared in
    /// the format of [HolderCredentialIssuanceFormat] using the input data to create it. If the
    /// request is successfully prepared, the [HolderV2] will transition to [RequestPrepared] where
    /// the request message can be sent.
    ///
    /// In the event of failure, an error is returned which contains the reason for failure
    /// and the state machine before any transitions. Consumers should decide whether the failure
    /// is terminal, in which case they should prepare a problem report.
    pub async fn prepare_credential_request(
        self,
        input_data: &T::CreateRequestInput,
    ) -> VcxSMTransitionResult<(HolderV2<RequestPrepared<T>>, RequestCredentialV2), Self> {
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

        let attachments_format_and_data =
            vec![(T::get_request_attachment_format(), attachment_data)];
        let request = create_request_message_from_attachments(
            attachments_format_and_data,
            Some(self.thread_id.clone()),
        );

        let new_state = RequestPrepared {
            request_preparation_metadata: output_metadata,
        };

        let holder = HolderV2 {
            state: new_state,
            thread_id: self.thread_id,
        };

        Ok((holder, request))
    }
}

impl<T: HolderCredentialIssuanceFormat> HolderV2<RequestPrepared<T>> {
    /// Initialize a [HolderV2] by preparing a request. This API should only be used to create
    /// standalone requests that are not in response to an ongoing protocol thread (i.e. in
    /// response to an offer).
    ///
    /// To create a request in response to an ongoing protocol thread, the
    /// [HolderV2::prepare_credential_request] method should be used.
    ///
    /// The request is prepared in the [HolderCredentialIssuanceFormat] using the input data to
    /// create it. Note that the [HolderCredentialIssuanceFormat] MUST support standalone request
    /// creation for this function to succeed, some formats (such as hlindy or anoncreds) do not
    /// support this.
    pub async fn with_request(
        input_data: &T::CreateRequestInput,
    ) -> VcxResult<(HolderV2<RequestPrepared<T>>, RequestCredentialV2)> {
        let (attachment_data, output_metadata) =
            T::create_request_attachment_content_independent_of_offer(input_data).await?;

        let attachments_format_and_data =
            vec![(T::get_request_attachment_format(), attachment_data)];
        let request = create_request_message_from_attachments(attachments_format_and_data, None);

        let thread_id = get_thread_id_or_message_id!(request);

        let new_state = RequestPrepared {
            request_preparation_metadata: output_metadata,
        };

        let holder = HolderV2 {
            thread_id,
            state: new_state,
        };

        Ok((holder, request))
    }

    /// Receive a credential in response to a request message that was sent to the issuer.
    /// The received credential is processed and stored in accordance to the
    /// [HolderCredentialIssuanceFormat] being used.
    ///
    /// In the event of failure, an error is returned which contains the reason for failure
    /// and the state machine before any transitions. Consumers should decide whether the failure
    /// is terminal, in which case they should prepare a problem report.
    pub async fn receive_credential(
        self,
        credential: IssueCredentialV2,
        input_data: &T::StoreCredentialInput,
    ) -> VcxSMTransitionResult<HolderV2<CredentialReceived<T>>, Self> {
        let is_match = matches_thread_id!(credential, self.thread_id.as_str());
        if !is_match {
            return Err(RecoveredSMError {
                error: unmatched_thread_id_error(credential.into(), &self.thread_id),
                state_machine: self,
            });
        }
        let credential_received_metadata = match T::process_and_store_credential(
            &credential,
            input_data,
            &self.state.request_preparation_metadata,
        )
        .await
        {
            Ok(data) => data,
            Err(error) => {
                return Err(RecoveredSMError {
                    error,
                    state_machine: self,
                })
            }
        };

        let new_state = CredentialReceived {
            credential,
            stored_credential_metadata: credential_received_metadata,
        };
        Ok(HolderV2 {
            state: new_state,
            thread_id: self.thread_id,
        })
    }
}

impl<T: HolderCredentialIssuanceFormat> HolderV2<CredentialReceived<T>> {
    /// Get details about the credential that was received and stored.
    /// The details are specific to the [HolderCredentialIssuanceFormat] being used.
    pub fn get_stored_credential_metadata(&self) -> &T::StoredCredentialMetadata {
        &self.state.stored_credential_metadata
    }

    // TODO - consider enum variants for (HolderV2<AckPrepared>, HoldverV2<Completed>)
    /// Transition into the [Complete] state, by preparing an Ack message, only if required.
    pub fn prepare_ack_if_required(self) -> (HolderV2<Completed<T>>, Option<AckCredentialV2>) {
        let should_ack = self.state.credential.decorators.please_ack.is_some();

        let ack = if should_ack {
            Some(
                AckCredentialV2::builder()
                    .id(uuid::Uuid::new_v4().to_string())
                    .content(AckContent::builder().status(AckStatus::Ok).build())
                    .decorators(
                        AckDecorators::builder()
                            .thread(Thread::builder().thid(self.thread_id.clone()).build())
                            .build(),
                    )
                    .build(),
            )
        } else {
            None
        };
        let holder = HolderV2 {
            state: Completed {
                _marker: PhantomData,
            },
            thread_id: self.thread_id,
        };

        (holder, ack)
    }
}

impl HolderV2<Failed> {
    /// Get the prepared [CredIssuanceProblemReportV2] to be sent to the issuer to report a failure.
    pub fn get_problem_report(&self) -> &CredIssuanceProblemReportV2 {
        &self.state.problem_report
    }
}

impl<S> HolderV2<S> {
    /// Transition into the [Failed] state by preparing a problem report message for the issuer.
    /// The problem report message is generated by using details from the provided [Error].
    pub fn prepare_problem_report_with_error<E>(self, err: &E) -> HolderV2<Failed>
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

        HolderV2 {
            state: new_state,
            thread_id: self.thread_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use base64::{engine::general_purpose, Engine};
    use messages::decorators::attachment::AttachmentType;
    use shared_vcx::maybe_known::MaybeKnown;

    use crate::protocols::issuance_v2::{
        formats::holder::mocks::MockHolderCredentialIssuanceFormat,
        state_machines::holder::{states::proposal_prepared::ProposalPrepared, HolderV2},
    };

    #[tokio::test]
    async fn test_with_proposal_creates_message_with_attachments() {
        // note synchronization issues. might need to just set this once globally and use constant
        // data
        let ctx = MockHolderCredentialIssuanceFormat::create_proposal_attachment_content_context();

        ctx.expect()
            .returning(|_| Ok(String::from("data").into_bytes()));

        let ctx2 = MockHolderCredentialIssuanceFormat::get_proposal_attachment_format_context();
        ctx2.expect()
            .returning(|| MaybeKnown::Unknown(String::from("format")));

        let (_holder, proposal) =
            HolderV2::<ProposalPrepared<MockHolderCredentialIssuanceFormat>>::with_proposal(
                &String::from("in"),
                None,
            )
            .await
            .unwrap();

        let formats = proposal.content.formats.clone();
        let attachments = proposal.content.filters_attach.clone();

        assert_eq!(formats.len(), 1);
        assert_eq!(attachments.len(), 1);

        assert_eq!(formats[0].attach_id, attachments[0].id.clone().unwrap());
        assert_eq!(
            formats[0].format,
            MaybeKnown::Unknown(String::from("format"))
        );

        let AttachmentType::Base64(b64_content) = attachments[0].data.content.clone() else {
            panic!("wrong attachment type")
        };

        let decoded = general_purpose::URL_SAFE.decode(&b64_content).unwrap();

        assert_eq!(String::from_utf8(decoded).unwrap(), String::from("data"));
    }

    // TODO - unit test all when we're happy with the layout
}
