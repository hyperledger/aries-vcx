use std::{collections::HashMap, fmt, sync::Arc};

use messages::{
    a2a::{A2AMessage, MessageId},
    concepts::{ack::Ack, problem_report::ProblemReport},
    protocols::issuance::{
        credential::Credential,
        credential_ack::CredentialAck,
        credential_offer::CredentialOffer,
        credential_proposal::{CredentialProposal, CredentialProposalData},
        credential_request::CredentialRequest,
    },
    status::Status,
};

use crate::{
    common::credentials::{get_cred_rev_id, is_cred_revoked},
    core::profile::profile::Profile,
    errors::error::prelude::*,
    global::settings,
    protocols::{
        common::build_problem_report_msg,
        issuance::{
            actions::CredentialIssuanceAction,
            holder::states::{
                finished::FinishedHolderState, initial::InitialHolderState, offer_received::OfferReceivedState,
                proposal_sent::ProposalSentState, request_sent::RequestSentState,
            },
            verify_thread_id,
        },
        SendClosure,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HolderFullState {
    Initial(InitialHolderState),
    ProposalSent(ProposalSentState),
    OfferReceived(OfferReceivedState),
    RequestSent(RequestSentState),
    Finished(FinishedHolderState),
}

#[derive(Debug, PartialEq, Eq)]
pub enum HolderState {
    Initial,
    ProposalSent,
    OfferReceived,
    RequestSent,
    Finished,
    Failed,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct HolderSM {
    state: HolderFullState,
    source_id: String,
    thread_id: String,
}

impl Default for HolderFullState {
    fn default() -> Self {
        Self::OfferReceived(OfferReceivedState::default())
    }
}

impl fmt::Display for HolderFullState {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            HolderFullState::Initial(_) => f.write_str("Initial"),
            HolderFullState::ProposalSent(_) => f.write_str("ProposalSent"),
            HolderFullState::OfferReceived(_) => f.write_str("OfferReceived"),
            HolderFullState::RequestSent(_) => f.write_str("RequestSent"),
            HolderFullState::Finished(_) => f.write_str("Finished"),
        }
    }
}

fn build_credential_request_msg(credential_request_attach: String, thread_id: &str) -> VcxResult<CredentialRequest> {
    CredentialRequest::create()
        .set_thread_id(thread_id)
        .set_out_time()
        .set_requests_attach(credential_request_attach)
        .map_err(|err| err.into())
}

fn build_credential_ack(thread_id: &str) -> Ack {
    CredentialAck::create().set_thread_id(thread_id).set_out_time()
}

impl HolderSM {
    pub fn new(source_id: String) -> Self {
        HolderSM {
            thread_id: MessageId::new().0,
            state: HolderFullState::Initial(InitialHolderState::new()),
            source_id,
        }
    }

    pub fn from_offer(offer: CredentialOffer, source_id: String) -> Self {
        HolderSM {
            thread_id: offer.id.0.clone(),
            state: HolderFullState::OfferReceived(OfferReceivedState::new(offer)),
            source_id,
        }
    }

    pub fn get_source_id(&self) -> String {
        self.source_id.clone()
    }

    pub fn get_state(&self) -> HolderState {
        match self.state {
            HolderFullState::Initial(_) => HolderState::Initial,
            HolderFullState::ProposalSent(_) => HolderState::ProposalSent,
            HolderFullState::OfferReceived(_) => HolderState::OfferReceived,
            HolderFullState::RequestSent(_) => HolderState::RequestSent,
            HolderFullState::Finished(ref status) => match status.status {
                Status::Success => HolderState::Finished,
                _ => HolderState::Failed,
            },
        }
    }

    #[allow(dead_code)]
    pub fn get_proposal(&self) -> VcxResult<CredentialProposal> {
        match &self.state {
            HolderFullState::ProposalSent(state) => Ok(state.credential_proposal.clone()),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Proposal not available in this state",
            )),
        }
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        trace!(
            "Holder::find_message_to_handle >>> messages: {:?}, state: {:?}",
            messages,
            self.state
        );
        for (uid, message) in messages {
            match self.state {
                HolderFullState::ProposalSent(_) => {
                    if let A2AMessage::CredentialOffer(offer) = message {
                        if offer.from_thread(&self.thread_id) {
                            return Some((uid, A2AMessage::CredentialOffer(offer)));
                        }
                    }
                }
                HolderFullState::RequestSent(_) => match message {
                    A2AMessage::Credential(credential) => {
                        if credential.from_thread(&self.thread_id) {
                            return Some((uid, A2AMessage::Credential(credential)));
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

    pub async fn handle_message(
        self,
        profile: &Arc<dyn Profile>,
        cim: CredentialIssuanceAction,
        send_message: Option<SendClosure>,
    ) -> VcxResult<HolderSM> {
        trace!("Holder::handle_message >>> cim: {:?}, state: {:?}", cim, self.state);
        let thread_id = self.get_thread_id()?;
        verify_thread_id(&thread_id, &cim)?;
        let holder_sm = match cim {
            CredentialIssuanceAction::CredentialProposalSend(proposal_data) => {
                let send_message = send_message.ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Attempted to call undefined send_message callback",
                ))?;
                self.send_proposal(proposal_data, send_message).await?
            }
            CredentialIssuanceAction::CredentialOffer(offer) => self.receive_offer(offer)?,
            CredentialIssuanceAction::CredentialRequestSend(my_pw_did) => {
                let send_message = send_message.ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Attempted to call undefined send_message callback",
                ))?;
                self.send_request(profile, my_pw_did, send_message).await?
            }
            CredentialIssuanceAction::CredentialOfferReject(comment) => {
                let send_message = send_message.ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Attempted to call undefined send_message callback",
                ))?;
                self.decline_offer(comment, send_message).await?
            }
            CredentialIssuanceAction::Credential(credential) => {
                let send_message = send_message.ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Attempted to call undefined send_message callback",
                ))?;
                self.receive_credential(profile, credential, send_message).await?
            }
            CredentialIssuanceAction::ProblemReport(problem_report) => self.receive_problem_report(problem_report)?,
            _ => self,
        };
        Ok(holder_sm)
    }

    pub async fn send_proposal(
        self,
        proposal_data: CredentialProposalData,
        send_message: SendClosure,
    ) -> VcxResult<Self> {
        verify_thread_id(
            &self.thread_id,
            &CredentialIssuanceAction::CredentialProposalSend(proposal_data.clone()),
        )?;
        let state = match self.state {
            HolderFullState::Initial(_) => {
                let proposal = CredentialProposal::from(proposal_data).set_id(&self.thread_id);
                send_message(proposal.to_a2a_message()).await?;
                HolderFullState::ProposalSent(ProposalSentState::new(proposal))
            }
            HolderFullState::OfferReceived(_) => {
                let proposal = CredentialProposal::from(proposal_data).set_thread_id(&self.thread_id);
                send_message(proposal.to_a2a_message()).await?;
                HolderFullState::ProposalSent(ProposalSentState::new(proposal))
            }
            s => {
                warn!("Unable to send credential proposal in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub fn receive_offer(self, offer: CredentialOffer) -> VcxResult<Self> {
        verify_thread_id(
            &self.thread_id,
            &CredentialIssuanceAction::CredentialOffer(offer.clone()),
        )?;
        let state = match self.state {
            HolderFullState::ProposalSent(_) => HolderFullState::OfferReceived(OfferReceivedState::new(offer)),
            s => {
                warn!("Unable to receive credential offer in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub async fn send_request(
        self,
        profile: &Arc<dyn Profile>,
        my_pw_did: String,
        send_message: SendClosure,
    ) -> VcxResult<Self> {
        let state = match self.state {
            HolderFullState::OfferReceived(state_data) => {
                match _make_credential_request(profile, self.thread_id.clone(), my_pw_did, &state_data.offer).await {
                    Ok((cred_request, req_meta, cred_def_json)) => {
                        send_message(cred_request.to_a2a_message()).await?;
                        HolderFullState::RequestSent((state_data, req_meta, cred_def_json).into())
                    }
                    Err(err) => {
                        let problem_report = build_problem_report_msg(Some(err.to_string()), &self.thread_id);
                        error!(
                            "Failed to create credential request, sending problem report: {:?}",
                            problem_report
                        );
                        send_message(problem_report.to_a2a_message()).await?;
                        HolderFullState::Finished(problem_report.into())
                    }
                }
            }
            s => {
                warn!("Unable to send credential request in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub async fn decline_offer(self, comment: Option<String>, send_message: SendClosure) -> VcxResult<Self> {
        let state = match self.state {
            HolderFullState::OfferReceived(_) => {
                let problem_report = build_problem_report_msg(comment, &self.thread_id);
                send_message(problem_report.to_a2a_message()).await?;
                HolderFullState::Finished(problem_report.into())
            }
            s => {
                warn!("Unable to decline credential offer in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub async fn receive_credential(
        self,
        profile: &Arc<dyn Profile>,
        credential: Credential,
        send_message: SendClosure,
    ) -> VcxResult<Self> {
        let state = match self.state {
            HolderFullState::RequestSent(state_data) => {
                match _store_credential(profile, &credential, &state_data.req_meta, &state_data.cred_def_json).await {
                    Ok((cred_id, rev_reg_def_json)) => {
                        if credential.please_ack.is_some() {
                            let ack = build_credential_ack(&self.thread_id);
                            send_message(A2AMessage::CredentialAck(ack)).await?;
                        }
                        HolderFullState::Finished((state_data, cred_id, credential, rev_reg_def_json).into())
                    }
                    Err(err) => {
                        let problem_report = build_problem_report_msg(Some(err.to_string()), &self.thread_id);
                        error!(
                            "Failed to process or save received credential, sending problem report: {:?}",
                            problem_report
                        );
                        send_message(problem_report.to_a2a_message()).await?;
                        HolderFullState::Finished(problem_report.into())
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
        let state = match self.state {
            HolderFullState::ProposalSent(_) | HolderFullState::RequestSent(_) => {
                HolderFullState::Finished(problem_report.into())
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

    pub fn get_credential(&self) -> VcxResult<(String, A2AMessage)> {
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
                Ok((cred_id, credential.to_a2a_message()))
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

    pub fn get_offer(&self) -> VcxResult<CredentialOffer> {
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

    pub async fn is_revokable(&self, profile: &Arc<dyn Profile>) -> VcxResult<bool> {
        match self.state {
            HolderFullState::Initial(ref state) => state.is_revokable(),
            HolderFullState::ProposalSent(ref state) => state.is_revokable(profile).await,
            HolderFullState::OfferReceived(ref state) => state.is_revokable(profile).await,
            HolderFullState::RequestSent(ref state) => state.is_revokable(),
            HolderFullState::Finished(ref state) => state.is_revokable(),
        }
    }

    pub async fn is_revoked(&self, profile: &Arc<dyn Profile>) -> VcxResult<bool> {
        if self.is_revokable(profile).await? {
            let rev_reg_id = self.get_rev_reg_id()?;
            let cred_id = self.get_cred_id()?;
            let rev_id = get_cred_rev_id(profile, &cred_id).await?;
            is_cred_revoked(profile, &rev_reg_id, &rev_id).await
        } else {
            Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Unable to check revocation status - this credential is not revokable",
            ))
        }
    }

    pub async fn delete_credential(&self, profile: &Arc<dyn Profile>) -> VcxResult<()> {
        trace!("Holder::delete_credential");

        match self.state {
            HolderFullState::Finished(ref state) => {
                let cred_id = state.cred_id.clone().ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Cannot get credential: credential id not found",
                ))?;
                _delete_credential(profile, &cred_id).await
            }
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Cannot delete credential: credential issuance is not finished yet",
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
    profile: &Arc<dyn Profile>,
    credential: &Credential,
    req_meta: &str,
    cred_def_json: &str,
) -> VcxResult<(String, Option<String>)> {
    trace!(
        "Holder::_store_credential >>> credential: {:?}, req_meta: {}, cred_def_json: {}",
        credential,
        req_meta,
        cred_def_json
    );

    let ledger = Arc::clone(profile).inject_ledger();
    let anoncreds = Arc::clone(profile).inject_anoncreds();

    let credential_json = credential.credentials_attach.content()?;
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

async fn _delete_credential(profile: &Arc<dyn Profile>, cred_id: &str) -> VcxResult<()> {
    trace!("Holder::_delete_credential >>> cred_id: {}", cred_id);

    let anoncreds = Arc::clone(profile).inject_anoncreds();
    anoncreds.prover_delete_credential(cred_id).await
}

pub async fn create_credential_request(
    profile: &Arc<dyn Profile>,
    cred_def_id: &str,
    prover_did: &str,
    cred_offer: &str,
) -> VcxResult<(String, String, String, String)> {
    let ledger = Arc::clone(profile).inject_ledger();
    let anoncreds = Arc::clone(profile).inject_anoncreds();
    let cred_def_json = ledger.get_cred_def(cred_def_id, None).await?;

    let master_secret_id = settings::DEFAULT_LINK_SECRET_ALIAS;
    anoncreds
        .prover_create_credential_req(prover_did, cred_offer, &cred_def_json, master_secret_id)
        .await
        .map_err(|err| err.extend("Cannot create credential request"))
        .map(|(s1, s2)| (s1, s2, cred_def_id.to_string(), cred_def_json))
}

async fn _make_credential_request(
    profile: &Arc<dyn Profile>,
    thread_id: String,
    my_pw_did: String,
    offer: &CredentialOffer,
) -> VcxResult<(CredentialRequest, String, String)> {
    trace!(
        "Holder::_make_credential_request >>> my_pw_did: {:?}, offer: {:?}",
        my_pw_did,
        offer
    );

    let cred_offer = offer.offers_attach.content()?;
    trace!("Parsed cred offer attachment: {}", cred_offer);
    let cred_def_id = parse_cred_def_id_from_cred_offer(&cred_offer)?;
    let (req, req_meta, _cred_def_id, cred_def_json) =
        create_credential_request(profile, &cred_def_id, &my_pw_did, &cred_offer).await?;
    trace!("Created cred def json: {}", cred_def_json);
    let credential_request_msg = build_credential_request_msg(req, &thread_id)?;
    Ok((credential_request_msg, req_meta, cred_def_json))
}

#[cfg(feature = "general_test")]
#[cfg(test)]
mod test {
    use messages::protocols::issuance::{
        credential::test_utils::_credential,
        credential_offer::test_utils::_credential_offer,
        credential_proposal::test_utils::_credential_proposal,
        credential_request::test_utils::{_credential_request, _my_pw_did},
        test_utils::{_credential_ack, _problem_report},
    };

    use super::*;
    use crate::{
        common::test_utils::mock_profile,
        test::source_id,
        utils::{constants, devsetup::SetupMocks},
    };

    fn _holder_sm() -> HolderSM {
        HolderSM::from_offer(_credential_offer(), source_id())
    }

    pub fn _send_message() -> Option<SendClosure> {
        Some(Box::new(|_: A2AMessage| Box::pin(async { VcxResult::Ok(()) })))
    }

    impl HolderSM {
        async fn to_request_sent_state(mut self) -> HolderSM {
            self = self
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialRequestSend(_my_pw_did()),
                    _send_message(),
                )
                .await
                .unwrap();
            self
        }

        async fn to_finished_state(mut self) -> HolderSM {
            self = self
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialRequestSend(_my_pw_did()),
                    _send_message(),
                )
                .await
                .unwrap();
            self = self
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::Credential(_credential()),
                    _send_message(),
                )
                .await
                .unwrap();
            self
        }
    }

    mod new {
        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_holder_new() {
            let _setup = SetupMocks::init();

            let holder_sm = _holder_sm();

            assert_match!(HolderFullState::OfferReceived(_), holder_sm.state);
            assert_eq!(source_id(), holder_sm.get_source_id());
        }
    }

    mod build_messages {
        use messages::a2a::MessageId;

        use crate::{
            protocols::issuance::holder::state_machine::{build_credential_ack, build_credential_request_msg},
            utils::devsetup::{was_in_past, SetupMocks},
        };

        #[test]
        #[cfg(feature = "general_test")]
        fn test_holder_build_credential_request_msg() {
            let _setup = SetupMocks::init();
            let msg = build_credential_request_msg("{}".into(), "12345").unwrap();

            assert_eq!(msg.id, MessageId::default());
            assert_eq!(msg.thread.unwrap().thid.unwrap(), "12345");
            assert!(was_in_past(
                &msg.timing.unwrap().out_time.unwrap(),
                chrono::Duration::milliseconds(100),
            )
            .unwrap());
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_holder_build_credential_ack() {
            let _setup = SetupMocks::init();

            let msg = build_credential_ack("12345");

            assert_eq!(msg.id, MessageId::default());
            assert_eq!(msg.thread.thid.unwrap(), "12345");
            assert!(was_in_past(
                &msg.timing.unwrap().out_time.unwrap(),
                chrono::Duration::milliseconds(100),
            )
            .unwrap());
        }
    }

    mod step {
        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_holder_init() {
            let _setup = SetupMocks::init();

            let holder_sm = _holder_sm();
            assert_match!(HolderFullState::OfferReceived(_), holder_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_credential_request_sent_message_from_offer_received_state() {
            let _setup = SetupMocks::init();

            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialRequestSend(_my_pw_did()),
                    _send_message(),
                )
                .await
                .unwrap();

            assert_match!(HolderFullState::RequestSent(_), holder_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_credential_request_sent_message_from_offer_received_state_for_invalid_offer() {
            let _setup = SetupMocks::init();

            let credential_offer = CredentialOffer::create()
                .set_offers_attach(r#"{"credential offer": {}}"#)
                .unwrap();

            let mut holder_sm = HolderSM::from_offer(credential_offer, "test source".to_string());
            holder_sm = holder_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialRequestSend(_my_pw_did()),
                    _send_message(),
                )
                .await
                .unwrap();

            assert_match!(HolderFullState::Finished(_), holder_sm.state);
            assert_eq!(
                Status::Failed(ProblemReport::default()).code(),
                holder_sm.credential_status()
            );
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_other_messages_from_offer_received_state() {
            let _setup = SetupMocks::init();

            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialSend(),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(HolderFullState::OfferReceived(_), holder_sm.state);

            holder_sm = holder_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::ProblemReport(_problem_report()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(HolderFullState::OfferReceived(_), holder_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_credential_message_from_request_sent_state() {
            let _setup = SetupMocks::init();

            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialRequestSend(_my_pw_did()),
                    _send_message(),
                )
                .await
                .unwrap();
            holder_sm = holder_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::Credential(_credential()),
                    _send_message(),
                )
                .await
                .unwrap();

            assert_match!(HolderFullState::Finished(_), holder_sm.state);
            assert_eq!(Status::Success.code(), holder_sm.credential_status());
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_invalid_credential_message_from_request_sent_state() {
            let _setup = SetupMocks::init();

            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialRequestSend(_my_pw_did()),
                    _send_message(),
                )
                .await
                .unwrap();
            holder_sm = holder_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::Credential(Credential::create()),
                    _send_message(),
                )
                .await
                .unwrap();

            assert_match!(HolderFullState::Finished(_), holder_sm.state);
            assert_eq!(
                Status::Failed(ProblemReport::default()).code(),
                holder_sm.credential_status()
            );
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_problem_report_from_request_sent_state() {
            let _setup = SetupMocks::init();

            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialRequestSend(_my_pw_did()),
                    _send_message(),
                )
                .await
                .unwrap();
            holder_sm = holder_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::ProblemReport(_problem_report()),
                    _send_message(),
                )
                .await
                .unwrap();

            assert_match!(HolderFullState::Finished(_), holder_sm.state);
            assert_eq!(
                Status::Failed(ProblemReport::default()).code(),
                holder_sm.credential_status()
            );
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_other_messages_from_request_sent_state() {
            let _setup = SetupMocks::init();

            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialRequestSend(_my_pw_did()),
                    _send_message(),
                )
                .await
                .unwrap();

            holder_sm = holder_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialOffer(_credential_offer()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(HolderFullState::RequestSent(_), holder_sm.state);

            holder_sm = holder_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialAck(_credential_ack()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(HolderFullState::RequestSent(_), holder_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_message_from_finished_state() {
            let _setup = SetupMocks::init();

            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialRequestSend(_my_pw_did()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(HolderFullState::RequestSent(_), holder_sm.state);

            holder_sm = holder_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::Credential(_credential()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(HolderFullState::Finished(_), holder_sm.state);

            holder_sm = holder_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialOffer(_credential_offer()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(HolderFullState::Finished(_), holder_sm.state);

            holder_sm = holder_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::Credential(_credential()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(HolderFullState::Finished(_), holder_sm.state);

            holder_sm = holder_sm
                .handle_message(
                    &mock_profile(),
                    CredentialIssuanceAction::CredentialAck(_credential_ack()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(HolderFullState::Finished(_), holder_sm.state);
        }
    }

    mod find_message_to_handle {
        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_holder_find_message_to_handle_from_offer_received_state() {
            let _setup = SetupMocks::init();

            let holder = _holder_sm();

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

                assert!(holder.find_message_to_handle(messages).is_none());
            }
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_holder_find_message_to_handle_from_request_sent_state() {
            let _setup = SetupMocks::init();

            let holder = _holder_sm().to_request_sent_state().await;

            // CredentialAck
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                    "key_2".to_string() => A2AMessage::CredentialRequest(_credential_request()),
                    "key_3".to_string() => A2AMessage::CredentialProposal(_credential_proposal()),
                    "key_4".to_string() => A2AMessage::Credential(_credential())
                );

                let (uid, message) = holder.find_message_to_handle(messages).unwrap();
                assert_eq!("key_4", uid);
                assert_match!(A2AMessage::Credential(_), message);
            }

            // Problem Report
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                    "key_2".to_string() => A2AMessage::CredentialRequest(_credential_request()),
                    "key_3".to_string() => A2AMessage::CredentialProposal(_credential_proposal()),
                    "key_4".to_string() => A2AMessage::CredentialAck(_credential_ack()),
                    "key_5".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                let (uid, message) = holder.find_message_to_handle(messages).unwrap();
                assert_eq!("key_5", uid);
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

                assert!(holder.find_message_to_handle(messages).is_none());
            }

            // No messages
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                    "key_2".to_string() => A2AMessage::CredentialRequest(_credential_request()),
                    "key_3".to_string() => A2AMessage::CredentialProposal(_credential_proposal())
                );

                assert!(holder.find_message_to_handle(messages).is_none());
            }
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_holder_find_message_to_handle_from_finished_state() {
            let _setup = SetupMocks::init();

            let holder = _holder_sm().to_finished_state().await;

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

                assert!(holder.find_message_to_handle(messages).is_none());
            }
        }
    }

    mod get_state {
        use super::*;

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_get_state() {
            let _setup = SetupMocks::init();

            assert_eq!(HolderState::OfferReceived, _holder_sm().get_state());
            assert_eq!(
                HolderState::RequestSent,
                _holder_sm().to_request_sent_state().await.get_state()
            );
            assert_eq!(
                HolderState::Finished,
                _holder_sm().to_finished_state().await.get_state()
            );
        }
    }

    mod get_tails_location {
        use super::*;

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_get_tails_location() {
            let _setup = SetupMocks::init();

            assert_eq!(
                Err(AriesVcxErrorKind::NotReady),
                _holder_sm().get_tails_location().map_err(|e| e.kind())
            );
            assert_eq!(
                Err(AriesVcxErrorKind::NotReady),
                _holder_sm()
                    .to_request_sent_state()
                    .await
                    .get_tails_location()
                    .map_err(|e| e.kind())
            );
            assert_eq!(
                constants::TEST_TAILS_LOCATION,
                _holder_sm().to_finished_state().await.get_tails_location().unwrap()
            );
        }
    }

    mod get_tails_hash {
        use super::*;

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_get_tails_hash() {
            let _setup = SetupMocks::init();

            assert_eq!(
                Err(AriesVcxErrorKind::NotReady),
                _holder_sm().get_tails_hash().map_err(|e| e.kind())
            );
            assert_eq!(
                Err(AriesVcxErrorKind::NotReady),
                _holder_sm()
                    .to_request_sent_state()
                    .await
                    .get_tails_hash()
                    .map_err(|e| e.kind())
            );

            assert_eq!(
                constants::TEST_TAILS_HASH,
                _holder_sm().to_finished_state().await.get_tails_hash().unwrap()
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
                Err(AriesVcxErrorKind::NotReady),
                _holder_sm().get_rev_reg_id().map_err(|e| e.kind())
            );
            assert_eq!(
                Err(AriesVcxErrorKind::NotReady),
                _holder_sm()
                    .to_request_sent_state()
                    .await
                    .get_rev_reg_id()
                    .map_err(|e| e.kind())
            );

            assert_eq!(
                constants::REV_REG_ID,
                _holder_sm().to_finished_state().await.get_rev_reg_id().unwrap()
            );
        }
    }

    mod is_revokable {
        use super::*;

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_is_revokable() {
            let _setup = SetupMocks::init();
            assert_eq!(true, _holder_sm().is_revokable(&mock_profile()).await.unwrap());
            assert_eq!(
                true,
                _holder_sm()
                    .to_request_sent_state()
                    .await
                    .is_revokable(&mock_profile())
                    .await
                    .unwrap()
            );
            assert_eq!(
                true,
                _holder_sm()
                    .to_finished_state()
                    .await
                    .is_revokable(&mock_profile())
                    .await
                    .unwrap()
            );
        }
    }
}
