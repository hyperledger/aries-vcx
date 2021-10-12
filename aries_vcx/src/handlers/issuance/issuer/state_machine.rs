use std::collections::HashMap;

use crate::error::{VcxError, VcxErrorKind, VcxResult};
use crate::handlers::issuance::issuer::issuer::IssuerState;
use crate::handlers::issuance::issuer::states::credential_sent::CredentialSentState;
use crate::handlers::issuance::issuer::states::finished::FinishedState;
use crate::handlers::issuance::issuer::states::initial::InitialState;
use crate::handlers::issuance::issuer::states::offer_sent::OfferSentState;
use crate::handlers::issuance::issuer::states::requested_received::RequestReceivedState;
use crate::handlers::issuance::issuer::states::proposal_received::ProposalReceivedState;
use crate::handlers::issuance::issuer::states::OfferInfo;
use crate::handlers::issuance::issuer::utils::encode_attributes;
use crate::handlers::issuance::messages::CredentialIssuanceMessage;
use crate::handlers::issuance::verify_thread_id;
use crate::libindy::utils::anoncreds::{self, libindy_issuer_create_credential_offer};
use crate::messages::a2a::A2AMessage;
use crate::messages::error::ProblemReport;
use crate::messages::issuance::credential::Credential;
use crate::messages::issuance::credential_offer::CredentialOffer;
use crate::messages::issuance::credential_request::CredentialRequest;
use crate::messages::issuance::credential_proposal::CredentialProposal;
use crate::messages::issuance::CredentialPreviewData;
use crate::messages::mime_type::MimeType;
use crate::messages::status::Status;
use crate::settings;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum IssuerFullState {
    Initial(InitialState),
    ProposalReceived(ProposalReceivedState),
    OfferSent(OfferSentState),
    RequestReceived(RequestReceivedState),
    CredentialSent(CredentialSentState),
    Finished(FinishedState),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevocationInfoV1 {
    pub cred_rev_id: Option<String>,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>,
}

impl IssuerFullState {
    pub fn thread_id(&self) -> String {
        match self {
            IssuerFullState::Initial(_) => String::new(),
            IssuerFullState::ProposalReceived(state) => state.credential_proposal.id.0.clone(),
            IssuerFullState::OfferSent(state) => state.thread_id.clone(),
            IssuerFullState::RequestReceived(state) => state.thread_id.clone(),
            IssuerFullState::CredentialSent(state) => state.thread_id.clone(),
            IssuerFullState::Finished(state) => state.thread_id.clone(),
        }
    }
}

impl Default for IssuerFullState {
    fn default() -> Self {
        Self::Initial(InitialState::default())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct IssuerSM {
    state: IssuerFullState,
    source_id: String,
}

impl IssuerSM {
    pub fn new(source_id: &str, cred_def_id: &str, credential_data: &str, rev_reg_id: Option<String>, tails_file: Option<String>) -> Self {
        IssuerSM {
            state: IssuerFullState::Initial(InitialState::new(cred_def_id, credential_data, rev_reg_id, tails_file)),
            source_id: source_id.to_string(),
        }
    }

    pub fn from_proposal(source_id: &str, credential_proposal: &CredentialProposal) -> Self {
        IssuerSM {
            state: IssuerFullState::ProposalReceived(ProposalReceivedState::new(credential_proposal.clone(), None, None, None)),
            source_id: source_id.to_string(),
        }
    }

    pub fn get_source_id(&self) -> String {
        self.source_id.clone()
    }

    pub fn step(state: IssuerFullState, source_id: String) -> Self {
        IssuerSM {
            state,
            source_id,
        }
    }

    pub fn revoke(&self, publish: bool) -> VcxResult<()> {
        trace!("Issuer::revoke >>> publish: {}", publish);
        match &self.state {
            IssuerFullState::Finished(state) => {
                match &state.revocation_info_v1 {
                    Some(rev_info) => {
                        if let (Some(cred_rev_id), Some(rev_reg_id), Some(tails_file)) = (&rev_info.cred_rev_id, &rev_info.rev_reg_id, &rev_info.tails_file) {
                            if publish {
                                anoncreds::revoke_credential(tails_file, rev_reg_id, cred_rev_id)?;
                            } else {
                                anoncreds::revoke_credential_local(tails_file, rev_reg_id, cred_rev_id)?;
                            }
                            Ok(())
                        } else {
                            warn!("Missing data to perform revocation. rev_info={:?}", rev_info);
                            Err(VcxError::from(VcxErrorKind::InvalidRevocationDetails))
                        }
                    }
                    None => Err(VcxError::from(VcxErrorKind::NotReady))
                }
            }
            _ => Err(VcxError::from(VcxErrorKind::NotReady))
        }
    }

    pub fn get_rev_reg_id(&self) -> VcxResult<String> {
        let rev_registry = match &self.state {
            IssuerFullState::Initial(state) => state.rev_reg_id.clone(),
            IssuerFullState::ProposalReceived(state) => state.rev_reg_id.clone(),
            IssuerFullState::OfferSent(state) => state.rev_reg_id.clone(),
            IssuerFullState::RequestReceived(state) => state.rev_reg_id.clone(),
            IssuerFullState::CredentialSent(state) => state.revocation_info_v1.clone()
                .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "No revocation info found - is this credential revokable?"))?
                .rev_reg_id,
            IssuerFullState::Finished(state) => state.revocation_info_v1.clone()
                .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "No revocation info found - is this credential revokable?"))?
                .rev_reg_id
        };
        rev_registry.ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "No revocation registry id found on revocation info - is this credential revokable?"))
    }

    pub fn is_revokable(&self) -> VcxResult<bool> {
        match &self.state {
            IssuerFullState::Initial(state) => Ok(state.rev_reg_id.is_some()),
            IssuerFullState::ProposalReceived(state) => state.is_revokable(),
            IssuerFullState::OfferSent(state) => Ok(state.rev_reg_id.is_some()),
            IssuerFullState::RequestReceived(state) => Ok(state.rev_reg_id.is_some()),
            IssuerFullState::CredentialSent(state) => Ok(state.revocation_info_v1.is_some()),
            IssuerFullState::Finished(state) => Ok(state.revocation_info_v1.is_some())
        }
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        trace!("IssuerSM::find_message_to_handle >>> messages: {:?}, state: {:?}", messages, self.state);

        for (uid, message) in messages {
            match self.state {
                IssuerFullState::Initial(_) => {
                    // do not process messages
                }
                IssuerFullState::ProposalReceived(_) => {
                    // do not process messages
                }
                IssuerFullState::OfferSent(_) => {
                    match message {
                        A2AMessage::CredentialRequest(credential) => {
                            if credential.from_thread(&self.state.thread_id()) {
                                return Some((uid, A2AMessage::CredentialRequest(credential)));
                            }
                        }
                        A2AMessage::CredentialProposal(credential_proposal) => {
                            if let Some(ref thread) = credential_proposal.thread {
                                if thread.is_reply(&self.state.thread_id()) {
                                    return Some((uid, A2AMessage::CredentialProposal(credential_proposal)));
                                }
                            }
                        }
                        A2AMessage::CommonProblemReport(problem_report) => {
                            if problem_report.from_thread(&self.state.thread_id()) {
                                return Some((uid, A2AMessage::CommonProblemReport(problem_report)));
                            }
                        }
                        _ => {}
                    }
                }
                IssuerFullState::RequestReceived(_) => {
                    // do not process messages
                }
                IssuerFullState::CredentialSent(_) => {
                    match message {
                        A2AMessage::Ack(ack) | A2AMessage::CredentialAck(ack) => {
                            if ack.from_thread(&self.state.thread_id()) {
                                return Some((uid, A2AMessage::CredentialAck(ack)));
                            }
                        }
                        A2AMessage::CommonProblemReport(problem_report) => {
                            if problem_report.from_thread(&self.state.thread_id()) {
                                return Some((uid, A2AMessage::CommonProblemReport(problem_report)));
                            }
                        }
                        _ => {}
                    }
                }
                IssuerFullState::Finished(_) => {
                    // do not process messages
                }
            };
        }

        None
    }

    pub fn get_state(&self) -> IssuerState {
        match self.state {
            IssuerFullState::Initial(_) => IssuerState::Initial,
            IssuerFullState::ProposalReceived(_) => IssuerState::ProposalReceived,
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

    pub fn set_offer(self, values: &CredentialPreviewData, cred_def_id: &str, rev_reg_id: Option<String>, tails_file: Option<String>) -> VcxResult<IssuerSM> {
        let IssuerSM { state, source_id } = self;
        let state = match state {
            IssuerFullState::Initial(state) => {
                IssuerFullState::Initial(InitialState {
                    offer_info: OfferInfo::new(values.to_string()?, cred_def_id.to_string()),
                    rev_reg_id,
                    tails_file
                })
            }
            IssuerFullState::ProposalReceived(state) => {
                IssuerFullState::ProposalReceived(ProposalReceivedState {
                    offer_info: Some(OfferInfo::new(values.to_string()?, cred_def_id.to_string())),
                    rev_reg_id,
                    tails_file,
                    ..state
                })
            }
            _ => { state }
        };
        Ok(IssuerSM::step(state, source_id))
    }

    pub fn handle_message(self, cim: CredentialIssuanceMessage, send_message: Option<&impl Fn(&A2AMessage) -> VcxResult<()>>) -> VcxResult<IssuerSM> {
        trace!("IssuerSM::handle_message >>> cim: {:?}, state: {:?}", cim, self.state);
        verify_thread_id(&self.get_thread_id()?, &cim)?;
        let IssuerSM { state, source_id } = self;
        let state = match state {
            IssuerFullState::Initial(state_data) => match cim {
                CredentialIssuanceMessage::CredentialOfferSend(comment) => {
                    let cred_offer = libindy_issuer_create_credential_offer(&state_data.offer_info.cred_def_id)?;
                    let cred_offer_msg = CredentialOffer::create()
                        .set_offers_attach(&cred_offer)?
                        .set_comment(comment);
                    let cred_offer_msg = _append_credential_preview(cred_offer_msg, &state_data.offer_info.credential_json)?;
                    send_message.ok_or(
                        VcxError::from_msg(VcxErrorKind::InvalidState, "Attempted to call undefined send_message callback")
                    )?(&cred_offer_msg.to_a2a_message())?;
                    IssuerFullState::OfferSent((state_data, cred_offer, cred_offer_msg.id).into())
                }
                _ => {
                    warn!("Unable to process this message in this state, ignoring...");
                    IssuerFullState::Initial(state_data)
                }
            }
            IssuerFullState::ProposalReceived(state_data) => match cim {
                CredentialIssuanceMessage::CredentialOfferSend(comment) => {
                    let (cred_def_id, credential_json) = match state_data.offer_info {
                        None => return Err(VcxError::from_msg(VcxErrorKind::InvalidState, "`set_offer()` must be called before sending credential offer after receiving proposal")),
                        Some(offer_info) => (offer_info.cred_def_id, offer_info.credential_json)
                    };
                    let cred_offer = libindy_issuer_create_credential_offer(&cred_def_id)?;
                    let thread_id = state_data.credential_proposal.get_thread_id().unwrap_or(state_data.credential_proposal.id.0.clone());
                    let cred_offer_msg = CredentialOffer::create()
                        .set_thread_id(&thread_id)
                        .set_offers_attach(&cred_offer)?
                        .set_comment(comment);
                    let cred_offer_msg = _append_credential_preview(cred_offer_msg, &credential_json)?;
                    send_message.ok_or(
                        VcxError::from_msg(VcxErrorKind::InvalidState, "Attempted to call undefined send_message callback")
                    )?(&cred_offer_msg.to_a2a_message())?;
                    IssuerFullState::OfferSent((credential_json, cred_offer, thread_id, state_data.rev_reg_id, state_data.tails_file).into())
                }
                _ => {
                    warn!("Unable to process this message in this state, ignoring...");
                    IssuerFullState::ProposalReceived(state_data)
                }
            }
            IssuerFullState::OfferSent(state_data) => match cim {
                CredentialIssuanceMessage::CredentialRequest(request) => {
                    IssuerFullState::RequestReceived((state_data, request).into())
                }
                CredentialIssuanceMessage::CredentialProposal(proposal) => {
                    IssuerFullState::ProposalReceived(ProposalReceivedState::new(proposal, None, None, None))
                }
                CredentialIssuanceMessage::ProblemReport(problem_report) => {
                    IssuerFullState::Finished((state_data, problem_report).into())
                }
                _ => {
                    warn!("In this state Credential Issuance can accept only Request, Proposal and Problem Report");
                    IssuerFullState::OfferSent(state_data)
                }
            },
            IssuerFullState::RequestReceived(state_data) => match cim {
                CredentialIssuanceMessage::CredentialSend() => {
                    let credential_msg = _create_credential(&state_data.request, &state_data.rev_reg_id, &state_data.tails_file, &state_data.offer, &state_data.cred_data, &state_data.thread_id);
                    match credential_msg {
                        Ok((credential_msg, cred_rev_id)) => {
                            let credential_msg = credential_msg.set_thread_id(&state_data.thread_id);
                            send_message.ok_or(
                                VcxError::from_msg(VcxErrorKind::InvalidState, "Attempted to call undefined send_message callback")
                            )?(&credential_msg.to_a2a_message())?;
                            IssuerFullState::Finished((state_data, cred_rev_id).into())
                        }
                        Err(err) => {
                            let problem_report = ProblemReport::create()
                                .set_comment(Some(err.to_string()))
                                .set_thread_id(&state_data.thread_id);

                            send_message.ok_or(
                                VcxError::from_msg(VcxErrorKind::InvalidState, "Attempted to call undefined send_message callback")
                            )?(&problem_report.to_a2a_message())?;
                            // TODO: Shouldn't we transition to CredentialSent and wait for ack?
                            IssuerFullState::Finished((state_data, problem_report).into())
                        }
                    }
                }
                _ => {
                    warn!("In this state Credential Issuance can accept only CredentialSend");
                    IssuerFullState::RequestReceived(state_data)
                }
            }
            IssuerFullState::CredentialSent(state_data) => match cim {
                CredentialIssuanceMessage::ProblemReport(_problem_report) => {
                    info!("Interaction closed with failure");
                    IssuerFullState::Finished(state_data.into())
                }
                CredentialIssuanceMessage::CredentialAck(_ack) => {
                    info!("Interaction closed with success");
                    IssuerFullState::Finished(state_data.into())
                }
                _ => {
                    warn!("In this state Credential Issuance can accept only Ack and Problem Report");
                    IssuerFullState::CredentialSent(state_data)
                }
            }
            IssuerFullState::Finished(state_data) => {
                warn!("Exchange is finished, no messages can be sent or received");
                IssuerFullState::Finished(state_data)
            }
        };

        Ok(IssuerSM::step(state, source_id))
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

    pub fn get_thread_id(&self) -> VcxResult<String> {
        Ok(self.state.thread_id())
    }
}


fn _append_credential_preview(cred_offer_msg: CredentialOffer, credential_json: &str) -> VcxResult<CredentialOffer> {
    trace!("Issuer::_append_credential_preview >>> cred_offer_msg: {:?}, credential_json: {:?}", cred_offer_msg, credential_json);

    let cred_values: serde_json::Value = serde_json::from_str(credential_json)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Can't deserialize credential preview json. credential_json: {}, error: {:?}", credential_json, err)))?;

    let mut new_offer = cred_offer_msg;
    match cred_values {
        serde_json::Value::Array(cred_values) => {
            for cred_value in cred_values.iter() {
                let key = cred_value.get("name").ok_or(VcxError::from_msg(VcxErrorKind::InvalidAttributesStructure, format!("No 'name' field in cred_value: {:?}", cred_value)))?;
                let value = cred_value.get("value").ok_or(VcxError::from_msg(VcxErrorKind::InvalidAttributesStructure, format!("No 'value' field in cred_value: {:?}", cred_value)))?;
                new_offer = new_offer.add_credential_preview_data(
                    &key.to_string(),
                    &value.to_string(),
                    MimeType::Plain,
                )?;
            };
        }
        serde_json::Value::Object(values_map) => {
            for item in values_map.iter() {
                let (key, value) = item;
                new_offer = new_offer.add_credential_preview_data(
                    key,
                    &value.to_string(),
                    MimeType::Plain,
                )?;
            }
        }
        _ => {}
    };

    Ok(new_offer)
}

fn _create_credential(request: &CredentialRequest, rev_reg_id: &Option<String>, tails_file: &Option<String>, offer: &str, cred_data: &str, thread_id: &str) -> VcxResult<(Credential, Option<String>)> {
    trace!("Issuer::_create_credential >>> request: {:?}, rev_reg_id: {:?}, tails_file: {:?}, offer: {}, cred_data: {}, thread_id: {}", request, rev_reg_id, tails_file, offer, cred_data, thread_id);
    if !request.from_thread(&thread_id) {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot handle credential request: thread id does not match: {:?}", request.thread)));
    };
    let request = &request.requests_attach.content()?;
    let cred_data = encode_attributes(cred_data)?;
    let (ser_credential, cred_rev_id, _) = anoncreds::libindy_issuer_create_credential(offer,
                                                                                       &request,
                                                                                       &cred_data,
                                                                                       rev_reg_id.clone(),
                                                                                       tails_file.clone())?;
    let credential = Credential::create().set_credential(ser_credential)?;
    Ok((credential, cred_rev_id))
}

#[cfg(test)]
pub mod test {
    use crate::messages::issuance::credential::test_utils::_credential;
    use crate::messages::issuance::credential_offer::test_utils::_credential_offer;
    use crate::messages::issuance::credential_proposal::test_utils::_credential_proposal;
    use crate::messages::issuance::credential_request::test_utils::{_credential_request, _credential_request_1};
    use crate::messages::issuance::test::{_ack, _problem_report};
    use crate::messages::a2a::A2AMessage;
    use crate::test::source_id;
    use crate::utils::devsetup::SetupMocks;
    use agency_client::mocking::HttpClientMockResponse;

    use super::*;

    pub fn _rev_reg_id() -> String {
        String::from("TEST_REV_REG_ID")
    }

    pub fn _tails_file() -> String {
        String::from("TEST_TAILS_FILE")
    }

    pub fn _send_message() -> Option<&'static impl Fn(&A2AMessage) -> VcxResult<()>> {
        Some(&|_: &A2AMessage| VcxResult::Ok(()))
    }

    fn _issuer_sm() -> IssuerSM {
        IssuerSM::new(&source_id(), "test", &json!({"name": "alice"}).to_string(), Some(_rev_reg_id()), Some(_tails_file()))
    }

    fn _issuer_sm_from_proposal() -> IssuerSM {
        IssuerSM::from_proposal(&source_id(), &_credential_proposal())
    }

    impl IssuerSM {
        fn to_proposal_received_state(mut self) -> IssuerSM {
            let values = _credential_proposal().credential_proposal.clone();
            let cred_def_id = _credential_proposal().cred_def_id.clone();
            Self::from_proposal(&source_id(), &_credential_proposal())
                .set_offer(&values, &cred_def_id, Some(_rev_reg_id()), Some(_tails_file())).unwrap()
        }

        fn to_offer_sent_state(mut self) -> IssuerSM {
            self = self.handle_message(CredentialIssuanceMessage::CredentialOfferSend(None), _send_message()).unwrap();
            self
        }

        fn to_request_received_state(mut self) -> IssuerSM {
            self = self.handle_message(CredentialIssuanceMessage::CredentialOfferSend(None), _send_message()).unwrap();
            self = self.handle_message(CredentialIssuanceMessage::CredentialRequest(_credential_request()), _send_message()).unwrap();
            self
        }

        fn to_finished_state(mut self) -> IssuerSM {
            self = self.handle_message(CredentialIssuanceMessage::CredentialOfferSend(None), _send_message()).unwrap();
            self = self.handle_message(CredentialIssuanceMessage::CredentialRequest(_credential_request()), _send_message()).unwrap();
            self = self.handle_message(CredentialIssuanceMessage::CredentialSend(), _send_message()).unwrap();
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
    }

    mod handle_message {
        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_init() {
            let _setup = SetupMocks::init();

            let issuer_sm = _issuer_sm();

            assert_match!(IssuerFullState::Initial(_), issuer_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_credential_init_message_from_initial_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialOfferSend(None), _send_message()).unwrap();

            assert_match!(IssuerFullState::OfferSent(_), issuer_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_credential_offer_send_message_from_proposal_received() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm_from_proposal();
            let values = _credential_proposal().credential_proposal.clone();
            let cred_def_id = _credential_proposal().cred_def_id.clone();
            issuer_sm = issuer_sm.set_offer(&values, &cred_def_id, Some(_rev_reg_id()), Some(_tails_file())).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialOfferSend(None), _send_message()).unwrap();

            assert_match!(IssuerFullState::OfferSent(_), issuer_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_other_messages_from_proposal_received_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm_from_proposal();

            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::Credential(_credential()), _send_message()).unwrap();
            assert_match!(IssuerFullState::ProposalReceived(_), issuer_sm.state);

            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialRequest(_credential_request()), _send_message()).unwrap();
            assert_match!(IssuerFullState::ProposalReceived(_), issuer_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_other_messages_from_initial_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();

            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::Credential(_credential()), _send_message()).unwrap();
            assert_match!(IssuerFullState::Initial(_), issuer_sm.state);

            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialRequest(_credential_request()), _send_message()).unwrap();
            assert_match!(IssuerFullState::Initial(_), issuer_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_credential_request_message_from_offer_sent_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialOfferSend(None), _send_message()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialRequest(_credential_request()), _send_message()).unwrap();

            assert_match!(IssuerFullState::RequestReceived(_), issuer_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_credential_proposal_message_from_offer_sent_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialOfferSend(None), _send_message()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialProposal(_credential_proposal()), _send_message()).unwrap();

            assert_match!(IssuerFullState::ProposalReceived(_), issuer_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_problem_report_message_from_offer_sent_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialOfferSend(None), _send_message()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::ProblemReport(_problem_report()), _send_message()).unwrap();

            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);
            assert_eq!(Status::Failed(ProblemReport::default()).code(), issuer_sm.credential_status());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_other_messages_from_offer_sent_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialOfferSend(None), _send_message()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::Credential(_credential()), _send_message()).unwrap();

            assert_match!(IssuerFullState::OfferSent(_), issuer_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_credential_send_message_from_request_received_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialOfferSend(None), _send_message()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialRequest(_credential_request()), _send_message()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialSend(), _send_message()).unwrap();

            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);
            assert_eq!(Status::Success.code(), issuer_sm.credential_status());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_credential_send_message_from_request_received_state_with_invalid_request() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialOfferSend(None), _send_message()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialRequest(CredentialRequest::create()), _send_message()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialSend(), _send_message()).unwrap();

            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);
            assert_eq!(Status::Failed(ProblemReport::default()).code(), issuer_sm.credential_status());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_other_messages_from_request_received_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialOfferSend(None), _send_message()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialRequest(_credential_request()), _send_message()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialSend(), _send_message()).unwrap();

            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialSend(), _send_message()).unwrap();
            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);

            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialAck(_ack()), _send_message()).unwrap();
            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_credential_send_fails_with_incorrect_thread_id() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialOfferSend(None), _send_message()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialRequest(_credential_request_1()), _send_message()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialSend(), _send_message()).unwrap();
            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);
            assert_eq!(Status::Failed(ProblemReport::default()).code(), issuer_sm.credential_status());
        }

        // TRANSITIONS TO/FROM CREDENTIAL SENT STATE AREN'T POSSIBLE NOW

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_messages_from_finished_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialOfferSend(None), _send_message()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialRequest(_credential_request()), _send_message()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialSend(), _send_message()).unwrap();

            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialOfferSend(None), _send_message()).unwrap();
            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);

            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialRequest(_credential_request()), _send_message()).unwrap();
            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);

            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::Credential(_credential()), _send_message()).unwrap();
            assert_match!(IssuerFullState::Finished(_), issuer_sm.state);
        }
    }

    mod find_message_to_handle {
        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_find_message_to_handle_from_initial_state() {
            let _setup = SetupMocks::init();

            let issuer = _issuer_sm();

            let messages = map!(
                "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                "key_2".to_string() => A2AMessage::CredentialRequest(_credential_request()),
                "key_3".to_string() => A2AMessage::Credential(_credential()),
                "key_4".to_string() => A2AMessage::CredentialAck(_ack()),
                "key_5".to_string() => A2AMessage::CommonProblemReport(_problem_report())
            );
            assert!(issuer.find_message_to_handle(messages).is_none());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_find_message_to_handle_from_offer_sent_state() {
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

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_find_message_to_handle_from_request_state() {
            let _setup = SetupMocks::init();

            let issuer = _issuer_sm().to_finished_state();

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

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_find_message_to_handle_from_credential_sent_state() {
            let _setup = SetupMocks::init();

            let issuer = _issuer_sm().to_finished_state();

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

        #[test]
        #[cfg(feature = "general_test")]
        fn test_get_state() {
            let _setup = SetupMocks::init();

            assert_eq!(IssuerState::Initial, _issuer_sm().get_state());
            assert_eq!(IssuerState::ProposalReceived, _issuer_sm().to_proposal_received_state().get_state());
            assert_eq!(IssuerState::OfferSent, _issuer_sm().to_offer_sent_state().get_state());
            assert_eq!(IssuerState::RequestReceived, _issuer_sm().to_request_received_state().get_state());
            assert_eq!(IssuerState::Finished, _issuer_sm().to_finished_state().get_state());
        }
    }

    mod get_rev_reg_id {
        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_get_rev_reg_id() {
            let _setup = SetupMocks::init();

            assert_eq!(_rev_reg_id(), _issuer_sm().get_rev_reg_id().unwrap());
            assert_eq!(_rev_reg_id(), _issuer_sm().to_proposal_received_state().get_rev_reg_id().unwrap());
            assert_eq!(_rev_reg_id(), _issuer_sm().to_offer_sent_state().get_rev_reg_id().unwrap());
            assert_eq!(_rev_reg_id(), _issuer_sm().to_request_received_state().get_rev_reg_id().unwrap());
            assert_eq!(_rev_reg_id(), _issuer_sm().to_finished_state().get_rev_reg_id().unwrap());
        }
    }

    mod is_revokable {
        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_is_revokable() {
            let _setup = SetupMocks::init();

            assert_eq!(true, _issuer_sm().is_revokable().unwrap());
            assert_eq!(true, _issuer_sm().to_proposal_received_state().is_revokable().unwrap());
            assert_eq!(true, _issuer_sm().to_offer_sent_state().is_revokable().unwrap());
            assert_eq!(true, _issuer_sm().to_request_received_state().is_revokable().unwrap());
            assert_eq!(true, _issuer_sm().to_finished_state().is_revokable().unwrap());
        }
    }
}
