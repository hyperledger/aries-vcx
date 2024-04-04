use std::{fmt::Display, path::Path};

use anoncreds_types::data_types::messages::credential::Credential;
use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_ledger::ledger::base_ledger::AnoncredsLedgerRead;
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;
use chrono::Utc;
use messages::{
    decorators::{please_ack::PleaseAck, thread::Thread, timing::Timing},
    msg_fields::protocols::{
        cred_issuance::{
            v1::{
                ack::AckCredentialV1,
                issue_credential::{
                    IssueCredentialV1, IssueCredentialV1Content, IssueCredentialV1Decorators,
                },
                offer_credential::{
                    OfferCredentialV1, OfferCredentialV1Content, OfferCredentialV1Decorators,
                },
                propose_credential::ProposeCredentialV1,
                request_credential::RequestCredentialV1,
                CredentialIssuanceV1, CredentialPreviewV1,
            },
            CredentialIssuance,
        },
        report_problem::ProblemReport,
    },
    AriesMessage,
};
use uuid::Uuid;

use crate::{
    common::credentials::{encoding::encode_attributes, is_cred_revoked},
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    handlers::util::{
        get_attach_as_string, make_attach_from_str, matches_opt_thread_id, verify_thread_id,
        AttachmentId, OfferInfo, Status,
    },
    protocols::{
        common::build_problem_report_msg,
        issuance::issuer::states::{
            credential_set::CredentialSetState, finished::FinishedState,
            initial::InitialIssuerState, offer_set::OfferSetState,
            proposal_received::ProposalReceivedState, requested_received::RequestReceivedState,
        },
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum IssuerFullState {
    Initial(InitialIssuerState),
    OfferSet(OfferSetState),
    ProposalReceived(ProposalReceivedState),
    RequestReceived(RequestReceivedState),
    CredentialSet(CredentialSetState),
    Finished(FinishedState),
}

#[derive(Debug, PartialEq, Eq)]
pub enum IssuerState {
    Initial,
    OfferSet,
    ProposalReceived,
    RequestReceived,
    CredentialSet,
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
            IssuerFullState::RequestReceived(_) => f.write_str("RequestReceived"),
            IssuerFullState::CredentialSet(_) => f.write_str("CredentialSet"),
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

fn build_credential_message(
    libindy_credential: Credential,
    thread_id: String,
) -> IssueCredentialV1 {
    let id = Uuid::new_v4().to_string();

    let content = IssueCredentialV1Content::builder()
        .credentials_attach(vec![make_attach_from_str!(
            &serde_json::to_string(&libindy_credential).unwrap(),
            AttachmentId::Credential.as_ref().to_string()
        )])
        .build();

    let decorators = IssueCredentialV1Decorators::builder()
        .thread(Thread::builder().thid(thread_id).build())
        .please_ack(PleaseAck::builder().on(vec![]).build())
        .build();

    IssueCredentialV1::builder()
        .id(id)
        .content(content)
        .decorators(decorators)
        .build()
}

fn build_credential_offer(
    thread_id: &str,
    credential_offer: &str,
    credential_preview: CredentialPreviewV1,
    comment: Option<String>,
) -> VcxResult<OfferCredentialV1> {
    let id = thread_id.to_owned();

    let content = OfferCredentialV1Content::builder()
        .credential_preview(credential_preview)
        .offers_attach(vec![make_attach_from_str!(
            &credential_offer,
            AttachmentId::CredentialOffer.as_ref().to_string()
        )]);

    let content = if let Some(comment) = comment {
        content.comment(comment).build()
    } else {
        content.build()
    };

    let decorators = OfferCredentialV1Decorators::builder()
        .timing(Timing::builder().out_time(Utc::now()).build())
        .build();

    Ok(OfferCredentialV1::builder()
        .id(id)
        .content(content)
        .decorators(decorators)
        .build())
}

impl IssuerSM {
    pub fn new(source_id: &str) -> Self {
        Self {
            source_id: source_id.to_string(),
            thread_id: Uuid::new_v4().to_string(),
            state: IssuerFullState::Initial(InitialIssuerState {}),
        }
    }

    pub fn from_proposal(source_id: &str, credential_proposal: &ProposeCredentialV1) -> Self {
        Self {
            thread_id: credential_proposal.id.clone(),
            source_id: source_id.to_string(),
            state: IssuerFullState::ProposalReceived(ProposalReceivedState::new(
                credential_proposal.clone(),
                None,
            )),
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
            IssuerFullState::CredentialSet(state) => state.revocation_info_v1.clone(),
            IssuerFullState::Finished(state) => state.revocation_info_v1.clone(),
            _ => None,
        }
    }

    pub fn get_rev_id(&self) -> VcxResult<u32> {
        let err = AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidState,
            "No revocation info found - is this credential revokable?",
        );
        let rev_id = match &self.state {
            IssuerFullState::CredentialSet(state) => state
                .revocation_info_v1
                .as_ref()
                .ok_or(err)?
                .cred_rev_id
                .to_owned(),
            IssuerFullState::Finished(state) => state
                .revocation_info_v1
                .as_ref()
                .ok_or(err)?
                .cred_rev_id
                .to_owned(),
            _ => None,
        };
        rev_id
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Revocation info does not contain rev id",
            ))
            .and_then(|s| s.parse().map_err(Into::into))
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
            IssuerFullState::RequestReceived(state) => state.rev_reg_id.clone(),
            IssuerFullState::CredentialSet(state) => {
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
            IssuerFullState::CredentialSet(state) => _is_revokable(&state.revocation_info_v1),
            IssuerFullState::Finished(state) => _is_revokable(&state.revocation_info_v1),
            _ => false,
        }
    }

    pub async fn is_revoked(&self, ledger: &impl AnoncredsLedgerRead) -> VcxResult<bool> {
        if self.is_revokable() {
            let rev_reg_id = self.get_rev_reg_id()?;
            let rev_id = self.get_rev_id()?;
            is_cred_revoked(ledger, &rev_reg_id, rev_id).await
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
            IssuerFullState::RequestReceived(_) => IssuerState::RequestReceived,
            IssuerFullState::CredentialSet(_) => IssuerState::CredentialSet,
            IssuerFullState::Finished(ref status) => match status.status {
                Status::Success => IssuerState::Finished,
                _ => IssuerState::Failed,
            },
        }
    }

    pub fn get_proposal(&self) -> VcxResult<ProposeCredentialV1> {
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
        credential_preview: CredentialPreviewV1,
        comment: Option<String>,
        offer_info: &OfferInfo,
    ) -> VcxResult<Self> {
        let Self {
            state,
            source_id,
            thread_id,
        } = self;
        let state = match state {
            IssuerFullState::Initial(_)
            | IssuerFullState::OfferSet(_)
            | IssuerFullState::ProposalReceived(_) => {
                let cred_offer_msg = build_credential_offer(
                    &thread_id,
                    credential_offer,
                    credential_preview,
                    comment,
                )?;
                IssuerFullState::OfferSet(OfferSetState::new(
                    cred_offer_msg,
                    &offer_info.credential_json,
                    offer_info.cred_def_id.clone(),
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

    pub fn get_credential_offer_msg(&self) -> VcxResult<OfferCredentialV1> {
        match &self.state {
            IssuerFullState::OfferSet(state) => Ok(state.offer.clone()),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                format!(
                    "Can not get_credential_offer in current state {}.",
                    self.state
                ),
            )),
        }
    }

    pub fn receive_proposal(self, proposal: ProposeCredentialV1) -> VcxResult<Self> {
        verify_thread_id(
            &self.thread_id,
            &AriesMessage::CredentialIssuance(CredentialIssuance::V1(
                CredentialIssuanceV1::ProposeCredential(proposal.clone()),
            )),
        )?;
        let (state, thread_id) = match self.state {
            IssuerFullState::Initial(_) => {
                let thread_id = proposal.id.to_string();
                let state =
                    IssuerFullState::ProposalReceived(ProposalReceivedState::new(proposal, None));
                (state, thread_id)
            }
            IssuerFullState::OfferSet(_) => {
                let state =
                    IssuerFullState::ProposalReceived(ProposalReceivedState::new(proposal, None));
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

    pub fn receive_request(self, request: RequestCredentialV1) -> VcxResult<Self> {
        verify_thread_id(
            &self.thread_id,
            &AriesMessage::CredentialIssuance(CredentialIssuance::V1(
                CredentialIssuanceV1::RequestCredential(request.clone()),
            )),
        )?;
        let state = match self.state {
            IssuerFullState::OfferSet(state_data) => IssuerFullState::RequestReceived(
                RequestReceivedState::from_offer_set_and_request(state_data, request),
            ),
            s => {
                warn!("Unable to receive credential request in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub async fn build_credential(
        self,
        wallet: &impl BaseWallet,
        anoncreds: &impl BaseAnonCreds,
    ) -> VcxResult<Self> {
        let state = match self.state {
            IssuerFullState::RequestReceived(state_data) => {
                match create_credential(
                    wallet,
                    anoncreds,
                    &state_data.request,
                    &state_data.rev_reg_id,
                    &state_data.tails_file,
                    &state_data.offer,
                    &state_data.cred_data,
                    self.thread_id.clone(),
                )
                .await
                {
                    Ok((msg_issue_credential, cred_rev_id)) => {
                        // todo: have constructor for this
                        IssuerFullState::CredentialSet(CredentialSetState {
                            msg_issue_credential,
                            revocation_info_v1: Some(RevocationInfoV1 {
                                cred_rev_id: cred_rev_id.as_ref().map(ToString::to_string),
                                rev_reg_id: state_data.rev_reg_id,
                                tails_file: state_data.tails_file,
                            }),
                        })
                    }
                    // todo: 1. Don't transition, throw error, add to_failed transition() api which
                    // SM consumer can call       2. Also create separate
                    // "Failed" state
                    Err(err) => {
                        let problem_report =
                            build_problem_report_msg(Some(err.to_string()), &self.thread_id);
                        error!(
                            "Failed to create credential, generated problem report \
                             {problem_report:?}",
                        );
                        IssuerFullState::Finished(FinishedState::from_request_and_error(
                            state_data,
                            problem_report,
                        ))
                    }
                }
            }
            _ => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::NotReady,
                    "Invalid action",
                ));
            }
        };
        Ok(Self { state, ..self })
    }

    pub fn get_msg_issue_credential(self) -> VcxResult<IssueCredentialV1> {
        match self.state {
            IssuerFullState::CredentialSet(ref state_data) => {
                let mut msg_issue_credential: IssueCredentialV1 =
                    state_data.msg_issue_credential.clone();
                let timing = Timing::builder().out_time(Utc::now()).build();

                msg_issue_credential.decorators.timing = Some(timing);
                Ok(msg_issue_credential)
            }
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Invalid action",
            )),
        }
    }

    pub fn receive_ack(self, ack: AckCredentialV1) -> VcxResult<Self> {
        verify_thread_id(
            &self.thread_id,
            &AriesMessage::CredentialIssuance(CredentialIssuance::V1(CredentialIssuanceV1::Ack(
                ack,
            ))),
        )?;
        let state = match self.state {
            IssuerFullState::CredentialSet(state_data) => {
                IssuerFullState::Finished(FinishedState::from_credential_set_state(state_data))
            }
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
            &AriesMessage::ReportProblem(problem_report.clone()),
        )?;
        let state = match self.state {
            IssuerFullState::OfferSet(state_data) => IssuerFullState::Finished(
                FinishedState::from_offer_set_and_error(state_data, problem_report),
            ),
            IssuerFullState::CredentialSet(state_data) => {
                IssuerFullState::Finished(FinishedState::from_credential_set_state(state_data))
            }
            s => {
                warn!("Unable to receive credential ack in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
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

    pub fn get_problem_report(&self) -> VcxResult<ProblemReport> {
        match self.state {
            IssuerFullState::Finished(ref state) => match &state.status {
                Status::Failed(problem_report) => Ok(problem_report.clone()),
                Status::Declined(problem_report) => Ok(problem_report.clone()),
                _ => Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::NotReady,
                    "No problem report available in current state",
                )),
            },
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "No problem report available in current state",
            )),
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn create_credential(
    wallet: &impl BaseWallet,
    anoncreds: &impl BaseAnonCreds,
    request: &RequestCredentialV1,
    rev_reg_id: &Option<String>,
    tails_file: &Option<String>,
    offer: &OfferCredentialV1,
    cred_data: &str,
    thread_id: String,
) -> VcxResult<(IssueCredentialV1, Option<u32>)> {
    let offer = get_attach_as_string!(&offer.content.offers_attach);

    trace!(
        "Issuer::_create_credential >>> request: {:?}, rev_reg_id: {:?}, tails_file: {:?}, offer: \
         {}, cred_data: {}, thread_id: {}",
        request,
        rev_reg_id,
        tails_file,
        offer,
        cred_data,
        thread_id
    );
    if !matches_opt_thread_id!(request, thread_id.as_str()) {
        return Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            "Cannot handle credential request: thread id does not match",
        ));
    };

    let request = get_attach_as_string!(&request.content.requests_attach);

    let cred_data = encode_attributes(cred_data)?;
    let (libindy_credential, cred_rev_id) = anoncreds
        .issuer_create_credential(
            wallet,
            serde_json::from_str(&offer)?,
            serde_json::from_str(&request)?,
            serde_json::from_str(&cred_data)?,
            rev_reg_id
                .to_owned()
                .map(TryInto::try_into)
                .transpose()?
                .as_ref(),
            tails_file.clone().as_deref().map(Path::new),
        )
        .await?;
    let msg_issue_credential = build_credential_message(libindy_credential, thread_id);
    Ok((msg_issue_credential, cred_rev_id))
}
