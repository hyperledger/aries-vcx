use std::collections::HashMap;

use crate::api::VcxStateType;
use crate::aries::handlers::issuance::holder::states::finished::FinishedHolderState;
use crate::aries::handlers::issuance::holder::states::offer_received::OfferReceivedState;
use crate::aries::handlers::issuance::holder::states::request_sent::RequestSentState;
use crate::aries::handlers::issuance::messages::CredentialIssuanceMessage;
use crate::aries::messages::a2a::A2AMessage;
use crate::aries::messages::error::ProblemReport;
use crate::aries::messages::issuance::credential::Credential;
use crate::aries::messages::issuance::credential_ack::CredentialAck;
use crate::aries::messages::issuance::credential_offer::CredentialOffer;
use crate::aries::messages::issuance::credential_request::CredentialRequest;
use crate::aries::messages::status::Status;
use crate::connection;
use crate::error::prelude::*;
use crate::libindy::utils::anoncreds::{self, get_cred_def_json, libindy_prover_create_credential_req, libindy_prover_delete_credential, libindy_prover_store_credential};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HolderState {
    OfferReceived(OfferReceivedState),
    RequestSent(RequestSentState),
    Finished(FinishedHolderState),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HolderSM {
    state: HolderState,
    source_id: String,
    thread_id: String,
}

impl HolderSM {
    pub fn new(offer: CredentialOffer, source_id: String) -> Self {
        HolderSM {
            thread_id: offer.id.0.clone(),
            state: HolderState::OfferReceived(OfferReceivedState::new(offer)),
            source_id,
        }
    }

    pub fn get_source_id(&self) -> String {
        self.source_id.clone()
    }

    pub fn state(&self) -> u32 {
        match self.state {
            HolderState::OfferReceived(_) => VcxStateType::VcxStateRequestReceived as u32,
            HolderState::RequestSent(_) => VcxStateType::VcxStateOfferSent as u32,
            HolderState::Finished(ref status) => {
                match status.status {
                    Status::Success => VcxStateType::VcxStateAccepted as u32,
                    _ => VcxStateType::VcxStateNone as u32,
                }
            }
        }
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        trace!("Holder::find_message_to_handle >>> messages: {:?}", messages);

        for (uid, message) in messages {
            match self.state {
                HolderState::OfferReceived(_) => {
                    // do not process messages
                }
                HolderState::RequestSent(_) => {
                    match message {
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
                    }
                }
                HolderState::Finished(_) => {
                    // do not process messages
                }
            };
        }

        None
    }

    pub fn step(state: HolderState, source_id: String, thread_id: String) -> Self {
        HolderSM { state, source_id, thread_id }
    }

    pub fn handle_message(self, cim: CredentialIssuanceMessage, connection_handle: u32) -> VcxResult<HolderSM> {
        trace!("Holder::handle_message >>> cim: {:?}", cim);

        let HolderSM { state, source_id, thread_id } = self;
        let state = match state {
            HolderState::OfferReceived(state_data) => match cim {
                CredentialIssuanceMessage::CredentialRequestSend() => {
                    let request = _make_credential_request(connection_handle, &state_data.offer);
                    match request {
                        Ok((cred_request, req_meta, cred_def_json)) => {
                            let cred_request = cred_request
                                .set_thread_id(&thread_id);
                            connection::send_message(connection_handle, cred_request.to_a2a_message())?;
                            HolderState::RequestSent((state_data, req_meta, cred_def_json).into())
                        }
                        Err(err) => {
                            let problem_report = ProblemReport::create()
                                .set_comment(err.to_string())
                                .set_thread_id(&thread_id);
                            connection::send_message(connection_handle, problem_report.to_a2a_message())?;
                            HolderState::Finished((state_data, problem_report).into())
                        }
                    }
                }
                _ => {
                    warn!("Credential Issuance can only start on holder side with Credential Offer");
                    HolderState::OfferReceived(state_data)
                }
            },
            HolderState::RequestSent(state_data) => match cim {
                CredentialIssuanceMessage::Credential(credential) => {
                    let result = _store_credential(&credential, &state_data.req_meta, &state_data.cred_def_json);
                    match result {
                        Ok((cred_id, rev_reg_def_json)) => {
                            if credential.please_ack.is_some() {
                                let ack = CredentialAck::create().set_thread_id(&thread_id);
                                connection::send_message(connection_handle, A2AMessage::CredentialAck(ack))?;
                            }

                            HolderState::Finished((state_data, cred_id, credential, rev_reg_def_json).into())
                        }
                        Err(err) => {
                            let problem_report = ProblemReport::create()
                                .set_comment(err.to_string())
                                .set_thread_id(&thread_id);

                            connection::send_message(connection_handle, problem_report.to_a2a_message())?;
                            HolderState::Finished((state_data, problem_report).into())
                        }
                    }
                }
                CredentialIssuanceMessage::ProblemReport(problem_report) => {
                    HolderState::Finished((state_data, problem_report).into())
                }
                _ => {
                    warn!("In this state Credential Issuance can accept only Credential and Problem Report");
                    HolderState::RequestSent(state_data)
                }
            },
            HolderState::Finished(state_data) => {
                warn!("Exchange is finished, no messages can be sent or received");
                HolderState::Finished(state_data)
            }
        };
        Ok(HolderSM::step(state, source_id, thread_id))
    }

    pub fn credential_status(&self) -> u32 {
        match self.state {
            HolderState::Finished(ref state) => state.status.code(),
            _ => Status::Undefined.code()
        }
    }

    pub fn is_terminal_state(&self) -> bool {
        match self.state {
            HolderState::Finished(_) => true,
            _ => false
        }
    }

    pub fn get_credential(&self) -> VcxResult<(String, A2AMessage)> {
        match self.state {
            HolderState::Finished(ref state) => {
                let cred_id = state.cred_id.clone().ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "Cannot get credential: Credential Id not found"))?;
                let credential = state.credential.clone().ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "Cannot get credential: Credential not found"))?;
                Ok((cred_id, credential.to_a2a_message()))
            }
            _ => Err(VcxError::from_msg(VcxErrorKind::NotReady, "Cannot get credential: Credential Issuance is not finished yet"))
        }
    }

    pub fn get_attributes(&self) -> VcxResult<String> {
        match self.state {
            HolderState::Finished(ref state) => state.get_attributes(),
            HolderState::OfferReceived(ref state) => state.get_attributes(),
            _ => Err(VcxError::from_msg(VcxErrorKind::NotReady, "Cannot get credential attributes: credential offer or credential must be receieved first"))
        }
    }

    pub fn get_attachment(&self) -> VcxResult<String> {
        match self.state {
            HolderState::Finished(ref state) => state.get_attachment(),
            HolderState::OfferReceived(ref state) => state.get_attachment(),
            _ => Err(VcxError::from_msg(VcxErrorKind::NotReady, "Cannot get credential attachment: credential offer or credential must be receieved first"))
        }
    }

    pub fn get_tails_location(&self) -> VcxResult<String> {
        match self.state {
            HolderState::Finished(ref state) => state.get_tails_location(),
            _ => Err(VcxError::from_msg(VcxErrorKind::NotReady, "Cannot get tails location: credential exchange not finished yet"))
        }
    }

    pub fn get_tails_hash(&self) -> VcxResult<String> {
        match self.state {
            HolderState::Finished(ref state) => state.get_tails_hash(),
            _ => Err(VcxError::from_msg(VcxErrorKind::NotReady, "Cannot get tails hash: credential exchange not finished yet"))
        }
    }

    pub fn get_rev_reg_id(&self) -> VcxResult<String> {
        match self.state {
            HolderState::Finished(ref state) => state.get_rev_reg_id(),
            _ => Err(VcxError::from_msg(VcxErrorKind::NotReady, "Cannot get tails hash: credential exchange not finished yet"))
        }
    }

    pub fn delete_credential(&self) -> VcxResult<()> {
        trace!("Holder::delete_credential");

        match self.state {
            HolderState::Finished(ref state) => {
                let cred_id = state.cred_id.clone().ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "Cannot get credential: credential id not found"))?;
                _delete_credential(&cred_id)
            }
            _ => Err(VcxError::from_msg(VcxErrorKind::NotReady, "Cannot delete credential: credential issuance is not finished yet"))
        }
    }
}

fn _parse_cred_def_from_cred_offer(cred_offer: &str) -> VcxResult<String> {
    trace!("Holder::_parse_cred_def_from_cred_offer >>> cred_offer: {:?}", cred_offer);

    let parsed_offer: serde_json::Value = serde_json::from_str(cred_offer)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Invalid Credential Offer Json: {:?}", err)))?;

    let cred_def_id = parsed_offer["cred_def_id"].as_str()
        .ok_or_else(|| VcxError::from_msg(VcxErrorKind::InvalidJson, "Invalid Credential Offer Json: cred_def_id not found"))?;

    Ok(cred_def_id.to_string())
}

fn _parse_rev_reg_id_from_credential(credential: &str) -> VcxResult<Option<String>> {
    trace!("Holder::_parse_rev_reg_id_from_credential >>>");

    let parsed_credential: serde_json::Value = serde_json::from_str(credential)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Invalid Credential Json: {}, err: {:?}", credential, err)))?;

    let rev_reg_id = parsed_credential["rev_reg_id"].as_str().map(String::from);

    Ok(rev_reg_id)
}

fn _store_credential(credential: &Credential,
                     req_meta: &str, cred_def_json: &str) -> VcxResult<(String, Option<String>)> {
    trace!("Holder::_store_credential >>>");

    let credential_json = credential.credentials_attach.content()?;
    let rev_reg_id = _parse_rev_reg_id_from_credential(&credential_json)?;
    let rev_reg_def_json = if let Some(rev_reg_id) = rev_reg_id {
        let (_, json) = anoncreds::get_rev_reg_def_json(&rev_reg_id)?;
        Some(json)
    } else {
        None
    };

    let cred_id = libindy_prover_store_credential(None,
                                                  req_meta,
                                                  &credential_json,
                                                  cred_def_json,
                                                  rev_reg_def_json.as_ref().map(String::as_str))?;
    Ok((cred_id, rev_reg_def_json))
}

fn _delete_credential(cred_id: &str) -> VcxResult<()> {
    trace!("Holder::_delete_credential >>> cred_id: {}", cred_id);

    libindy_prover_delete_credential(cred_id)
}

pub fn create_credential_request(cred_def_id: &str, prover_did: &str, cred_offer: &str) -> VcxResult<(String, String, String, String)> {
    let (cred_def_id, cred_def_json) = get_cred_def_json(&cred_def_id)?;

    libindy_prover_create_credential_req(&prover_did,
                                         &cred_offer,
                                         &cred_def_json)
        .map_err(|err| err.extend("Cannot create credential request")).map(|(s1, s2)| (s1, s2, cred_def_id, cred_def_json))
}

fn _make_credential_request(conn_handle: u32, offer: &CredentialOffer) -> VcxResult<(CredentialRequest, String, String)> {
    trace!("Holder::_make_credential_request >>> conn_handle: {:?}, offer: {:?}", conn_handle, offer);

    let my_did = connection::get_pw_did(conn_handle)?;
    let cred_offer = offer.offers_attach.content()?;
    let cred_def_id = _parse_cred_def_from_cred_offer(&cred_offer)?;
    let (req, req_meta, _cred_def_id, cred_def_json) = create_credential_request(&cred_def_id, &my_did, &cred_offer)?;
    Ok((CredentialRequest::create().set_requests_attach(req)?, req_meta, cred_def_json))
}

#[cfg(test)]
mod test {
    use crate::aries::handlers::connection::tests::mock_connection;
    use crate::aries::messages::issuance::credential::tests::_credential;
    use crate::aries::messages::issuance::credential_offer::tests::_credential_offer;
    use crate::aries::messages::issuance::credential_proposal::tests::_credential_proposal;
    use crate::aries::messages::issuance::credential_request::tests::_credential_request;
    use crate::aries::messages::issuance::test::{_ack, _problem_report};
    use crate::aries::test::source_id;
    use crate::utils::constants;
    use crate::utils::devsetup::SetupMocks;

    use super::*;

    fn _holder_sm() -> HolderSM {
        HolderSM::new(_credential_offer(), source_id())
    }

    impl HolderSM {
        fn to_request_sent_state(mut self) -> HolderSM {
            self = self.handle_message(CredentialIssuanceMessage::CredentialRequestSend(), mock_connection()).unwrap();
            self
        }

        fn to_finished_state(mut self) -> HolderSM {
            let handle_conn = mock_connection();
            self = self.handle_message(CredentialIssuanceMessage::CredentialRequestSend(),handle_conn).unwrap();
            self = self.handle_message(CredentialIssuanceMessage::Credential(_credential()), handle_conn).unwrap();
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

            assert_match!(HolderState::OfferReceived(_), holder_sm.state);
            assert_eq!(source_id(), holder_sm.get_source_id());
        }
    }

    mod step {
        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_holder_init() {
            let _setup = SetupMocks::init();

            let holder_sm = _holder_sm();
            assert_match!(HolderState::OfferReceived(_), holder_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_credential_request_sent_message_from_offer_received_state() {
            let _setup = SetupMocks::init();

            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialRequestSend(), mock_connection()).unwrap();

            assert_match!(HolderState::RequestSent(_), holder_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_credential_request_sent_message_from_offer_received_state_for_invalid_offer() {
            let _setup = SetupMocks::init();

            let credential_offer = CredentialOffer::create().set_offers_attach(r#"{"credential offer": {}}"#).unwrap();

            let mut holder_sm = HolderSM::new(credential_offer, "test source".to_string());
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialRequestSend(), mock_connection()).unwrap();

            assert_match!(HolderState::Finished(_), holder_sm.state);
            assert_eq!(Status::Failed(ProblemReport::default()).code(), holder_sm.credential_status());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_other_messages_from_offer_received_state() {
            let _setup = SetupMocks::init();

            let mut holder_sm = _holder_sm();
            let handle_connection = mock_connection();
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialSend(), handle_connection).unwrap();
            assert_match!(HolderState::OfferReceived(_), holder_sm.state);

            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::ProblemReport(_problem_report()), handle_connection).unwrap();
            assert_match!(HolderState::OfferReceived(_), holder_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_credential_message_from_request_sent_state() {
            let _setup = SetupMocks::init();

            let handle_connection = mock_connection();
            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialRequestSend(), handle_connection).unwrap();
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::Credential(_credential()), handle_connection).unwrap();

            assert_match!(HolderState::Finished(_), holder_sm.state);
            assert_eq!(Status::Success.code(), holder_sm.credential_status());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_invalid_credential_message_from_request_sent_state() {
            let _setup = SetupMocks::init();

            let handle_connection = mock_connection();
            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialRequestSend(), handle_connection).unwrap();
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::Credential(Credential::create()), handle_connection).unwrap();

            assert_match!(HolderState::Finished(_), holder_sm.state);
            assert_eq!(Status::Failed(ProblemReport::default()).code(), holder_sm.credential_status());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_problem_report_from_request_sent_state() {
            let _setup = SetupMocks::init();

            let handle_connection = mock_connection();
            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialRequestSend(), handle_connection).unwrap();
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::ProblemReport(_problem_report()), handle_connection).unwrap();

            assert_match!(HolderState::Finished(_), holder_sm.state);
            assert_eq!(Status::Failed(ProblemReport::default()).code(), holder_sm.credential_status());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_other_messages_from_request_sent_state() {
            let _setup = SetupMocks::init();

            let handle_connection = mock_connection();
            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialRequestSend(), handle_connection).unwrap();

            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialOffer(_credential_offer()), handle_connection).unwrap();
            assert_match!(HolderState::RequestSent(_), holder_sm.state);

            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialAck(_ack()), handle_connection).unwrap();
            assert_match!(HolderState::RequestSent(_), holder_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_issuer_handle_message_from_finished_state() {
            let _setup = SetupMocks::init();

            let handle_connection = mock_connection();
            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialRequestSend(), handle_connection).unwrap();
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::Credential(_credential()), handle_connection).unwrap();

            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialOffer(_credential_offer()), handle_connection).unwrap();
            assert_match!(HolderState::Finished(_), holder_sm.state);

            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::Credential(_credential()), handle_connection).unwrap();
            assert_match!(HolderState::Finished(_), holder_sm.state);

            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialAck(_ack()), handle_connection).unwrap();
            assert_match!(HolderState::Finished(_), holder_sm.state);
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
                    "key_5".to_string() => A2AMessage::CredentialAck(_ack()),
                    "key_6".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                assert!(holder.find_message_to_handle(messages).is_none());
            }
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_holder_find_message_to_handle_from_request_sent_state() {
            let _setup = SetupMocks::init();

            let holder = _holder_sm().to_request_sent_state();

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
                    "key_4".to_string() => A2AMessage::CredentialAck(_ack()),
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
                    "key_5".to_string() => A2AMessage::CredentialAck(_ack().set_thread_id("")),
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

        #[test]
        #[cfg(feature = "general_test")]
        fn test_holder_find_message_to_handle_from_finished_state() {
            let _setup = SetupMocks::init();

            let holder = _holder_sm().to_finished_state();

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

                assert!(holder.find_message_to_handle(messages).is_none());
            }
        }
    }

    mod get_state {
        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_get_state() {
            let _setup = SetupMocks::init();

            assert_eq!(VcxStateType::VcxStateRequestReceived as u32, _holder_sm().state());
            assert_eq!(VcxStateType::VcxStateOfferSent as u32, _holder_sm().to_request_sent_state().state());
            assert_eq!(VcxStateType::VcxStateAccepted as u32, _holder_sm().to_finished_state().state());
        }
    }

    mod get_tails_location {
        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_get_tails_location() {
            let _setup = SetupMocks::init();

            assert_eq!(Err(VcxErrorKind::NotReady), _holder_sm().get_tails_location().map_err(|e| e.kind()));
            assert_eq!(Err(VcxErrorKind::NotReady), _holder_sm().to_request_sent_state().get_tails_location().map_err(|e| e.kind()));
            assert_eq!(constants::TEST_TAILS_LOCATION, _holder_sm().to_finished_state().get_tails_location().unwrap());
        }
    }

    mod get_tails_hash {
        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_get_tails_hash() {
            let _setup = SetupMocks::init();

            assert_eq!(Err(VcxErrorKind::NotReady), _holder_sm().get_tails_hash().map_err(|e| e.kind()));
            assert_eq!(Err(VcxErrorKind::NotReady), _holder_sm().to_request_sent_state().get_tails_hash().map_err(|e| e.kind()));

            assert_eq!(constants::TEST_TAILS_HASH, _holder_sm().to_finished_state().get_tails_hash().unwrap());
        }
    }

    mod get_rev_reg_id {
        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_get_rev_reg_id() {
            let _setup = SetupMocks::init();

            assert_eq!(Err(VcxErrorKind::NotReady), _holder_sm().get_rev_reg_id().map_err(|e| e.kind()));
            assert_eq!(Err(VcxErrorKind::NotReady), _holder_sm().to_request_sent_state().get_rev_reg_id().map_err(|e| e.kind()));

            assert_eq!(constants::REV_REG_ID, _holder_sm().to_finished_state().get_rev_reg_id().unwrap());
        }
    }
}
