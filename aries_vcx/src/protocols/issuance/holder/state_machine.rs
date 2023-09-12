use std::fmt;
use std::sync::Arc;

use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::ledger::base_ledger::AnoncredsLedgerRead;
use chrono::Utc;
use messages::decorators::thread::Thread;
use messages::decorators::timing::Timing;
use messages::msg_fields::protocols::cred_issuance::issue_credential::IssueCredential;
use messages::msg_fields::protocols::cred_issuance::offer_credential::OfferCredential;
use messages::msg_fields::protocols::cred_issuance::propose_credential::ProposeCredential;
use messages::msg_fields::protocols::cred_issuance::request_credential::{
    RequestCredential, RequestCredentialContent, RequestCredentialDecorators,
};
use messages::msg_fields::protocols::cred_issuance::CredentialIssuance;
use messages::msg_fields::protocols::report_problem::ProblemReport;
use messages::AriesMessage;
use uuid::Uuid;

use crate::common::credentials::{get_cred_rev_id, is_cred_revoked};
use crate::errors::error::prelude::*;
use crate::global::settings;
use crate::handlers::util::{get_attach_as_string, make_attach_from_str, verify_thread_id, AttachmentId, Status};
use crate::protocols::common::build_problem_report_msg;
use crate::protocols::issuance::holder::states::finished::FinishedHolderState;
use crate::protocols::issuance::holder::states::initial::InitialHolderState;
use crate::protocols::issuance::holder::states::offer_received::OfferReceivedState;
use crate::protocols::issuance::holder::states::proposal_set::ProposalSetState;
use crate::protocols::issuance::holder::states::request_set::RequestSetState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HolderFullState {
    Initial(InitialHolderState),
    ProposalSet(ProposalSetState),
    OfferReceived(OfferReceivedState),
    RequestSet(RequestSetState),
    Finished(FinishedHolderState),
}

#[derive(Debug, PartialEq, Eq)]
pub enum HolderState {
    Initial,
    ProposalSet,
    OfferReceived,
    RequestSet,
    Finished,
    Failed,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HolderSM {
    pub(crate) state: HolderFullState,
    pub(crate) source_id: String,
    pub(crate) thread_id: String,
}

impl fmt::Display for HolderFullState {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            HolderFullState::Initial(_) => f.write_str("Initial"),
            HolderFullState::ProposalSet(_) => f.write_str("ProposalSet"),
            HolderFullState::OfferReceived(_) => f.write_str("OfferReceived"),
            HolderFullState::RequestSet(_) => f.write_str("RequestSet"),
            HolderFullState::Finished(_) => f.write_str("Finished"),
        }
    }
}

fn _build_credential_request_msg(credential_request_attach: String, thread_id: &str) -> RequestCredential {
    let content = RequestCredentialContent::builder()
        .requests_attach(vec![make_attach_from_str!(
            &credential_request_attach,
            AttachmentId::CredentialRequest.as_ref().to_string()
        )])
        .build();

    let decorators = RequestCredentialDecorators::builder()
        .thread(Thread::builder().thid(thread_id.to_owned()).build())
        .timing(Timing::builder().out_time(Utc::now()).build())
        .build();

    RequestCredential::builder()
        .id(Uuid::new_v4().to_string())
        .content(content)
        .decorators(decorators)
        .build()
}

impl HolderSM {
    pub fn new(source_id: String) -> Self {
        HolderSM {
            thread_id: Uuid::new_v4().to_string(),
            state: HolderFullState::Initial(InitialHolderState::new()),
            source_id,
        }
    }

    pub fn from_offer(offer: OfferCredential, source_id: String) -> Self {
        HolderSM {
            thread_id: offer.id.clone(),
            state: HolderFullState::OfferReceived(OfferReceivedState::new(offer)),
            source_id,
        }
    }

    pub fn with_proposal(propose_credential: ProposeCredential, source_id: String) -> Self {
        HolderSM {
            thread_id: propose_credential.id.clone(),
            state: HolderFullState::ProposalSet(ProposalSetState::new(propose_credential)),
            source_id,
        }
    }

    pub fn get_source_id(&self) -> String {
        self.source_id.clone()
    }

    pub fn get_state(&self) -> HolderState {
        match self.state {
            HolderFullState::Initial(_) => HolderState::Initial,
            HolderFullState::ProposalSet(_) => HolderState::ProposalSet,
            HolderFullState::OfferReceived(_) => HolderState::OfferReceived,
            HolderFullState::RequestSet(_) => HolderState::RequestSet,
            HolderFullState::Finished(ref status) => match status.status {
                Status::Success => HolderState::Finished,
                _ => HolderState::Failed,
            },
        }
    }

    #[allow(dead_code)]
    pub fn get_proposal(&self) -> VcxResult<ProposeCredential> {
        match &self.state {
            HolderFullState::ProposalSet(state) => Ok(state.credential_proposal.clone()),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Proposal not available in this state",
            )),
        }
    }

    pub fn set_proposal(self, proposal: ProposeCredential) -> VcxResult<Self> {
        trace!("HolderSM::set_proposal >>");
        verify_thread_id(
            &self.thread_id,
            &AriesMessage::CredentialIssuance(CredentialIssuance::ProposeCredential(proposal.clone())),
        )?;
        let state = match self.state {
            HolderFullState::Initial(_) => {
                let mut proposal = proposal;
                proposal.id = self.thread_id.clone();
                HolderFullState::ProposalSet(ProposalSetState::new(proposal))
            }
            HolderFullState::OfferReceived(_) => {
                let mut proposal = proposal;
                proposal.id = self.thread_id.clone();
                HolderFullState::ProposalSet(ProposalSetState::new(proposal))
            }
            s => {
                warn!("Unable to set credential proposal in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub fn receive_offer(self, offer: OfferCredential) -> VcxResult<Self> {
        trace!("HolderSM::receive_offer >>");
        verify_thread_id(
            &self.thread_id,
            &AriesMessage::CredentialIssuance(CredentialIssuance::OfferCredential(offer.clone())),
        )?;
        let state = match self.state {
            HolderFullState::ProposalSet(_) => HolderFullState::OfferReceived(OfferReceivedState::new(offer)),
            s => {
                warn!("Unable to receive credential offer in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub async fn prepare_credential_request<'a>(
        self,
        ledger: &'a Arc<dyn AnoncredsLedgerRead>,
        anoncreds: &'a Arc<dyn BaseAnonCreds>,
        my_pw_did: String,
    ) -> VcxResult<Self> {
        trace!("HolderSM::prepare_credential_request >>");
        let state = match self.state {
            HolderFullState::OfferReceived(state_data) => {
                match build_credential_request_msg(
                    ledger,
                    anoncreds,
                    self.thread_id.clone(),
                    my_pw_did,
                    &state_data.offer,
                )
                .await
                {
                    Ok((msg_credential_request, req_meta, cred_def_json)) => {
                        HolderFullState::RequestSet(RequestSetState {
                            msg_credential_request,
                            req_meta,
                            cred_def_json,
                        })
                    }
                    Err(err) => {
                        let problem_report = build_problem_report_msg(Some(err.to_string()), &self.thread_id);
                        error!(
                            "Failed to create credential request, generating problem report: {:?}",
                            problem_report
                        );
                        HolderFullState::Finished(FinishedHolderState::new(problem_report))
                    }
                }
            }
            s => {
                warn!("Unable to set credential request in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub fn decline_offer(self, comment: Option<String>) -> VcxResult<Self> {
        trace!("HolderSM::decline_offer >>");
        let state = match self.state {
            HolderFullState::OfferReceived(_) => {
                let problem_report = build_problem_report_msg(comment, &self.thread_id);
                HolderFullState::Finished(FinishedHolderState::new(problem_report))
            }
            s => {
                warn!("Unable to decline credential offer in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub async fn receive_credential<'a>(
        self,
        ledger: &'a Arc<dyn AnoncredsLedgerRead>,
        anoncreds: &'a Arc<dyn BaseAnonCreds>,
        credential: IssueCredential,
    ) -> VcxResult<Self> {
        trace!("HolderSM::receive_credential >>");
        let state = match self.state {
            HolderFullState::RequestSet(state_data) => {
                match _store_credential(
                    ledger,
                    anoncreds,
                    &credential,
                    &state_data.req_meta,
                    &state_data.cred_def_json,
                )
                .await
                {
                    Ok((cred_id, rev_reg_def_json)) => {
                        HolderFullState::Finished((state_data, cred_id, credential, rev_reg_def_json).into())
                    }
                    Err(err) => {
                        let problem_report = build_problem_report_msg(Some(err.to_string()), &self.thread_id);
                        error!("Failed to process or save received credential: {problem_report:?}");
                        HolderFullState::Finished(FinishedHolderState::new(problem_report))
                    }
                }
            }
            s => {
                warn!("Unable to receive credential offer in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub fn receive_problem_report(self, problem_report: ProblemReport) -> VcxResult<Self> {
        warn!("HolderSM::receive_problem_report >> problem_report: {problem_report:?}");
        let state = match self.state {
            HolderFullState::ProposalSet(_) | HolderFullState::RequestSet(_) => {
                HolderFullState::Finished(FinishedHolderState::new(problem_report))
            }
            s => {
                warn!("Unable to receive problem report in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub fn credential_status(&self) -> u32 {
        match self.state {
            HolderFullState::Finished(ref state) => state.status.code(),
            _ => Status::Undefined.code(),
        }
    }

    pub fn is_terminal_state(&self) -> bool {
        matches!(self.state, HolderFullState::Finished(_))
    }

    pub fn get_credential(&self) -> VcxResult<(String, AriesMessage)> {
        match self.state {
            HolderFullState::Finished(ref state) => {
                let cred_id = state.cred_id.clone().ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Cannot get credential: Credential Id not found",
                ))?;
                let credential = state.credential.clone().ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Cannot get credential: Credential not found",
                ))?;
                Ok((cred_id, credential.into()))
            }
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Cannot get credential: Credential Issuance is not finished yet",
            )),
        }
    }

    pub fn get_attributes(&self) -> VcxResult<String> {
        match self.state {
            HolderFullState::Finished(ref state) => state.get_attributes(),
            HolderFullState::OfferReceived(ref state) => state.get_attributes(),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Cannot get credential attributes: credential offer or credential must be receieved first",
            )),
        }
    }

    pub fn get_attachment(&self) -> VcxResult<String> {
        match self.state {
            HolderFullState::Finished(ref state) => state.get_attachment(),
            HolderFullState::OfferReceived(ref state) => state.get_attachment(),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Cannot get credential attachment: credential offer or credential must be receieved first",
            )),
        }
    }

    pub fn get_tails_location(&self) -> VcxResult<String> {
        match self.state {
            HolderFullState::Finished(ref state) => state.get_tails_location(),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Cannot get tails location: credential exchange not finished yet",
            )),
        }
    }

    pub fn get_tails_hash(&self) -> VcxResult<String> {
        match self.state {
            HolderFullState::Finished(ref state) => state.get_tails_hash(),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Cannot get tails hash: credential exchange not finished yet",
            )),
        }
    }

    pub fn get_rev_reg_id(&self) -> VcxResult<String> {
        match self.state {
            HolderFullState::Finished(ref state) => state.get_rev_reg_id(),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Cannot get rev reg id: credential exchange not finished yet",
            )),
        }
    }

    pub fn get_cred_id(&self) -> VcxResult<String> {
        match self.state {
            HolderFullState::Finished(ref state) => state.get_cred_id(),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Cannot get credential id: credential exchange not finished yet",
            )),
        }
    }

    pub fn get_offer(&self) -> VcxResult<OfferCredential> {
        match self.state {
            HolderFullState::OfferReceived(ref state) => Ok(state.offer.clone()),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Credential offer can only be obtained from OfferReceived state",
            )),
        }
    }

    pub fn get_thread_id(&self) -> VcxResult<String> {
        Ok(self.thread_id.clone())
    }

    pub async fn is_revokable(&self, ledger: &Arc<dyn AnoncredsLedgerRead>) -> VcxResult<bool> {
        match self.state {
            HolderFullState::Initial(ref state) => state.is_revokable(),
            HolderFullState::ProposalSet(ref state) => state.is_revokable(ledger).await,
            HolderFullState::OfferReceived(ref state) => state.is_revokable(ledger).await,
            HolderFullState::RequestSet(ref state) => state.is_revokable(),
            HolderFullState::Finished(ref state) => state.is_revokable(),
        }
    }

    pub async fn is_revoked(
        &self,
        ledger: &Arc<dyn AnoncredsLedgerRead>,
        anoncreds: &Arc<dyn BaseAnonCreds>,
    ) -> VcxResult<bool> {
        if self.is_revokable(ledger).await? {
            let rev_reg_id = self.get_rev_reg_id()?;
            let cred_id = self.get_cred_id()?;
            let rev_id = get_cred_rev_id(anoncreds, &cred_id).await?;
            is_cred_revoked(ledger, &rev_reg_id, &rev_id).await
        } else {
            Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Unable to check revocation status - this credential is not revokable",
            ))
        }
    }

    pub async fn delete_credential(&self, anoncreds: &Arc<dyn BaseAnonCreds>) -> VcxResult<()> {
        trace!("Holder::delete_credential");

        match self.state {
            HolderFullState::Finished(ref state) => {
                let cred_id = state.cred_id.clone().ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Cannot get credential: credential id not found",
                ))?;
                anoncreds
                    .prover_delete_credential(&cred_id)
                    .await
                    .map_err(|err| err.into())
            }
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Cannot delete credential: credential issuance is not finished yet",
            )),
        }
    }

    pub fn get_problem_report(&self) -> VcxResult<ProblemReport> {
        match self.state {
            HolderFullState::Finished(ref state) => match &state.status {
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

pub fn parse_cred_def_id_from_cred_offer(cred_offer: &str) -> VcxResult<String> {
    trace!(
        "Holder::parse_cred_def_id_from_cred_offer >>> cred_offer: {:?}",
        cred_offer
    );

    let parsed_offer: serde_json::Value = serde_json::from_str(cred_offer).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!("Invalid Credential Offer Json: {:?}", err),
        )
    })?;

    let cred_def_id = parsed_offer["cred_def_id"].as_str().ok_or_else(|| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            "Invalid Credential Offer Json: cred_def_id not found",
        )
    })?;

    Ok(cred_def_id.to_string())
}

fn _parse_rev_reg_id_from_credential(credential: &str) -> VcxResult<Option<String>> {
    trace!("Holder::_parse_rev_reg_id_from_credential >>>");

    let parsed_credential: serde_json::Value = serde_json::from_str(credential).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!("Invalid Credential Json: {}, err: {:?}", credential, err),
        )
    })?;

    let rev_reg_id = parsed_credential["rev_reg_id"].as_str().map(String::from);
    trace!("Holder::_parse_rev_reg_id_from_credential <<< {:?}", rev_reg_id);

    Ok(rev_reg_id)
}

async fn _store_credential(
    ledger: &Arc<dyn AnoncredsLedgerRead>,
    anoncreds: &Arc<dyn BaseAnonCreds>,
    credential: &IssueCredential,
    req_meta: &str,
    cred_def_json: &str,
) -> VcxResult<(String, Option<String>)> {
    trace!(
        "Holder::_store_credential >>> credential: {:?}, req_meta: {}, cred_def_json: {}",
        credential,
        req_meta,
        cred_def_json
    );

    let credential_json = get_attach_as_string!(&credential.content.credentials_attach);

    let rev_reg_id = _parse_rev_reg_id_from_credential(&credential_json)?;
    let rev_reg_def_json = if let Some(rev_reg_id) = rev_reg_id {
        let json = ledger.get_rev_reg_def_json(&rev_reg_id).await?;
        Some(json)
    } else {
        None
    };

    let cred_id = anoncreds
        .prover_store_credential(
            None,
            req_meta,
            &credential_json,
            cred_def_json,
            rev_reg_def_json.as_deref(),
        )
        .await?;
    Ok((cred_id, rev_reg_def_json))
}

pub async fn create_anoncreds_credential_request(
    ledger: &Arc<dyn AnoncredsLedgerRead>,
    anoncreds: &Arc<dyn BaseAnonCreds>,
    cred_def_id: &str,
    prover_did: &str,
    cred_offer: &str,
) -> VcxResult<(String, String, String, String)> {
    let cred_def_json = ledger.get_cred_def(cred_def_id, None).await?;

    let master_secret_id = settings::DEFAULT_LINK_SECRET_ALIAS;
    anoncreds
        .prover_create_credential_req(prover_did, cred_offer, &cred_def_json, master_secret_id)
        .await
        .map_err(|err| err.extend("Cannot create credential request"))
        .map(|(s1, s2)| (s1, s2, cred_def_id.to_string(), cred_def_json))
        .map_err(|err| err.into())
}

async fn build_credential_request_msg(
    ledger: &Arc<dyn AnoncredsLedgerRead>,
    anoncreds: &Arc<dyn BaseAnonCreds>,
    thread_id: String,
    my_pw_did: String,
    offer: &OfferCredential,
) -> VcxResult<(RequestCredential, String, String)> {
    trace!(
        "Holder::_make_credential_request >>> my_pw_did: {:?}, offer: {:?}",
        my_pw_did,
        offer
    );

    let cred_offer = get_attach_as_string!(&offer.content.offers_attach);

    trace!("Parsed cred offer attachment: {}", cred_offer);
    let cred_def_id = parse_cred_def_id_from_cred_offer(&cred_offer)?;
    let (req, req_meta, _cred_def_id, cred_def_json) =
        create_anoncreds_credential_request(ledger, anoncreds, &cred_def_id, &my_pw_did, &cred_offer).await?;
    trace!("Created cred def json: {}", cred_def_json);
    let credential_request_msg = _build_credential_request_msg(req, &thread_id);
    Ok((credential_request_msg, req_meta, cred_def_json))
}
