use std::{collections::HashMap, fmt::Display, sync::Arc};

use messages::{
    a2a::{A2AMessage, MessageId},
    concepts::{ack::Ack, problem_report::ProblemReport},
    protocols::issuance::{
        credential::Credential,
        credential_offer::{CredentialOffer, OfferInfo},
        credential_proposal::CredentialProposal,
        credential_request::CredentialRequest,
        CredentialPreviewData,
    },
    status::Status,
};

use crate::{
    common::credentials::{encoding::encode_attributes, is_cred_revoked},
    core::profile::profile::Profile,
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    protocols::{
        common::build_problem_report_msg,
        issuance::{
            actions::CredentialIssuanceAction,
            issuer::states::{
                credential_sent::CredentialSentState, finished::FinishedState, initial::InitialIssuerState,
                offer_sent::OfferSentState, offer_set::OfferSetState, proposal_received::ProposalReceivedState,
                requested_received::RequestReceivedState,
            },
            verify_thread_id,
        },
        SendClosure,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum IssuerFullState {
    Initial(InitialIssuerState),
    OfferSet(OfferSetState),
    ProposalReceived(ProposalReceivedState),
    OfferSent(OfferSentState),
    RequestReceived(RequestReceivedState),
    CredentialSent(CredentialSentState),
    Finished(FinishedState),
}

#[derive(Debug, PartialEq, Eq)]
pub enum IssuerState {
    Initial,
    OfferSet,
    ProposalReceived,
    OfferSent,
    RequestReceived,
    CredentialSent,
    Finished,
    Failed,
}

// todo: Use this approach for logging in other protocols as well.
impl Display for IssuerFullState {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        match *self {
            IssuerFullState::Initial(_) => f.write_str("Initial"),
            IssuerFullState::OfferSet(_) => f.write_str("OfferSet"),
            IssuerFullState::ProposalReceived(_) => f.write_str("ProposalReceived"),
            IssuerFullState::OfferSent(_) => f.write_str("OfferSent"),
            IssuerFullState::RequestReceived(_) => f.write_str("RequestReceived"),
            IssuerFullState::CredentialSent(_) => f.write_str("CredentialSent"),
            IssuerFullState::Finished(_) => f.write_str("Finished"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevocationInfoV1 {
    pub cred_rev_id: Option<String>,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>,
}

impl Default for IssuerFullState {
    fn default() -> Self {
        Self::Initial(InitialIssuerState::default())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct IssuerSM {
    source_id: String,
    thread_id: String,
    state: IssuerFullState,
}

fn build_credential_message(libindy_credential: String) -> VcxResult<Credential> {
    Ok(Credential::create().set_credential(libindy_credential)?.set_out_time())
}

fn build_credential_offer(
    thread_id: &str,
    credential_offer: &str,
    credential_preview: CredentialPreviewData,
    comment: Option<String>,
) -> VcxResult<CredentialOffer> {
    Ok(CredentialOffer::create()
        .set_id(thread_id)
        .set_offers_attach(credential_offer)?
        .set_credential_preview_data(credential_preview)
        .set_comment(comment)
        .set_out_time())
}

impl IssuerSM {
    pub fn new(source_id: &str) -> Self {
        Self {
            source_id: source_id.to_string(),
            thread_id: MessageId::new().0,
            state: IssuerFullState::Initial(InitialIssuerState {}),
        }
    }

    pub fn from_proposal(source_id: &str, credential_proposal: &CredentialProposal) -> Self {
        Self {
            thread_id: credential_proposal.id.0.clone(),
            source_id: source_id.to_string(),
            state: IssuerFullState::ProposalReceived(ProposalReceivedState::new(credential_proposal.clone(), None)),
        }
    }

    pub fn get_source_id(&self) -> String {
        self.source_id.clone()
    }

    pub fn step(source_id: String, thread_id: String, state: IssuerFullState) -> Self {
        Self {
            source_id,
            thread_id,
            state,
        }
    }

    pub fn get_revocation_info(&self) -> Option<RevocationInfoV1> {
        match &self.state {
            IssuerFullState::CredentialSent(state) => state.revocation_info_v1.clone(),
            IssuerFullState::Finished(state) => state.revocation_info_v1.clone(),
            _ => None,
        }
    }

    pub fn get_rev_id(&self) -> VcxResult<String> {
        let err = AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidState,
            "No revocation info found - is this credential revokable?",
        );
        let rev_id = match &self.state {
            IssuerFullState::CredentialSent(state) => state.revocation_info_v1.as_ref().ok_or(err)?.cred_rev_id.clone(),
            IssuerFullState::Finished(state) => state.revocation_info_v1.as_ref().ok_or(err)?.cred_rev_id.clone(),
            _ => None,
        };
        rev_id.ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidState,
            "Revocation info does not contain rev id",
        ))
    }

    pub fn get_rev_reg_id(&self) -> VcxResult<String> {
        let rev_registry = match &self.state {
            IssuerFullState::Initial(_state) => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "No revocation info available in the initial state",
                ));
            }
            IssuerFullState::OfferSet(state) => state.rev_reg_id.clone(),
            IssuerFullState::ProposalReceived(state) => match &state.offer_info {
                Some(offer_info) => offer_info.rev_reg_id.clone(),
                _ => None,
            },
            IssuerFullState::OfferSent(state) => state.rev_reg_id.clone(),
            IssuerFullState::RequestReceived(state) => state.rev_reg_id.clone(),
            IssuerFullState::CredentialSent(state) => {
                state
                    .revocation_info_v1
                    .clone()
                    .ok_or(AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidState,
                        "No revocation info found - is this credential revokable?",
                    ))?
                    .rev_reg_id
            }
            IssuerFullState::Finished(state) => {
                state
                    .revocation_info_v1
                    .clone()
                    .ok_or(AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidState,
                        "No revocation info found - is this credential revokable?",
                    ))?
                    .rev_reg_id
            }
        };
        rev_registry.ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidState,
            "No revocation registry id found on revocation info - is this credential revokable?",
        ))
    }

    pub fn is_revokable(&self) -> bool {
        fn _is_revokable(rev_info: &Option<RevocationInfoV1>) -> bool {
            match rev_info {
                Some(rev_info) => rev_info.cred_rev_id.is_some(),
                None => false,
            }
        }
        match &self.state {
            IssuerFullState::CredentialSent(state) => _is_revokable(&state.revocation_info_v1),
            IssuerFullState::Finished(state) => _is_revokable(&state.revocation_info_v1),
            _ => false,
        }
    }

    pub async fn is_revoked(&self, profile: &Arc<dyn Profile>) -> VcxResult<bool> {
        if self.is_revokable() {
            let rev_reg_id = self.get_rev_reg_id()?;
            let rev_id = self.get_rev_id()?;
            is_cred_revoked(profile, &rev_reg_id, &rev_id).await
        } else {
            Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Unable to check revocation status - this credential is not revokable",
            ))
        }
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        trace!(
            "IssuerSM::find_message_to_handle >>> messages: {:?}, state: {:?}",
            messages,
            self.state
        );

        for (uid, message) in messages {
            match self.state {
                IssuerFullState::Initial(_) => {
                    if let A2AMessage::CredentialProposal(credential_proposal) = message {
                        return Some((uid, A2AMessage::CredentialProposal(credential_proposal)));
                    }
                }
                IssuerFullState::OfferSent(_) => match message {
                    A2AMessage::CredentialRequest(credential) => {
                        if credential.from_thread(&self.thread_id) {
                            return Some((uid, A2AMessage::CredentialRequest(credential)));
                        }
                    }
                    A2AMessage::CredentialProposal(credential_proposal) => {
                        if let Some(ref thread) = credential_proposal.thread {
                            if thread.is_reply(&self.thread_id) {
                                return Some((uid, A2AMessage::CredentialProposal(credential_proposal)));
                            }
                        }
                    }
                    A2AMessage::CommonProblemReport(problem_report) => {
                        if problem_report.from_thread(&self.thread_id) {
                            return Some((uid, A2AMessage::CommonProblemReport(problem_report)));
                        }
                    }
                    _ => {}
                },
                IssuerFullState::CredentialSent(_) => match message {
                    A2AMessage::Ack(ack) | A2AMessage::CredentialAck(ack) => {
                        if ack.from_thread(&self.thread_id) {
                            return Some((uid, A2AMessage::CredentialAck(ack)));
                        }
                    }
                    A2AMessage::CommonProblemReport(problem_report) => {
                        if problem_report.from_thread(&self.thread_id) {
                            return Some((uid, A2AMessage::CommonProblemReport(problem_report)));
                        }
                    }
                    _ => {}
                },
                _ => {}
            };
        }

        None
    }

    pub fn get_state(&self) -> IssuerState {
        match self.state {
            IssuerFullState::Initial(_) => IssuerState::Initial,
            IssuerFullState::ProposalReceived(_) => IssuerState::ProposalReceived,
            IssuerFullState::OfferSet(_) => IssuerState::OfferSet,
            IssuerFullState::OfferSent(_) => IssuerState::OfferSent,
            IssuerFullState::RequestReceived(_) => IssuerState::RequestReceived,
            IssuerFullState::CredentialSent(_) => IssuerState::CredentialSent,
            IssuerFullState::Finished(ref status) => match status.status {
                Status::Success => IssuerState::Finished,
                _ => IssuerState::Failed,
            },
        }
    }

    pub fn get_proposal(&self) -> VcxResult<CredentialProposal> {
        match &self.state {
            IssuerFullState::ProposalReceived(state) => Ok(state.credential_proposal.clone()),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Proposal is only available in ProposalReceived state",
            )),
        }
    }

    pub fn build_credential_offer_msg(
        self,
        credential_offer: &str,
        credential_preview: CredentialPreviewData,
        comment: Option<String>,
        offer_info: &OfferInfo,
    ) -> VcxResult<Self> {
        let Self {
            state,
            source_id,
            thread_id,
        } = self;
        let state = match state {
            IssuerFullState::Initial(_) | IssuerFullState::OfferSet(_) | IssuerFullState::ProposalReceived(_) => {
                let cred_offer_msg = build_credential_offer(&thread_id, credential_offer, credential_preview, comment)?;
                IssuerFullState::OfferSet(OfferSetState::new(
                    cred_offer_msg,
                    &offer_info.credential_json,
                    &offer_info.cred_def_id,
                    offer_info.rev_reg_id.clone(),
                    offer_info.tails_file.clone(),
                ))
            }
            _ => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    format!("Can not set_offer in current state {}.", state),
                ));
            }
        };
        Ok(Self::step(source_id, thread_id, state))
    }

    pub fn get_credential_offer_msg(&self) -> VcxResult<CredentialOffer> {
        match &self.state {
            IssuerFullState::OfferSet(state) => Ok(state.offer.clone()),
            IssuerFullState::OfferSent(state) => Ok(state.offer.clone()),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                format!("Can not get_credential_offer in current state {}.", self.state),
            )),
        }
    }

    pub fn mark_credential_offer_msg_sent(self) -> VcxResult<Self> {
        let Self {
            state,
            source_id,
            thread_id,
        } = self;
        let state = match state {
            IssuerFullState::OfferSet(state) => IssuerFullState::OfferSent(state.into()),
            IssuerFullState::OfferSent(state) => IssuerFullState::OfferSent(state),
            _ => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    format!("Can not mark_as_offer_sent in current state {}.", state),
                ))
            }
        };
        Ok(Self::step(source_id, thread_id, state))
    }

    pub fn receive_proposal(self, proposal: CredentialProposal) -> VcxResult<Self> {
        verify_thread_id(
            &self.thread_id,
            &CredentialIssuanceAction::CredentialProposal(proposal.clone()),
        )?;
        let (state, thread_id) = match self.state {
            IssuerFullState::Initial(_) => {
                let thread_id = proposal.id.0.to_string();
                let state = IssuerFullState::ProposalReceived(ProposalReceivedState::new(proposal, None));
                (state, thread_id)
            }
            IssuerFullState::OfferSent(_) => {
                verify_thread_id(
                    &self.thread_id,
                    &CredentialIssuanceAction::CredentialProposal(proposal.clone()),
                )?;
                let state = IssuerFullState::ProposalReceived(ProposalReceivedState::new(proposal, None));
                (state, self.thread_id.clone())
            }
            s => {
                warn!("Unable to receive credential proposal in state {}", s);
                (s, self.thread_id.clone())
            }
        };
        Ok(Self {
            state,
            thread_id,
            ..self
        })
    }

    pub async fn send_credential_offer(self, send_message: SendClosure) -> VcxResult<Self> {
        Ok(match self.state {
            IssuerFullState::OfferSet(ref state_data) => {
                let cred_offer_msg = state_data.offer.to_a2a_message();
                send_message(cred_offer_msg).await?;
                self.mark_credential_offer_msg_sent()?
            }
            _ => {
                return Err(AriesVcxError::from_msg(AriesVcxErrorKind::NotReady, "Invalid action"));
            }
        })
    }

    pub fn receive_request(self, request: CredentialRequest) -> VcxResult<Self> {
        verify_thread_id(
            &self.thread_id,
            &CredentialIssuanceAction::CredentialRequest(request.clone()),
        )?;
        let state = match self.state {
            IssuerFullState::OfferSent(state_data) => IssuerFullState::RequestReceived((state_data, request).into()),
            s => {
                warn!("Unable to receive credential request in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub async fn send_credential(self, profile: &Arc<dyn Profile>, send_message: SendClosure) -> VcxResult<Self> {
        let state = match self.state {
            IssuerFullState::RequestReceived(state_data) => {
                match _create_credential(
                    profile,
                    &state_data.request,
                    &state_data.rev_reg_id,
                    &state_data.tails_file,
                    &state_data.offer,
                    &state_data.cred_data,
                    &self.thread_id,
                )
                .await
                {
                    Ok((credential_msg, cred_rev_id)) => {
                        let credential_msg = credential_msg.set_thread_id(&self.thread_id).ask_for_ack(); // TODO: Make configurable
                        send_message(credential_msg.to_a2a_message()).await?;
                        IssuerFullState::CredentialSent((state_data, cred_rev_id).into())
                    }
                    Err(err) => {
                        let problem_report = build_problem_report_msg(Some(err.to_string()), &self.thread_id);
                        error!(
                            "Failed to create credential, sending problem report {:?}",
                            problem_report
                        );
                        send_message(problem_report.to_a2a_message()).await?;
                        IssuerFullState::Finished((state_data, problem_report).into())
                    }
                }
            }
            _ => {
                return Err(AriesVcxError::from_msg(AriesVcxErrorKind::NotReady, "Invalid action"));
            }
        };
        Ok(Self { state, ..self })
    }

    pub fn receive_ack(self, ack: Ack) -> VcxResult<Self> {
        verify_thread_id(&self.thread_id, &CredentialIssuanceAction::CredentialAck(ack))?;
        let state = match self.state {
            IssuerFullState::CredentialSent(state_data) => IssuerFullState::Finished(state_data.into()),
            s => {
                warn!("Unable to receive credential ack in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub fn receive_problem_report(self, problem_report: ProblemReport) -> VcxResult<Self> {
        verify_thread_id(
            &self.thread_id,
            &CredentialIssuanceAction::ProblemReport(problem_report.clone()),
        )?;
        let state = match self.state {
            IssuerFullState::OfferSent(state_data) => IssuerFullState::Finished((state_data, problem_report).into()),
            IssuerFullState::CredentialSent(state_data) => IssuerFullState::Finished((state_data).into()),
            s => {
                warn!("Unable to receive credential ack in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub async fn handle_message(
        self,
        profile: &Arc<dyn Profile>,
        cim: CredentialIssuanceAction,
        send_message: Option<SendClosure>,
    ) -> VcxResult<Self> {
        trace!("IssuerSM::handle_message >>> cim: {:?}, state: {:?}", cim, self.state);
        verify_thread_id(&self.thread_id, &cim)?;
        let issuer_sm = match cim {
            CredentialIssuanceAction::CredentialProposal(proposal) => self.receive_proposal(proposal)?,
            CredentialIssuanceAction::CredentialRequest(request) => self.receive_request(request)?,
            CredentialIssuanceAction::CredentialSend() => {
                let send_message = send_message.ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Attempted to call undefined send_message callback",
                ))?;
                self.send_credential(profile, send_message).await?
            }
            CredentialIssuanceAction::CredentialAck(ack) => self.receive_ack(ack)?,
            CredentialIssuanceAction::ProblemReport(problem_report) => self.receive_problem_report(problem_report)?,
            _ => self,
        };
        Ok(issuer_sm)
    }

    pub fn credential_status(&self) -> u32 {
        trace!("Issuer::credential_status >>>");

        match self.state {
            IssuerFullState::Finished(ref state) => state.status.code(),
            _ => Status::Undefined.code(),
        }
    }

    pub fn is_terminal_state(&self) -> bool {
        matches!(self.state, IssuerFullState::Finished(_))
    }

    pub fn thread_id(&self) -> VcxResult<String> {
        Ok(self.thread_id.clone())
    }
}

async fn _create_credential(
    profile: &Arc<dyn Profile>,
    request: &CredentialRequest,
    rev_reg_id: &Option<String>,
    tails_file: &Option<String>,
    offer: &CredentialOffer,
    cred_data: &str,
    thread_id: &str,
) -> VcxResult<(Credential, Option<String>)> {
    let anoncreds = Arc::clone(profile).inject_anoncreds();
    let offer = offer.offers_attach.content()?;
    trace!(
        "Issuer::_create_credential >>> request: {:?}, rev_reg_id: {:?}, tails_file: {:?}, offer: {}, cred_data: {}, \
         thread_id: {}",
        request,
        rev_reg_id,
        tails_file,
        offer,
        cred_data,
        thread_id
    );
    if !request.from_thread(thread_id) {
        return Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!(
                "Cannot handle credential request: thread id does not match: {:?}",
                request.thread
            ),
        ));
    };
    let request = &request.requests_attach.content()?;
    let cred_data = encode_attributes(cred_data)?;
    let (libindy_credential, cred_rev_id, _) = anoncreds
        .issuer_create_credential(&offer, request, &cred_data, rev_reg_id.clone(), tails_file.clone())
        .await?;
    let credential = build_credential_message(libindy_credential)?;
    Ok((credential, cred_rev_id))
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use messages::{
        a2a::A2AMessage,
        concepts::problem_report::ProblemReport,
        protocols::issuance::{
            credential::test_utils::_credential,
            credential_offer::test_utils::{_credential_offer, _offer_info},
            credential_proposal::test_utils::_credential_proposal,
            credential_request::test_utils::{_credential_request, _credential_request_1},
            test_utils::{_credential_ack, _problem_report},
        },
    };

    use super::*;
    use crate::{
        common::test_utils::mock_profile,
        test::source_id,
        utils::{constants::LIBINDY_CRED_OFFER, devsetup::SetupMocks},
    };

    pub fn _rev_reg_id() -> String {
        String::from("TEST_REV_REG_ID")
    }

    pub fn _tails_file() -> String {
        String::from("TEST_TAILS_FILE")
    }

    pub fn _send_message() -> Option<SendClosure> {
        Some(Box::new(|_: A2AMessage| Box::pin(async { VcxResult::Ok(()) })))
    }

    fn _issuer_sm() -> IssuerSM {
        IssuerSM::new(&source_id())
    }

    fn _issuer_sm_from_proposal() -> IssuerSM {
        IssuerSM::from_proposal(&source_id(), &_credential_proposal())
    }

    impl IssuerSM {
        fn to_proposal_received_state(self) -> IssuerSM {
            Self::from_proposal(&source_id(), &_credential_proposal())
        }

        fn to_offer_sent_state(mut self) -> IssuerSM {
            let cred_info = _offer_info();
            self = self
                .build_credential_offer_msg(
                    LIBINDY_CRED_OFFER,
                    CredentialPreviewData::new(),
                    Some("foo".into()),
                    &cred_info,
                )
                .unwrap();
            self = self.mark_credential_offer_msg_sent().unwrap();
            self
        }

        async fn to_request_received_state(mut self) -> IssuerSM {
            self = self.to_offer_sent_state();
            self = self
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialRequest(_credential_request()),
                    _send_message(),
                )
                .await
                .unwrap();
            self
        }

        async fn to_finished_state(mut self) -> IssuerSM {
            self = self.to_request_received_state().await;
            self = self
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialSend(),
                    _send_message(),
                )
                .await
                .unwrap();
            self = self
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialAck(_credential_ack()),
                    _send_message(),
                )
                .await
                .unwrap();
            self
        }
    }

    mod build_messages {
        use messages::{a2a::MessageId, protocols::issuance::CredentialPreviewData};

        use crate::{
            protocols::issuance::issuer::state_machine::{build_credential_message, build_credential_offer},
            utils::{
                constants::LIBINDY_CRED_OFFER,
                devsetup::{was_in_past, SetupMocks},
            },
        };

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_build_credential_offer() {
            let _setup = SetupMocks::init();
            let msg = build_credential_offer(
                "12345",
                LIBINDY_CRED_OFFER,
                CredentialPreviewData::new(),
                Some("foo".into()),
            )
            .unwrap();

            assert_eq!(msg.id, MessageId("12345".into()));
            assert!(msg.thread.is_none());
            assert!(was_in_past(
                &msg.timing.unwrap().out_time.unwrap(),
                chrono::Duration::milliseconds(100)
            )
            .unwrap());
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_build_credential_message() {
            let _setup = SetupMocks::init();

            let msg = build_credential_message("{}".into()).unwrap();

            assert_eq!(msg.id, MessageId::default());
            assert!(msg.thread.thid.is_none()); // todo: should have thread_id
            assert!(was_in_past(
                &msg.timing.unwrap().out_time.unwrap(),
                chrono::Duration::milliseconds(100)
            )
            .unwrap());
        }
    }

    mod new {
        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_new() {
            let _setup = SetupMocks::init();

            let issuer_sm = _issuer_sm();

            assert_match!(IssuerFullState::Initial(_), issuer_sm.state);
            assert_eq!(source_id(), issuer_sm.get_source_id());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_from_proposal() {
            let _setup = SetupMocks::init();

            let issuer_sm = _issuer_sm_from_proposal();

            assert_match!(IssuerFullState::ProposalReceived(_), issuer_sm.state);
            assert_eq!(source_id(), issuer_sm.get_source_id());
        }
    }

    mod handle_message {
        use super::*;

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_credential_proposal_message_from_initial_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialProposal(_credential_proposal()),
                    None,
                )
                .await
                .unwrap();

            assert_match!(IssuerFullState::ProposalReceived(_), issuer_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_set_credential_offer_message_in_initial_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            let _cred_offer = CredentialOffer::create().set_offers_attach(LIBINDY_CRED_OFFER).unwrap();
            let cred_info = _offer_info();
            issuer_sm = issuer_sm
                .build_credential_offer_msg(
                    LIBINDY_CRED_OFFER,
                    CredentialPreviewData::new(),
                    Some("foo".into()),
                    &cred_info,
                )
                .unwrap();
            issuer_sm = issuer_sm.mark_credential_offer_msg_sent().unwrap();

            assert_match!(IssuerFullState::OfferSent(_), issuer_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_other_messages_from_initial_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();

            issuer_sm = issuer_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::Credential(_credential()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(IssuerFullState::Initial(_), issuer_sm.state);

            issuer_sm = issuer_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialRequest(_credential_request()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(IssuerFullState::Initial(_), issuer_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_credential_offer_send_message_from_proposal_received() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm_from_proposal();
            issuer_sm = issuer_sm.to_offer_sent_state();

            assert_match!(IssuerFullState::OfferSent(_), issuer_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_other_messages_from_proposal_received_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm_from_proposal();

            issuer_sm = issuer_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::Credential(_credential()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(IssuerFullState::ProposalReceived(_), issuer_sm.state);

            issuer_sm = issuer_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialRequest(_credential_request()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(IssuerFullState::ProposalReceived(_), issuer_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_credential_request_message_from_offer_sent_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.to_offer_sent_state();
            issuer_sm = issuer_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialRequest(_credential_request()),
                    _send_message(),
                )
                .await
                .unwrap();

            assert_match!(IssuerFullState::RequestReceived(_), issuer_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_credential_proposal_message_from_offer_sent_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.to_offer_sent_state();
            issuer_sm = issuer_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialProposal(_credential_proposal()),
                    _send_message(),
                )
                .await
                .unwrap();

            assert_match!(IssuerFullState::ProposalReceived(_), issuer_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_problem_report_message_from_offer_sent_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.to_offer_sent_state();
            issuer_sm = issuer_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::ProblemReport(_problem_report()),
                    _send_message(),
                )
                .await
                .unwrap();

            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);
            assert_eq!(
                Status::Failed(ProblemReport::default()).code(),
                issuer_sm.credential_status()
            );
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_other_messages_from_offer_sent_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.to_offer_sent_state();
            issuer_sm = issuer_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::Credential(_credential()),
                    _send_message(),
                )
                .await
                .unwrap();

            assert_match!(IssuerFullState::OfferSent(_), issuer_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_credential_send_message_from_request_received_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.to_offer_sent_state();
            issuer_sm = issuer_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialRequest(_credential_request()),
                    _send_message(),
                )
                .await
                .unwrap();
            issuer_sm = issuer_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialSend(),
                    _send_message(),
                )
                .await
                .unwrap();

            assert_match!(IssuerFullState::CredentialSent(_), issuer_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_credential_send_message_from_request_received_state_with_invalid_request() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.to_offer_sent_state();
            issuer_sm = issuer_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialRequest(CredentialRequest::create()),
                    _send_message(),
                )
                .await
                .unwrap();
            issuer_sm = issuer_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialSend(),
                    _send_message(),
                )
                .await
                .unwrap();

            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);
            assert_eq!(
                Status::Failed(ProblemReport::default()).code(),
                issuer_sm.credential_status()
            );
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_other_messages_from_request_received_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.to_offer_sent_state();
            issuer_sm = issuer_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialRequest(_credential_request()),
                    _send_message(),
                )
                .await
                .unwrap();
            issuer_sm = issuer_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialSend(),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(IssuerFullState::CredentialSent(_), issuer_sm.state);
            issuer_sm = issuer_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialAck(_credential_ack()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_credential_send_fails_with_incorrect_thread_id() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.to_offer_sent_state();
            issuer_sm = issuer_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialRequest(_credential_request_1()),
                    _send_message(),
                )
                .await
                .unwrap();
            issuer_sm = issuer_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialSend(),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);
            assert_eq!(
                Status::Failed(ProblemReport::default()).code(),
                issuer_sm.credential_status()
            );
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_messages_from_finished_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.to_finished_state().await;
            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);

            issuer_sm = issuer_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialRequest(_credential_request()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);

            issuer_sm = issuer_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::Credential(_credential()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_in_finished_state_returns_error_on_set_offer() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.to_finished_state().await;
            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);
            let _cred_offer = CredentialOffer::create().set_offers_attach(LIBINDY_CRED_OFFER).unwrap();
            let cred_info = _offer_info();

            let res1 = issuer_sm.build_credential_offer_msg(
                LIBINDY_CRED_OFFER,
                CredentialPreviewData::new(),
                Some("foo".into()),
                &cred_info,
            );
            assert!(res1.is_err());
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_in_finished_state_returns_error_on_mark_credential_offer_msg_sent() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.to_finished_state().await;
            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);

            let res1 = issuer_sm.mark_credential_offer_msg_sent();
            assert!(res1.is_err());
        }
    }

    mod find_message_to_handle {
        use super::*;

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_find_message_to_handle_from_initial_state() {
            let _setup = SetupMocks::init();

            let issuer = _issuer_sm();

            let messages = map!(
                "key_1".to_string() => A2AMessage::CredentialProposal(_credential_proposal()),
                "key_2".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                "key_3".to_string() => A2AMessage::CredentialRequest(_credential_request()),
                "key_4".to_string() => A2AMessage::Credential(_credential()),
                "key_5".to_string() => A2AMessage::CredentialAck(_credential_ack()),
                "key_6".to_string() => A2AMessage::CommonProblemReport(_problem_report())
            );
            let (uid, message) = issuer.find_message_to_handle(messages).unwrap();
            assert_eq!("key_1", uid);
            assert_match!(A2AMessage::CredentialProposal(_), message);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_find_message_to_handle_from_offer_sent_state() {
            let _setup = SetupMocks::init();

            let issuer = _issuer_sm().to_offer_sent_state();

            // CredentialRequest
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                    "key_2".to_string() => A2AMessage::Credential(_credential()),
                    "key_3".to_string() => A2AMessage::CredentialRequest(_credential_request())
                );

                let (uid, message) = issuer.find_message_to_handle(messages).unwrap();
                assert_eq!("key_3", uid);
                assert_match!(A2AMessage::CredentialRequest(_), message);
            }

            // CredentialProposal
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                    "key_2".to_string() => A2AMessage::CredentialAck(_credential_ack()),
                    "key_3".to_string() => A2AMessage::Credential(_credential()),
                    "key_4".to_string() => A2AMessage::CredentialProposal(_credential_proposal())
                );

                let (uid, message) = issuer.find_message_to_handle(messages).unwrap();
                assert_eq!("key_4", uid);
                assert_match!(A2AMessage::CredentialProposal(_), message);
            }

            // Problem Report
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                    "key_2".to_string() => A2AMessage::CredentialAck(_credential_ack()),
                    "key_3".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                let (uid, message) = issuer.find_message_to_handle(messages).unwrap();
                assert_eq!("key_3", uid);
                assert_match!(A2AMessage::CommonProblemReport(_), message);
            }

            // No messages for different Thread ID
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer().set_thread_id("")),
                    "key_2".to_string() => A2AMessage::CredentialRequest(_credential_request().set_thread_id("")),
                    "key_3".to_string() => A2AMessage::CredentialProposal(_credential_proposal().set_thread_id("")),
                    "key_4".to_string() => A2AMessage::Credential(_credential().set_thread_id("")),
                    "key_5".to_string() => A2AMessage::CredentialAck(_credential_ack().set_thread_id("")),
                    "key_6".to_string() => A2AMessage::CommonProblemReport(_problem_report().set_thread_id(""))
                );

                assert!(issuer.find_message_to_handle(messages).is_none());
            }

            // No messages
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                    "key_2".to_string() => A2AMessage::CredentialAck(_credential_ack())
                );

                assert!(issuer.find_message_to_handle(messages).is_none());
            }
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_find_message_to_handle_from_request_state() {
            let _setup = SetupMocks::init();

            let issuer = _issuer_sm().to_finished_state().await;

            // No messages
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                    "key_2".to_string() => A2AMessage::CredentialRequest(_credential_request()),
                    "key_3".to_string() => A2AMessage::CredentialProposal(_credential_proposal()),
                    "key_4".to_string() => A2AMessage::Credential(_credential()),
                    "key_5".to_string() => A2AMessage::CredentialAck(_credential_ack()),
                    "key_6".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                assert!(issuer.find_message_to_handle(messages).is_none());
            }
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_find_message_to_handle_from_credential_sent_state() {
            let _setup = SetupMocks::init();

            let issuer = _issuer_sm().to_finished_state().await;

            // No messages
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                    "key_2".to_string() => A2AMessage::CredentialRequest(_credential_request()),
                    "key_3".to_string() => A2AMessage::CredentialProposal(_credential_proposal()),
                    "key_4".to_string() => A2AMessage::Credential(_credential()),
                    "key_5".to_string() => A2AMessage::CredentialAck(_credential_ack()),
                    "key_6".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                assert!(issuer.find_message_to_handle(messages).is_none());
            }
        }
    }

    mod get_state {
        use super::*;

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_get_state() {
            let _setup = SetupMocks::init();

            assert_eq!(IssuerState::Initial, _issuer_sm().get_state());
            assert_eq!(
                IssuerState::ProposalReceived,
                _issuer_sm().to_proposal_received_state().get_state()
            );
            assert_eq!(IssuerState::OfferSent, _issuer_sm().to_offer_sent_state().get_state());
            assert_eq!(
                IssuerState::RequestReceived,
                _issuer_sm().to_request_received_state().await.get_state()
            );
            assert_eq!(
                IssuerState::Finished,
                _issuer_sm().to_finished_state().await.get_state()
            );
        }
    }

    mod get_rev_reg_id {
        use super::*;

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_get_rev_reg_id() {
            let _setup = SetupMocks::init();

            assert_eq!(
                AriesVcxErrorKind::InvalidState,
                _issuer_sm().get_rev_reg_id().unwrap_err().kind()
            );
            assert_eq!(
                AriesVcxErrorKind::InvalidState,
                _issuer_sm()
                    .to_proposal_received_state()
                    .get_rev_reg_id()
                    .unwrap_err()
                    .kind()
            );
            assert_eq!(
                _rev_reg_id(),
                _issuer_sm().to_offer_sent_state().get_rev_reg_id().unwrap()
            );
            assert_eq!(
                _rev_reg_id(),
                _issuer_sm().to_request_received_state().await.get_rev_reg_id().unwrap()
            );
            assert_eq!(
                _rev_reg_id(),
                _issuer_sm().to_finished_state().await.get_rev_reg_id().unwrap()
            );
        }
    }

    mod is_revokable {
        use super::*;

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_is_revokable() {
            let _setup = SetupMocks::init();

            assert_eq!(false, _issuer_sm().is_revokable());
            assert_eq!(false, _issuer_sm().to_proposal_received_state().is_revokable());
            assert_eq!(false, _issuer_sm().to_offer_sent_state().is_revokable());
            assert_eq!(false, _issuer_sm().to_request_received_state().await.is_revokable());
            assert_eq!(false, _issuer_sm().to_finished_state().await.is_revokable());
        }
    }
}
