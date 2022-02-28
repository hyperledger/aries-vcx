use std::collections::HashMap;
use std::fmt::Display;

use crate::error::{VcxError, VcxErrorKind, VcxResult};
use crate::libindy::credentials::encode_attributes;
use crate::protocols::SendClosure;
use crate::libindy::utils::anoncreds;
use crate::messages::a2a::{A2AMessage, MessageId};
use crate::messages::error::ProblemReport;
use crate::messages::issuance::credential::Credential;
use crate::messages::issuance::credential_offer::{CredentialOffer, OfferInfo};
use crate::messages::issuance::credential_proposal::CredentialProposal;
use crate::messages::issuance::credential_request::CredentialRequest;
use crate::messages::status::Status;
use crate::protocols::issuance::actions::CredentialIssuanceAction;
use crate::protocols::issuance::issuer::states::credential_sent::CredentialSentState;
use crate::protocols::issuance::issuer::states::finished::FinishedState;
use crate::protocols::issuance::issuer::states::initial::InitialIssuerState;
use crate::protocols::issuance::issuer::states::offer_sent::OfferSentState;
use crate::protocols::issuance::issuer::states::offer_set::OfferSetState;
use crate::protocols::issuance::issuer::states::proposal_received::ProposalReceivedState;
use crate::protocols::issuance::issuer::states::requested_received::RequestReceivedState;
use crate::protocols::issuance::verify_thread_id;

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

#[derive(Debug, PartialEq)]
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

    pub fn revoke(&self, publish: bool) -> VcxResult<()> {
        trace!("Issuer::revoke >>> publish: {}", publish);
        fn _revoke(rev_info: &Option<RevocationInfoV1>, publish: bool) -> VcxResult<()> {
            match rev_info {
                Some(rev_info) => {
                    if let (Some(cred_rev_id), Some(rev_reg_id), Some(tails_file)) = (&rev_info.cred_rev_id, &rev_info.rev_reg_id, &rev_info.tails_file) {
                        if publish {
                            anoncreds::revoke_credential(tails_file, rev_reg_id, cred_rev_id)?;
                        } else {
                            anoncreds::revoke_credential_local(tails_file, rev_reg_id, cred_rev_id)?;
                        }
                        Ok(())
                    } else {
                        Err(VcxError::from_msg(VcxErrorKind::InvalidRevocationDetails, "Missing data to perform revocation."))
                    }
                }
                None => Err(VcxError::from(VcxErrorKind::NotReady))
            }
        }

        match &self.state {
            IssuerFullState::CredentialSent(state) => _revoke(&state.revocation_info_v1, publish),
            IssuerFullState::Finished(state) => _revoke(&state.revocation_info_v1, publish),
            _ => Err(VcxError::from(VcxErrorKind::NotReady))
        }
    }

    pub fn get_rev_reg_id(&self) -> VcxResult<String> {
        let rev_registry = match &self.state {
            IssuerFullState::Initial(_state) => { return Err(VcxError::from_msg(VcxErrorKind::InvalidState, "No revocation info available in the initial state")); }
            IssuerFullState::OfferSet(state) => state.rev_reg_id.clone(),
            IssuerFullState::ProposalReceived(state) => match &state.offer_info {
                Some(offer_info) => offer_info.rev_reg_id.clone(),
                _ => None
            }
            IssuerFullState::OfferSent(state) => state.rev_reg_id.clone(),
            IssuerFullState::RequestReceived(state) => state.rev_reg_id.clone(),
            IssuerFullState::CredentialSent(state) => state.revocation_info_v1.clone()
                .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "No revocation info found - is this credential revokable?"))?
                .rev_reg_id,
            IssuerFullState::Finished(state) => state.revocation_info_v1.clone()
                .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "No revocation info found - is this credential revokable?"))?
                .rev_reg_id,
        };
        rev_registry.ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "No revocation registry id found on revocation info - is this credential revokable?"))
    }

    pub fn is_revokable(&self) -> bool {
        fn _is_revokable(rev_info: &Option<RevocationInfoV1>) -> bool {
            match rev_info {
                Some(rev_info) => rev_info.cred_rev_id.is_some(),
                None => false
            }
        }
        match &self.state {
            IssuerFullState::CredentialSent(state) => _is_revokable(&state.revocation_info_v1),
            IssuerFullState::Finished(state) => _is_revokable(&state.revocation_info_v1),
            _ => false
        }
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        trace!("IssuerSM::find_message_to_handle >>> messages: {:?}, state: {:?}", messages, self.state);

        for (uid, message) in messages {
            match self.state {
                IssuerFullState::Initial(_) => {
                    match message {
                        A2AMessage::CredentialProposal(credential_proposal) => {
                            return Some((uid, A2AMessage::CredentialProposal(credential_proposal)));
                        }
                        _ => {}
                    }
                }
                IssuerFullState::OfferSent(_) => {
                    match message {
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
                    }
                }
                IssuerFullState::CredentialSent(_) => {
                    match message {
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
                    }
                }
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
            IssuerFullState::Finished(ref status) => {
                match status.status {
                    Status::Success => IssuerState::Finished,
                    _ => IssuerState::Failed
                }
            }
        }
    }

    pub fn get_proposal(&self) -> VcxResult<CredentialProposal> {
        match &self.state {
            IssuerFullState::ProposalReceived(state) => Ok(state.credential_proposal.clone()),
            _ => Err(VcxError::from_msg(VcxErrorKind::InvalidState, "Proposal is only available in ProposalReceived state"))
        }
    }

    pub fn set_offer(self, cred_offer_msg: CredentialOffer, offer_info: &OfferInfo) -> VcxResult<Self> {
        let Self { state, source_id, thread_id } = self;
        let state = match state {
            IssuerFullState::Initial(_) | IssuerFullState::OfferSet(_) | IssuerFullState::ProposalReceived(_) => {
                IssuerFullState::OfferSet(OfferSetState::new(
                    cred_offer_msg,
                    &offer_info.credential_json,
                    &offer_info.cred_def_id,
                    offer_info.rev_reg_id.clone(),
                    offer_info.tails_file.clone(),
                ))
            }
            _ => {
                return Err(VcxError::from_msg(VcxErrorKind::InvalidState,
                                              format!("Can not set_offer in current state {}.", state),
                ));
            }
        };
        Ok(Self::step(source_id, thread_id, state))
    }

    pub fn get_credential_offer(&self) -> VcxResult<CredentialOffer> {
        match &self.state {
            IssuerFullState::OfferSet(state) => Ok(state.offer.clone()),
            IssuerFullState::OfferSent(state) => Ok(state.offer.clone()),
            _ => Err(VcxError::from_msg(VcxErrorKind::InvalidState, format!("Can not get_credential_offer in current state {}.", self.state)))
        }
    }

    pub fn mark_credential_offer_msg_sent(self) -> VcxResult<Self> {
        let Self { state, source_id, thread_id } = self;
        let state = match state {
            IssuerFullState::OfferSet(state) => IssuerFullState::OfferSent(state.into()),
            IssuerFullState::OfferSent(state) => IssuerFullState::OfferSent(state),
            _ => return Err(VcxError::from_msg(VcxErrorKind::InvalidState,
                                               format!("Can not mark_as_offer_sent in current state {}.", state),
            ))
        };
        Ok(Self::step(source_id, thread_id, state))
    }

    pub async fn handle_message(self, cim: CredentialIssuanceAction, send_message: Option<SendClosure>) -> VcxResult<Self> {
        trace!("IssuerSM::handle_message >>> cim: {:?}, state: {:?}", cim, self.state);
        verify_thread_id(&self.thread_id, &cim)?;
        let state_name = self.state.to_string();
        let Self { state, source_id, thread_id } = self;
        let (state, thread_id) = match state {
            IssuerFullState::Initial(state_data) => match cim {
                CredentialIssuanceAction::CredentialProposal(proposal) => {
                    let thread_id = proposal.id.0.to_string();
                    (IssuerFullState::ProposalReceived(ProposalReceivedState::new(proposal, None)), thread_id)
                }
                _ => {
                    warn!("Unable to process received message in state {}", state_name);
                    (IssuerFullState::Initial(state_data), thread_id)
                }
            },
            IssuerFullState::ProposalReceived(state_data) => match cim {
                _ => {
                    warn!("Unable to process received message in state {}", state_name);
                    (IssuerFullState::ProposalReceived(state_data), thread_id)
                }
            }
            IssuerFullState::OfferSent(state_data) => match cim {
                CredentialIssuanceAction::CredentialRequest(request) => {
                    (IssuerFullState::RequestReceived((state_data, request).into()), thread_id)
                }
                CredentialIssuanceAction::CredentialProposal(proposal) => {
                    (IssuerFullState::ProposalReceived(ProposalReceivedState::new(proposal, None)), thread_id)
                }
                CredentialIssuanceAction::ProblemReport(problem_report) => {
                    (IssuerFullState::Finished((state_data, problem_report).into()), thread_id)
                }
                _ => {
                    warn!("Unable to process received message in state {}", state_name);
                    (IssuerFullState::OfferSent(state_data), thread_id)
                }
            },
            IssuerFullState::RequestReceived(state_data) => match cim {
                CredentialIssuanceAction::CredentialSend() => {
                    let credential_msg = _create_credential(&state_data.request, &state_data.rev_reg_id, &state_data.tails_file, &state_data.offer, &state_data.cred_data, &thread_id);
                    match credential_msg {
                        Ok((credential_msg, cred_rev_id)) => {
                            let credential_msg = credential_msg.set_thread_id(&thread_id).ask_for_ack(); // TODO: Make configurable
                            send_message.ok_or(
                                VcxError::from_msg(VcxErrorKind::InvalidState, "Attempted to call undefined send_message callback")
                            )?(credential_msg.to_a2a_message()).await?;
                            (IssuerFullState::CredentialSent((state_data, cred_rev_id).into()), thread_id)
                        }
                        Err(err) => {
                            let problem_report = ProblemReport::create()
                                .set_comment(Some(err.to_string()))
                                .set_thread_id(&thread_id);

                            send_message.ok_or(
                                VcxError::from_msg(VcxErrorKind::InvalidState, "Attempted to call undefined send_message callback")
                            )?(problem_report.to_a2a_message()).await?;
                            (IssuerFullState::Finished((state_data, problem_report).into()), thread_id)
                        }
                    }
                }
                _ => {
                    warn!("Unable to process received message in state {}", state_name);
                    (IssuerFullState::RequestReceived(state_data), thread_id)
                }
            }
            IssuerFullState::CredentialSent(state_data) => match cim {
                CredentialIssuanceAction::ProblemReport(_problem_report) => {
                    info!("Interaction closed with failure");
                    (IssuerFullState::Finished(state_data.into()), thread_id)
                }
                CredentialIssuanceAction::CredentialAck(_ack) => {
                    info!("Interaction closed with success");
                    (IssuerFullState::Finished(state_data.into()), thread_id)
                }
                _ => {
                    warn!("Unable to process received message in state {}", state_name);
                    (IssuerFullState::CredentialSent(state_data), thread_id)
                }
            }
            IssuerFullState::Finished(state_data) => {
                warn!("Unable to process received message in state {}", state_name);
                (IssuerFullState::Finished(state_data), thread_id)
            }
            IssuerFullState::OfferSet(state_data) => {
                warn!("Unable to process received message in state {}", state_name);
                (IssuerFullState::OfferSet(state_data), thread_id)
            }
        };

        Ok(Self::step(source_id, thread_id, state))
    }

    pub fn credential_status(&self) -> u32 {
        trace!("Issuer::credential_status >>>");

        match self.state {
            IssuerFullState::Finished(ref state) => state.status.code(),
            _ => Status::Undefined.code()
        }
    }

    pub fn is_terminal_state(&self) -> bool {
        match self.state {
            IssuerFullState::Finished(_) => true,
            _ => false
        }
    }

    pub fn thread_id(&self) -> VcxResult<String> {
        Ok(self.thread_id.clone())
    }
}

fn _create_credential(request: &CredentialRequest, rev_reg_id: &Option<String>, tails_file: &Option<String>, offer: &CredentialOffer, cred_data: &str, thread_id: &str) -> VcxResult<(Credential, Option<String>)> {
    let offer = offer.offers_attach.content()?;
    trace!("Issuer::_create_credential >>> request: {:?}, rev_reg_id: {:?}, tails_file: {:?}, offer: {}, cred_data: {}, thread_id: {}", request, rev_reg_id, tails_file, offer, cred_data, thread_id);
    if !request.from_thread(&thread_id) {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot handle credential request: thread id does not match: {:?}", request.thread)));
    };
    let request = &request.requests_attach.content()?;
    let cred_data = encode_attributes(cred_data)?;
    let (ser_credential, cred_rev_id, _) = anoncreds::libindy_issuer_create_credential(&offer,
                                                                                       &request,
                                                                                       &cred_data,
                                                                                       rev_reg_id.clone(),
                                                                                       tails_file.clone())?;
    let credential = Credential::create().set_credential(ser_credential)?;
    Ok((credential, cred_rev_id))
}

#[cfg(test)]
pub mod test {
    use crate::messages::a2a::A2AMessage;
    use crate::messages::issuance::credential::test_utils::_credential;
    use crate::messages::issuance::credential_offer::test_utils::{_credential_offer, _offer_info};
    use crate::messages::issuance::credential_proposal::test_utils::_credential_proposal;
    use crate::messages::issuance::credential_request::test_utils::{_credential_request, _credential_request_1};
    use crate::messages::issuance::test::{_ack, _problem_report};
    use crate::test::source_id;
    use crate::utils::constants::LIBINDY_CRED_OFFER;
    use crate::utils::devsetup::SetupMocks;

    use super::*;

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
            let cred_offer = CredentialOffer::create()
                .set_offers_attach(LIBINDY_CRED_OFFER).unwrap();
            let cred_info = _offer_info();
            self = self.set_offer(cred_offer, &cred_info).unwrap();
            self = self.mark_credential_offer_msg_sent().unwrap();
            self
        }

        async fn to_request_received_state(mut self) -> IssuerSM {
            self = self.to_offer_sent_state();
            self = self.handle_message(CredentialIssuanceAction::CredentialRequest(_credential_request()), _send_message()).await.unwrap();
            self
        }

        async fn to_finished_state(mut self) -> IssuerSM {
            self = self.to_request_received_state().await;
            self = self.handle_message(CredentialIssuanceAction::CredentialSend(), _send_message()).await.unwrap();
            self = self.handle_message(CredentialIssuanceAction::CredentialAck(_ack()), _send_message()).await.unwrap();
            self
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
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceAction::CredentialProposal(_credential_proposal()), None).await.unwrap();

            assert_match!(IssuerFullState::ProposalReceived(_), issuer_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_set_credential_offer_message_in_initial_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            let cred_offer = CredentialOffer::create()
                .set_offers_attach(LIBINDY_CRED_OFFER).unwrap();
            let cred_info = _offer_info();
            issuer_sm = issuer_sm.set_offer(cred_offer, &cred_info).unwrap();
            issuer_sm = issuer_sm.mark_credential_offer_msg_sent().unwrap();

            assert_match!(IssuerFullState::OfferSent(_), issuer_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_other_messages_from_initial_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();

            issuer_sm = issuer_sm.handle_message(CredentialIssuanceAction::Credential(_credential()), _send_message()).await.unwrap();
            assert_match!(IssuerFullState::Initial(_), issuer_sm.state);

            issuer_sm = issuer_sm.handle_message(CredentialIssuanceAction::CredentialRequest(_credential_request()), _send_message()).await.unwrap();
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

            issuer_sm = issuer_sm.handle_message(CredentialIssuanceAction::Credential(_credential()), _send_message()).await.unwrap();
            assert_match!(IssuerFullState::ProposalReceived(_), issuer_sm.state);

            issuer_sm = issuer_sm.handle_message(CredentialIssuanceAction::CredentialRequest(_credential_request()), _send_message()).await.unwrap();
            assert_match!(IssuerFullState::ProposalReceived(_), issuer_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_credential_request_message_from_offer_sent_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.to_offer_sent_state();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceAction::CredentialRequest(_credential_request()), _send_message()).await.unwrap();

            assert_match!(IssuerFullState::RequestReceived(_), issuer_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_credential_proposal_message_from_offer_sent_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.to_offer_sent_state();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceAction::CredentialProposal(_credential_proposal()), _send_message()).await.unwrap();

            assert_match!(IssuerFullState::ProposalReceived(_), issuer_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_problem_report_message_from_offer_sent_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.to_offer_sent_state();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceAction::ProblemReport(_problem_report()), _send_message()).await.unwrap();

            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);
            assert_eq!(Status::Failed(ProblemReport::default()).code(), issuer_sm.credential_status());
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_other_messages_from_offer_sent_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.to_offer_sent_state();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceAction::Credential(_credential()), _send_message()).await.unwrap();

            assert_match!(IssuerFullState::OfferSent(_), issuer_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_credential_send_message_from_request_received_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.to_offer_sent_state();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceAction::CredentialRequest(_credential_request()), _send_message()).await.unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceAction::CredentialSend(), _send_message()).await.unwrap();

            assert_match!(IssuerFullState::CredentialSent(_), issuer_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_credential_send_message_from_request_received_state_with_invalid_request() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.to_offer_sent_state();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceAction::CredentialRequest(CredentialRequest::create()), _send_message()).await.unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceAction::CredentialSend(), _send_message()).await.unwrap();

            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);
            assert_eq!(Status::Failed(ProblemReport::default()).code(), issuer_sm.credential_status());
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_other_messages_from_request_received_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.to_offer_sent_state();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceAction::CredentialRequest(_credential_request()), _send_message()).await.unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceAction::CredentialSend(), _send_message()).await.unwrap();
            assert_match!(IssuerFullState::CredentialSent(_), issuer_sm.state);
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceAction::CredentialAck(_ack()), _send_message()).await.unwrap();
            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_credential_send_fails_with_incorrect_thread_id() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.to_offer_sent_state();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceAction::CredentialRequest(_credential_request_1()), _send_message()).await.unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceAction::CredentialSend(), _send_message()).await.unwrap();
            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);
            assert_eq!(Status::Failed(ProblemReport::default()).code(), issuer_sm.credential_status());
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_handle_messages_from_finished_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.to_finished_state().await;
            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);

            issuer_sm = issuer_sm.handle_message(CredentialIssuanceAction::CredentialRequest(_credential_request()), _send_message()).await.unwrap();
            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);

            issuer_sm = issuer_sm.handle_message(CredentialIssuanceAction::Credential(_credential()), _send_message()).await.unwrap();
            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_issuer_in_finished_state_returns_error_on_set_offer() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.to_finished_state().await;
            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);
            let cred_offer = CredentialOffer::create()
                .set_offers_attach(LIBINDY_CRED_OFFER).unwrap();
            let cred_info = _offer_info();

            let res1 = issuer_sm.set_offer(cred_offer, &cred_info);
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
                "key_5".to_string() => A2AMessage::CredentialAck(_ack()),
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
                    "key_2".to_string() => A2AMessage::CredentialAck(_ack()),
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
                    "key_2".to_string() => A2AMessage::CredentialAck(_ack()),
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
                    "key_5".to_string() => A2AMessage::CredentialAck(_ack().set_thread_id("")),
                    "key_6".to_string() => A2AMessage::CommonProblemReport(_problem_report().set_thread_id(""))
                );

                assert!(issuer.find_message_to_handle(messages).is_none());
            }

            // No messages
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                    "key_2".to_string() => A2AMessage::CredentialAck(_ack())
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
                    "key_5".to_string() => A2AMessage::CredentialAck(_ack()),
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
                    "key_5".to_string() => A2AMessage::CredentialAck(_ack()),
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
            assert_eq!(IssuerState::ProposalReceived, _issuer_sm().to_proposal_received_state().get_state());
            assert_eq!(IssuerState::OfferSent, _issuer_sm().to_offer_sent_state().get_state());
            assert_eq!(IssuerState::RequestReceived, _issuer_sm().to_request_received_state().await.get_state());
            assert_eq!(IssuerState::Finished, _issuer_sm().to_finished_state().await.get_state());
        }
    }

    mod get_rev_reg_id {
        use super::*;

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_get_rev_reg_id() {
            let _setup = SetupMocks::init();

            assert_eq!(VcxErrorKind::InvalidState, _issuer_sm().get_rev_reg_id().unwrap_err().kind());
            assert_eq!(VcxErrorKind::InvalidState, _issuer_sm().to_proposal_received_state().get_rev_reg_id().unwrap_err().kind());
            assert_eq!(_rev_reg_id(), _issuer_sm().to_offer_sent_state().get_rev_reg_id().unwrap());
            assert_eq!(_rev_reg_id(), _issuer_sm().to_request_received_state().await.get_rev_reg_id().unwrap());
            assert_eq!(_rev_reg_id(), _issuer_sm().to_finished_state().await.get_rev_reg_id().unwrap());
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
