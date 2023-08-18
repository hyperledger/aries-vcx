use std::fmt::Display;
use std::sync::Arc;

use crate::handlers::util::{
    AttachmentId, get_attach_as_string, make_attach_from_str, matches_opt_thread_id, OfferInfo,
    Status,
};
use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::ledger::base_ledger::AnoncredsLedgerRead;
use chrono::Utc;
use messages::decorators::please_ack::PleaseAck;
use messages::decorators::thread::Thread;
use messages::decorators::timing::Timing;
use messages::msg_fields::protocols::cred_issuance::ack::AckCredential;
use messages::msg_fields::protocols::cred_issuance::issue_credential::{
    IssueCredential, IssueCredentialContent, IssueCredentialDecorators,
};
use messages::msg_fields::protocols::cred_issuance::offer_credential::{
    OfferCredential, OfferCredentialContent, OfferCredentialDecorators,
};
use messages::msg_fields::protocols::cred_issuance::propose_credential::ProposeCredential;
use messages::msg_fields::protocols::cred_issuance::request_credential::RequestCredential;
use messages::msg_fields::protocols::cred_issuance::CredentialPreview;
use messages::msg_fields::protocols::report_problem::ProblemReport;
use uuid::Uuid;

use crate::common::credentials::encoding::encode_attributes;
use crate::common::credentials::is_cred_revoked;
use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};
use crate::protocols::common::build_problem_report_msg;
use crate::protocols::issuance::actions::CredentialIssuanceAction;
use crate::protocols::issuance::issuer::states::credential_sent::CredentialSentState;
use crate::protocols::issuance::issuer::states::finished::FinishedState;
use crate::protocols::issuance::issuer::states::initial::InitialIssuerState;
use crate::protocols::issuance::issuer::states::offer_sent::OfferSentState;
use crate::protocols::issuance::issuer::states::offer_set::OfferSetState;
use crate::protocols::issuance::issuer::states::proposal_received::ProposalReceivedState;
use crate::protocols::issuance::issuer::states::requested_received::RequestReceivedState;
use crate::protocols::issuance::verify_thread_id;
use crate::protocols::SendClosure;

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
    pub(crate) source_id: String,
    pub(crate) thread_id: String,
    pub(crate) state: IssuerFullState,
}

fn build_credential_message(libindy_credential: String) -> VcxResult<IssueCredential> {
    let id = Uuid::new_v4().to_string();

    let content = IssueCredentialContent::new(vec![make_attach_from_str!(
        &libindy_credential,
        AttachmentId::Credential.as_ref().to_string()
    )]);

    let mut decorators = IssueCredentialDecorators::new(Thread::new(id.clone())); // this needs a Thread per RFC...
    let mut timing = Timing::default();
    timing.out_time = Some(Utc::now());
    decorators.timing = Some(timing);

    Ok(IssueCredential::with_decorators(id, content, decorators))
}

fn build_credential_offer(
    thread_id: &str,
    credential_offer: &str,
    credential_preview: CredentialPreview,
    comment: Option<String>,
) -> VcxResult<OfferCredential> {
    let id = thread_id.to_owned();

    let mut content = OfferCredentialContent::new(
        credential_preview,
        vec![make_attach_from_str!(
            &credential_offer,
            AttachmentId::CredentialOffer.as_ref().to_string()
        )],
    );
    content.comment = comment;

    let mut decorators = OfferCredentialDecorators::default();
    let mut timing = Timing::default();
    timing.out_time = Some(Utc::now());
    decorators.timing = Some(timing);

    Ok(OfferCredential::with_decorators(id, content, decorators))
}

impl IssuerSM {
    pub fn new(source_id: &str) -> Self {
        Self {
            source_id: source_id.to_string(),
            thread_id: Uuid::new_v4().to_string(),
            state: IssuerFullState::Initial(InitialIssuerState {}),
        }
    }

    pub fn from_proposal(source_id: &str, credential_proposal: &ProposeCredential) -> Self {
        Self {
            thread_id: credential_proposal.id.clone(),
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

    pub async fn is_revoked(&self, ledger: &Arc<dyn AnoncredsLedgerRead>) -> VcxResult<bool> {
        if self.is_revokable() {
            let rev_reg_id = self.get_rev_reg_id()?;
            let rev_id = self.get_rev_id()?;
            is_cred_revoked(ledger, &rev_reg_id, &rev_id).await
        } else {
            Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Unable to check revocation status - this credential is not revokable",
            ))
        }
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

    pub fn get_proposal(&self) -> VcxResult<ProposeCredential> {
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
        credential_preview: CredentialPreview,
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

    pub fn get_credential_offer_msg(&self) -> VcxResult<OfferCredential> {
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

    pub fn receive_proposal(self, proposal: ProposeCredential) -> VcxResult<Self> {
        verify_thread_id(
            &self.thread_id,
            &CredentialIssuanceAction::CredentialProposal(proposal.clone()),
        )?;
        let (state, thread_id) = match self.state {
            IssuerFullState::Initial(_) => {
                let thread_id = proposal.id.to_string();
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
                let cred_offer_msg = state_data.offer.clone().into();
                send_message(cred_offer_msg).await?;
                self.mark_credential_offer_msg_sent()?
            }
            _ => {
                return Err(AriesVcxError::from_msg(AriesVcxErrorKind::NotReady, "Invalid action"));
            }
        })
    }

    pub fn receive_request(self, request: RequestCredential) -> VcxResult<Self> {
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

    pub async fn send_credential(
        self,
        anoncreds: &Arc<dyn BaseAnonCreds>,
        send_message: SendClosure,
    ) -> VcxResult<Self> {
        let state = match self.state {
            IssuerFullState::RequestReceived(state_data) => {
                match _create_credential(
                    anoncreds,
                    &state_data.request,
                    &state_data.rev_reg_id,
                    &state_data.tails_file,
                    &state_data.offer,
                    &state_data.cred_data,
                    &self.thread_id,
                )
                .await
                {
                    Ok((mut credential_msg, cred_rev_id)) => {
                        credential_msg.decorators.thread.thid = self.thread_id.clone();
                        credential_msg.decorators.please_ack = Some(PleaseAck::new(vec![])); // ask_for_ack sets this to an empty vec

                        send_message(credential_msg.into()).await?;
                        IssuerFullState::CredentialSent((state_data, cred_rev_id).into())
                    }
                    Err(err) => {
                        let problem_report = build_problem_report_msg(Some(err.to_string()), &self.thread_id);
                        error!(
                            "Failed to create credential, sending problem report {:?}",
                            problem_report
                        );
                        send_message(problem_report.clone().into()).await?;
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

    pub fn receive_ack(self, ack: AckCredential) -> VcxResult<Self> {
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
        anoncreds: &Arc<dyn BaseAnonCreds>,
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
                self.send_credential(anoncreds, send_message).await?
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
    anoncreds: &Arc<dyn BaseAnonCreds>,
    request: &RequestCredential,
    rev_reg_id: &Option<String>,
    tails_file: &Option<String>,
    offer: &OfferCredential,
    cred_data: &str,
    thread_id: &str,
) -> VcxResult<(IssueCredential, Option<String>)> {
    let offer = get_attach_as_string!(&offer.content.offers_attach);

    trace!("Issuer::_create_credential >>> request: {:?}, rev_reg_id: {:?}, tails_file: {:?}, offer: {}, cred_data: {}, thread_id: {}", request, rev_reg_id, tails_file, offer, cred_data, thread_id);
    if !matches_opt_thread_id!(request, thread_id) {
        return Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!("Cannot handle credential request: thread id does not match"),
        ));
    };

    let request = get_attach_as_string!(&request.content.requests_attach);

    let cred_data = encode_attributes(cred_data)?;
    let (libindy_credential, cred_rev_id, _) = anoncreds
        .issuer_create_credential(&offer, &request, &cred_data, rev_reg_id.clone(), tails_file.clone())
        .await?;
    let credential = build_credential_message(libindy_credential)?;
    Ok((credential, cred_rev_id))
}
