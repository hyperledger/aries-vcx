use std::collections::HashMap;

use crate::libindy::utils::anoncreds::{self, libindy_issuer_create_credential_offer};

use crate::aries::handlers::issuance::issuer::states::credential_sent::CredentialSentState;
use crate::aries::handlers::issuance::issuer::states::finished::FinishedState;
use crate::aries::handlers::issuance::issuer::states::initial::InitialState;
use crate::aries::handlers::issuance::issuer::states::offer_sent::OfferSentState;
use crate::aries::handlers::issuance::issuer::states::requested_received::RequestReceivedState;
use crate::aries::handlers::issuance::issuer::utils::encode_attributes;
use crate::aries::handlers::issuance::messages::CredentialIssuanceMessage;
use crate::aries::messages::a2a::A2AMessage;
use crate::aries::messages::error::ProblemReport;
use crate::aries::messages::issuance::credential::Credential;
use crate::aries::messages::issuance::credential_offer::CredentialOffer;
use crate::aries::messages::issuance::credential_request::CredentialRequest;
use crate::aries::messages::mime_type::MimeType;
use crate::aries::messages::status::Status;
use crate::error::{VcxError, VcxErrorKind, VcxResult};
use crate::api::VcxStateType;
use crate::connection::send_message;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum IssuerState {
    Initial(InitialState),
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

impl IssuerState {
    pub fn thread_id(&self) -> String {
        match self {
            IssuerState::Initial(_) => String::new(),
            IssuerState::OfferSent(state) => state.thread_id.clone(),
            IssuerState::RequestReceived(state) => state.thread_id.clone(),
            IssuerState::CredentialSent(state) => state.thread_id.clone(),
            IssuerState::Finished(state) => state.thread_id.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IssuerSM {
    state: IssuerState,
    source_id: String,
}

impl IssuerSM {
    pub fn new(cred_def_id: &str, credential_data: &str, rev_reg_id: Option<String>, tails_file: Option<String>, source_id: &str) -> Self {
        IssuerSM {
            state: IssuerState::Initial(InitialState::new(cred_def_id, credential_data, rev_reg_id, tails_file)),
            source_id: source_id.to_string(),
        }
    }

    pub fn get_source_id(&self) -> String {
        self.source_id.clone()
    }

    pub fn step(state: IssuerState, source_id: String) -> Self {
        IssuerSM {
            state,
            source_id,
        }
    }

    pub fn revoke(&self, publish: bool) -> VcxResult<()> {
        trace!("Issuer::revoke >>> publish={}", publish);
        match &self.state {
            IssuerState::Finished(state) => {
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
            IssuerState::Initial(state) => state.rev_reg_id.clone(),
            IssuerState::OfferSent(state) => state.rev_reg_id.clone(),
            IssuerState::RequestReceived(state) => state.rev_reg_id.clone(),
            IssuerState::CredentialSent(state) => state.revocation_info_v1.clone()
                .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "No revocation info found - is this credential revokable?"))?
                .rev_reg_id,
            IssuerState::Finished(state) => state.revocation_info_v1.clone()
                .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "No revocation info found - is this credential revokable?"))?
                .rev_reg_id
        };
        rev_registry.ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "No revocation registry id found on revocation info - is this credential revokable?"))
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        trace!("Issuer::find_message_to_handle >>> messages: {:?}", messages);

        for (uid, message) in messages {
            match self.state {
                IssuerState::Initial(_) => {
                    // do not process messages
                }
                IssuerState::OfferSent(_) => {
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
                IssuerState::RequestReceived(_) => {
                    // do not process messages
                }
                IssuerState::CredentialSent(_) => {
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
                IssuerState::Finished(_) => {
                    // do not process messages
                }
            };
        }

        None
    }

    pub fn state(&self) -> u32 {
        match self.state {
            IssuerState::Initial(_) => VcxStateType::VcxStateInitialized as u32,
            IssuerState::OfferSent(_) => VcxStateType::VcxStateOfferSent as u32,
            IssuerState::RequestReceived(_) => VcxStateType::VcxStateRequestReceived as u32,
            IssuerState::CredentialSent(_) => VcxStateType::VcxStateAccepted as u32,
            IssuerState::Finished(ref status) => {
                match status.status {
                    Status::Success => VcxStateType::VcxStateAccepted as u32,
                    _ => VcxStateType::VcxStateNone as u32,
                }
            }
        }
    }

    pub fn handle_message(self, cim: CredentialIssuanceMessage, connection_handle: u32) -> VcxResult<IssuerSM> {
        trace!("IssuerSM::handle_message >>> cim: {:?}", cim);

        let IssuerSM { state, source_id } = self;
        let state = match state {
            IssuerState::Initial(state_data) => match cim {
                CredentialIssuanceMessage::CredentialInit(comment) => {
                    let cred_offer = libindy_issuer_create_credential_offer(&state_data.cred_def_id)?;
                    let cred_offer_msg = CredentialOffer::create()
                        .set_offers_attach(&cred_offer)?
                        .set_comment(comment);
                    let cred_offer_msg = _append_credential_preview(cred_offer_msg, &state_data.credential_json)?;
                    send_message(connection_handle, cred_offer_msg.to_a2a_message())?;
                    IssuerState::OfferSent((state_data, cred_offer, cred_offer_msg.id).into())
                }
                _ => {
                    warn!("Credential Issuance can only start on issuer side with init");
                    IssuerState::Initial(state_data)
                }
            }
            IssuerState::OfferSent(state_data) => match cim {
                CredentialIssuanceMessage::CredentialRequest(request) => {
                    IssuerState::RequestReceived((state_data, request).into())
                }
                CredentialIssuanceMessage::CredentialProposal(_) => {
                    let problem_report = ProblemReport::create()
                        .set_comment(String::from("CredentialProposal is not supported"))
                        .set_thread_id(&state_data.thread_id);

                    send_message(connection_handle, problem_report.to_a2a_message())?;
                    IssuerState::Finished((state_data, problem_report).into())
                }
                CredentialIssuanceMessage::ProblemReport(problem_report) => {
                    IssuerState::Finished((state_data, problem_report).into())
                }
                _ => {
                    warn!("In this state Credential Issuance can accept only Request, Proposal and Problem Report");
                    IssuerState::OfferSent(state_data)
                }
            },
            IssuerState::RequestReceived(state_data) => match cim {
                CredentialIssuanceMessage::CredentialSend() => {
                    let credential_msg = _create_credential(&state_data.request, &state_data.rev_reg_id, &state_data.tails_file, &state_data.offer, &state_data.cred_data);
                    match credential_msg {
                        Ok((credential_msg, cred_rev_id)) => {
                            let credential_msg = credential_msg.set_thread_id(&state_data.thread_id);
                            send_message(connection_handle, credential_msg.to_a2a_message())?;
                            IssuerState::Finished((state_data, cred_rev_id).into())
                        }
                        Err(err) => {
                            let problem_report = ProblemReport::create()
                                .set_comment(err.to_string())
                                .set_thread_id(&state_data.thread_id);

                            send_message(connection_handle, problem_report.to_a2a_message())?;
                            IssuerState::Finished((state_data, problem_report).into())
                        }
                    }
                }
                _ => {
                    warn!("In this state Credential Issuance can accept only CredentialSend");
                    IssuerState::RequestReceived(state_data)
                }
            }
            IssuerState::CredentialSent(state_data) => match cim {
                CredentialIssuanceMessage::ProblemReport(_problem_report) => {
                    info!("Interaction closed with failure");
                    IssuerState::Finished(state_data.into())
                }
                CredentialIssuanceMessage::CredentialAck(_ack) => {
                    info!("Interaction closed with success");
                    IssuerState::Finished(state_data.into())
                }
                _ => {
                    warn!("In this state Credential Issuance can accept only Ack and Problem Report");
                    IssuerState::CredentialSent(state_data)
                }
            }
            IssuerState::Finished(state_data) => {
                warn!("Exchange is finished, no messages can be sent or received");
                IssuerState::Finished(state_data)
            }
        };

        Ok(IssuerSM::step(state, source_id))
    }

    pub fn credential_status(&self) -> u32 {
        trace!("Issuer::credential_status >>>");

        match self.state {
            IssuerState::Finished(ref state) => state.status.code(),
            _ => Status::Undefined.code()
        }
    }

    pub fn is_terminal_state(&self) -> bool {
        match self.state {
            IssuerState::Finished(_) => true,
            _ => false
        }
    }
}


fn _append_credential_preview(cred_offer_msg: CredentialOffer, credential_json: &str) -> VcxResult<CredentialOffer> {
    trace!("Issuer::_append_credential_preview >>> cred_offer_msg: {:?}, credential_json: {:?}", cred_offer_msg, credential_json);

    let cred_values: serde_json::Value = serde_json::from_str(credential_json)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Can't deserialize credential preview json. credential_json={} error={:?}", credential_json, err)))?;

    let values_map = cred_values.as_object()
        .ok_or(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Credential Preview is not object. credential_json={}", credential_json)))?;

    let mut new_offer = cred_offer_msg;
    for item in values_map.iter() {
        let (key, value) = item;
        new_offer = new_offer.add_credential_preview_data(
            key,
            value.as_str()
                .ok_or(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Problem adding credential preview data {}:{:?}", key, value.as_str())))?,
            MimeType::Plain,
        )?;
    }
    Ok(new_offer)
}

fn _create_credential(request: &CredentialRequest, rev_reg_id: &Option<String>, tails_file: &Option<String>, offer: &str, cred_data: &str) -> VcxResult<(Credential, Option<String>)> {
    trace!("Issuer::_create_credential >>> request: {:?}, rev_reg_id: {:?}, tails_file: {:?}, offer: {:?}, cred_data: {:?}", request, rev_reg_id, tails_file, offer, cred_data);

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
    use crate::aries::handlers::connection::tests::mock_connection;
    use crate::aries::messages::issuance::credential::tests::_credential;
    use crate::aries::messages::issuance::credential_offer::tests::_credential_offer;
    use crate::aries::messages::issuance::credential_proposal::tests::_credential_proposal;
    use crate::aries::messages::issuance::credential_request::tests::_credential_request;
    use crate::aries::messages::issuance::test::{_ack, _problem_report};
    use crate::aries::test::source_id;
    use crate::utils::devsetup::SetupMocks;

    use super::*;

    fn _rev_reg_id() -> String {
        String::from("TEST_REV_REG_ID")
    }

    fn _tails_file() -> String {
        String::from("TEST_TAILS_FILE")
    }

    fn _issuer_sm() -> IssuerSM {
        IssuerSM::new("test", &json!({"name": "alice"}).to_string(), Some(_rev_reg_id()), Some(_tails_file()), &source_id())
    }

    impl IssuerSM {
        fn to_offer_sent_state(mut self) -> IssuerSM {
            self = self.handle_message(CredentialIssuanceMessage::CredentialInit(None), mock_connection()).unwrap();
            self
        }

        fn to_request_received_state(mut self) -> IssuerSM {
            self = self.handle_message(CredentialIssuanceMessage::CredentialInit(None), mock_connection()).unwrap();
            self = self.handle_message(CredentialIssuanceMessage::CredentialRequest(_credential_request()), mock_connection()).unwrap();
            self
        }

        fn to_finished_state(mut self) -> IssuerSM {
            let conn_handle = mock_connection();
            self = self.handle_message(CredentialIssuanceMessage::CredentialInit(None), mock_connection()).unwrap();
            self = self.handle_message(CredentialIssuanceMessage::CredentialRequest(_credential_request()), mock_connection()).unwrap();
            self = self.handle_message(CredentialIssuanceMessage::CredentialSend(), mock_connection()).unwrap();
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

            assert_match!(IssuerState::Initial(_), issuer_sm.state);
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

            assert_match!(IssuerState::Initial(_), issuer_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_credential_init_message_from_initial_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialInit(None), mock_connection()).unwrap();

            assert_match!(IssuerState::OfferSent(_), issuer_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_other_messages_from_initial_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();

            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::Credential(_credential()), mock_connection()).unwrap();
            assert_match!(IssuerState::Initial(_), issuer_sm.state);

            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialRequest(_credential_request()), mock_connection()).unwrap();
            assert_match!(IssuerState::Initial(_), issuer_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_credential_request_message_from_offer_sent_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialInit(None), mock_connection()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialRequest(_credential_request()), mock_connection()).unwrap();

            assert_match!(IssuerState::RequestReceived(_), issuer_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_credential_proposal_message_from_offer_sent_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialInit(None), mock_connection()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialProposal(_credential_proposal()), mock_connection()).unwrap();

            assert_match!(IssuerState::Finished(_), issuer_sm.state);
            assert_eq!(Status::Failed(ProblemReport::default()).code(), issuer_sm.credential_status());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_problem_report_message_from_offer_sent_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialInit(None), mock_connection()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::ProblemReport(_problem_report()), mock_connection()).unwrap();

            assert_match!(IssuerState::Finished(_), issuer_sm.state);
            assert_eq!(Status::Failed(ProblemReport::default()).code(), issuer_sm.credential_status());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_other_messages_from_offer_sent_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialInit(None), mock_connection()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::Credential(_credential()), mock_connection()).unwrap();

            assert_match!(IssuerState::OfferSent(_), issuer_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_credential_send_message_from_request_received_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            let conn_handle = mock_connection();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialInit(None), mock_connection()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialRequest(_credential_request()), mock_connection()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialSend(), mock_connection()).unwrap();

            assert_match!(IssuerState::Finished(_), issuer_sm.state);
            assert_eq!(Status::Success.code(), issuer_sm.credential_status());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_credential_send_message_from_request_received_state_with_invalid_request() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            let conn_handle = mock_connection();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialInit(None), mock_connection()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialRequest(CredentialRequest::create()), mock_connection()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialSend(), mock_connection()).unwrap();

            assert_match!(IssuerState::Finished(_), issuer_sm.state);
            assert_eq!(Status::Failed(ProblemReport::default()).code(), issuer_sm.credential_status());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_other_messages_from_request_received_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialInit(None), mock_connection()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialRequest(_credential_request()), mock_connection()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialSend(), mock_connection()).unwrap();

            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialSend(), mock_connection()).unwrap();
            assert_match!(IssuerState::Finished(_), issuer_sm.state);

            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialAck(_ack()), mock_connection()).unwrap();
            assert_match!(IssuerState::Finished(_), issuer_sm.state);
        }

        // TRANSITIONS TO/FROM CREDENTIAL SENT STATE AREN'T POSSIBLE NOW

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_messages_from_finished_state() {
            let _setup = SetupMocks::init();

            let mut issuer_sm = _issuer_sm();
            let conn_handle = mock_connection();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialInit(None), mock_connection()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialRequest(_credential_request()), mock_connection()).unwrap();
            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialSend(), mock_connection()).unwrap();

            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialInit(None), mock_connection()).unwrap();
            assert_match!(IssuerState::Finished(_), issuer_sm.state);

            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::CredentialRequest(_credential_request()), mock_connection()).unwrap();
            assert_match!(IssuerState::Finished(_), issuer_sm.state);

            issuer_sm = issuer_sm.handle_message(CredentialIssuanceMessage::Credential(_credential()), mock_connection()).unwrap();
            assert_match!(IssuerState::Finished(_), issuer_sm.state);
        }
    }

    mod find_message_to_handle {
        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_find_message_to_handle_from_initial_state() {
            let _setup = SetupMocks::init();

            let issuer = _issuer_sm();

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

            assert_eq!(VcxStateType::VcxStateInitialized as u32, _issuer_sm().state());
            assert_eq!(VcxStateType::VcxStateOfferSent as u32, _issuer_sm().to_offer_sent_state().state());
            assert_eq!(VcxStateType::VcxStateRequestReceived as u32, _issuer_sm().to_request_received_state().state());
            assert_eq!(VcxStateType::VcxStateAccepted as u32, _issuer_sm().to_finished_state().state());
        }
    }

    mod get_rev_reg_id {
        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_get_rev_reg_id() {
            let _setup = SetupMocks::init();

            assert_eq!(_rev_reg_id(), _issuer_sm().get_rev_reg_id().unwrap());
            assert_eq!(_rev_reg_id(), _issuer_sm().to_offer_sent_state().get_rev_reg_id().unwrap());
            assert_eq!(_rev_reg_id(), _issuer_sm().to_request_received_state().get_rev_reg_id().unwrap());
            assert_eq!(_rev_reg_id(), _issuer_sm().to_finished_state().get_rev_reg_id().unwrap());
        }
    }
}
