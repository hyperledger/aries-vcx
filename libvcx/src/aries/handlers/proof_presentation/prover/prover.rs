use std::convert::TryInto;

use ::{connection, settings};
use error::prelude::*;
use messages::proofs::proof_message::ProofMessage;
use utils::libindy::anoncreds;
use aries::handlers::proof_presentation::prover::messages::ProverMessages;
use aries::messages::a2a::A2AMessage;
use aries::messages::proof_presentation::presentation::Presentation;
use aries::messages::proof_presentation::presentation_proposal::PresentationPreview;
use aries::messages::proof_presentation::presentation_request::PresentationRequest;
use aries::handlers::proof_presentation::prover::state_machine::ProverSM;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Prover {
    prover_sm: ProverSM
}

impl Prover {
    pub fn create(source_id: &str, presentation_request: PresentationRequest) -> VcxResult<Prover> {
        trace!("Prover::create >>> source_id: {}, presentation_request: {:?}", source_id, presentation_request);
        Ok(Prover {
            prover_sm: ProverSM::new(presentation_request, source_id.to_string()),
        })
    }

    pub fn state(&self) -> u32 { self.prover_sm.state() }

    pub fn presentation_status(&self) -> u32 {
        trace!("Prover::presentation_state >>>");
        self.prover_sm.presentation_status()
    }

    pub fn retrieve_credentials(&self) -> VcxResult<String> {
        trace!("Prover::retrieve_credentials >>>");
        let presentation_request = self.prover_sm.presentation_request().request_presentations_attach.content()?;
        anoncreds::libindy_prover_get_credentials_for_proof_req(&presentation_request)
    }

    pub fn generate_presentation(&mut self, credentials: String, self_attested_attrs: String) -> VcxResult<()> {
        trace!("Prover::generate_presentation >>> credentials: {}, self_attested_attrs: {:?}", credentials, self_attested_attrs);
        self.step(ProverMessages::PreparePresentation((credentials, self_attested_attrs)))
    }

    pub fn generate_presentation_msg(&self) -> VcxResult<String> {
        trace!("Prover::generate_presentation_msg >>>");
        let proof = self.prover_sm.presentation()?.to_owned();
        Ok(json!(proof).to_string())
    }

    pub fn set_presentation(&mut self, presentation: Presentation) -> VcxResult<()> {
        trace!("Prover::set_presentation >>>");
        self.step(ProverMessages::SetPresentation(presentation))
    }

    pub fn send_presentation(&mut self, connection_handle: u32) -> VcxResult<()> {
        trace!("Prover::send_presentation >>>");
        self.step(ProverMessages::SendPresentation(connection_handle))
    }

    pub fn update_state(&mut self, message: Option<&str>, connection_handle: Option<u32>) -> VcxResult<()> {
        trace!("Prover::update_state >>> connection_handle: {:?}, message: {:?}", connection_handle, message);

        if !self.prover_sm.has_transitions() { 
            trace!("Prover::update_state >> found no available transition");
            return Ok(());
        }

        let connection_handle = connection_handle.unwrap_or(self.prover_sm.connection_handle()?);
        self.prover_sm.set_connection_handle(connection_handle);

        if let Some(message_) = message {
            return self.update_state_with_message(message_);
        }

        let messages = connection::get_messages(connection_handle)?;
        trace!("Prover::update_state >>> found messages: {:?}", messages);

        if let Some((uid, message)) = self.prover_sm.find_message_to_handle(messages) {
            self.handle_message(message.into())?;
            connection::update_message_status(connection_handle, uid)?;
        };

        Ok(())
    }

    pub fn update_state_with_message(&mut self, message: &str) -> VcxResult<()> {
        trace!("Prover::update_state_with_message >>> message: {:?}", message);

        let a2a_message: A2AMessage = ::serde_json::from_str(&message)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidOption, format!("Cannot updated state with message: Message deserialization failed: {:?}", err)))?;

        self.handle_message(a2a_message.into())?;

        Ok(())
    }

    pub fn handle_message(&mut self, message: ProverMessages) -> VcxResult<()> {
        trace!("Prover::handle_message >>> message: {:?}", message);
        self.step(message)
    }

    pub fn get_presentation_request(connection_handle: u32, msg_id: &str) -> VcxResult<PresentationRequest> {
        trace!("Prover::get_presentation_request >>> connection_handle: {:?}, msg_id: {:?}", connection_handle, msg_id);

        let message = connection::get_message_by_id(connection_handle, msg_id.to_string())?;

        let presentation_request: PresentationRequest = match message {
            A2AMessage::PresentationRequest(presentation_request) => presentation_request,
            msg => {
                return Err(VcxError::from_msg(VcxErrorKind::InvalidMessages,
                                              format!("Message of different type was received: {:?}", msg)));
            }
        };

        Ok(presentation_request)
    }

    pub fn get_presentation_request_messages(connection_handle: u32) -> VcxResult<Vec<PresentationRequest>> {
        trace!("Prover::get_presentation_request_messages >>> connection_handle: {:?}", connection_handle);

        let presentation_requests: Vec<PresentationRequest> =
            connection::get_messages(connection_handle)?
                .into_iter()
                .filter_map(|(_, message)| {
                    match message {
                        A2AMessage::PresentationRequest(presentation_request) => Some(presentation_request),
                        _ => None
                    }
                })
                .collect();

        Ok(presentation_requests)
    }

    pub fn get_source_id(&self) -> String { self.prover_sm.source_id() }

    pub fn step(&mut self, message: ProverMessages) -> VcxResult<()> {
        self.prover_sm = self.prover_sm.clone().step(message)?;
        Ok(())
    }

    pub fn decline_presentation_request(&mut self, connection_handle: u32, reason: Option<String>, proposal: Option<String>) -> VcxResult<()> {
        trace!("Prover::decline_presentation_request >>> connection_handle: {}, reason: {:?}, proposal: {:?}", connection_handle, reason, proposal);
        match (reason, proposal) {
            (Some(reason), None) => {
                self.step(ProverMessages::RejectPresentationRequest((connection_handle, reason)))
            }
            (None, Some(proposal)) => {
                let presentation_preview: PresentationPreview = serde_json::from_str(&proposal)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot serialize Presentation Preview: {:?}", err)))?;

                self.step(ProverMessages::ProposePresentation((connection_handle, presentation_preview)))
            }
            (None, None) => {
                return Err(VcxError::from_msg(VcxErrorKind::InvalidOption, "Either `reason` or `proposal` parameter must be specified."));
            }
            (Some(_), Some(_)) => {
                return Err(VcxError::from_msg(VcxErrorKind::InvalidOption, "Only one of `reason` or `proposal` parameters must be specified."));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::devsetup::*;
    use utils::get_temp_dir_path;
    use utils::constants::TEST_TAILS_FILE;
    use aries::messages::proof_presentation::presentation_request::{PresentationRequest, PresentationRequestData};

    #[test]
    #[cfg(feature = "pool_tests")]
    fn test_retrieve_credentials() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        ::utils::libindy::anoncreds::tests::create_and_store_credential(::utils::constants::DEFAULT_SCHEMA_ATTRS, false);
        let (_, _, req, _) = ::utils::libindy::anoncreds::tests::create_proof();

        let pres_req_data: PresentationRequestData = serde_json::from_str(&req).unwrap();
        let mut proof_req = PresentationRequest::create().set_request_presentations_attach(&pres_req_data).unwrap();
        let mut proof: Prover = Prover::create("1", proof_req).unwrap();

        let retrieved_creds = proof.retrieve_credentials().unwrap();
        assert!(retrieved_creds.len() > 500);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_retrieve_credentials_emtpy() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let mut req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({}),
           "requested_predicates": json!({}),
        });

        let pres_req_data: PresentationRequestData = serde_json::from_str(&req.to_string()).unwrap();
        let proof_req = PresentationRequest::create().set_request_presentations_attach(&pres_req_data).unwrap();
        let proof: Prover = Prover::create("1", proof_req).unwrap();

        let retrieved_creds = proof.retrieve_credentials().unwrap();
        assert_eq!(retrieved_creds, "{}".to_string());

        req["requested_attributes"]["address1_1"] = json!({"name": "address1"});
        let pres_req_data: PresentationRequestData = serde_json::from_str(&req.to_string()).unwrap();
        let proof_req = PresentationRequest::create().set_request_presentations_attach(&pres_req_data).unwrap();
        let proof: Prover = Prover::create("2", proof_req).unwrap();

        let retrieved_creds = proof.retrieve_credentials().unwrap();
        assert_eq!(retrieved_creds, json!({"attrs":{"address1_1":[]}}).to_string());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_case_for_proof_req_doesnt_matter_for_retrieve_creds() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        ::utils::libindy::anoncreds::tests::create_and_store_credential(::utils::constants::DEFAULT_SCHEMA_ATTRS, false);
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let mut req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({
               "zip_1": json!({
                   "name":"zip",
                   "restrictions": [json!({ "issuer_did": did })]
               })
           }),
           "requested_predicates": json!({}),
        });

        let pres_req_data: PresentationRequestData = serde_json::from_str(&req.to_string()).unwrap();
        let proof_req = PresentationRequest::create().set_request_presentations_attach(&pres_req_data).unwrap();
        let proof: Prover = Prover::create("1", proof_req).unwrap();

        // All lower case
        let retrieved_creds = proof.retrieve_credentials().unwrap();
        assert!(retrieved_creds.contains(r#""zip":"84000""#));
        let ret_creds_as_value: serde_json::Value = serde_json::from_str(&retrieved_creds).unwrap();
        assert_eq!(ret_creds_as_value["attrs"]["zip_1"][0]["cred_info"]["attrs"]["zip"], "84000");

        // First letter upper
        req["requested_attributes"]["zip_1"]["name"] = json!("Zip");
        let pres_req_data: PresentationRequestData = serde_json::from_str(&req.to_string()).unwrap();
        let proof_req = PresentationRequest::create().set_request_presentations_attach(&pres_req_data).unwrap();
        let proof: Prover = Prover::create("2", proof_req).unwrap();
        let retrieved_creds2 = proof.retrieve_credentials().unwrap();
        assert!(retrieved_creds2.contains(r#""zip":"84000""#));

        // Entire word upper
        req["requested_attributes"]["zip_1"]["name"] = json!("ZIP");
        let pres_req_data: PresentationRequestData = serde_json::from_str(&req.to_string()).unwrap();
        let proof_req = PresentationRequest::create().set_request_presentations_attach(&pres_req_data).unwrap();
        let proof: Prover = Prover::create("1", proof_req).unwrap();
        let retrieved_creds3 = proof.retrieve_credentials().unwrap();
        assert!(retrieved_creds3.contains(r#""zip":"84000""#));
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_retrieve_credentials_fails_with_no_proof_req() {
        let _setup = SetupLibraryWallet::init();

        let proof_req = PresentationRequest::create();
        let proof = Prover::create("1", proof_req).unwrap();
        assert_eq!(proof.retrieve_credentials().unwrap_err().kind(), VcxErrorKind::InvalidJson);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_generate_proof() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        ::utils::libindy::anoncreds::tests::create_and_store_credential(::utils::constants::DEFAULT_SCHEMA_ATTRS, true);
        let to = time::get_time().sec;
        let indy_proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "0.1",
            "requested_attributes": {
                "address1_1": {
                    "name": "address1",
                    "restrictions": [{"issuer_did": did}],
                    "non_revoked":  {"from": 123, "to": to}
                },
                "zip_2": { "name": "zip" }
            },
            "self_attested_attr_3": json!({
                   "name":"self_attested_attr",
             }),
            "requested_predicates": {},
            "non_revoked": {"from": 098, "to": to}
        }).to_string();

        let pres_req_data: PresentationRequestData = serde_json::from_str(&indy_proof_req).unwrap();
        let proof_req = PresentationRequest::create().set_request_presentations_attach(&pres_req_data).unwrap();
        let mut proof: Prover = Prover::create("1", proof_req).unwrap();

        let all_creds: serde_json::Value = serde_json::from_str(&proof.retrieve_credentials().unwrap()).unwrap();
        let selected_credentials: serde_json::Value = json!({
           "attrs":{
              "address1_1": {
                "credential": all_creds["attrs"]["address1_1"][0],
                "tails_file": get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()
              },
              "zip_2": {
                "credential": all_creds["attrs"]["zip_2"][0],
                "tails_file": get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()
              },
           },
           "predicates":{ }
        });

        let self_attested: serde_json::Value = json!({
              "self_attested_attr_3":"attested_val"
        });

        let generated_proof = proof.generate_presentation(selected_credentials.to_string(), self_attested.to_string());
        assert!(generated_proof.is_ok());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_generate_self_attested_proof() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let indy_proof_req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({
               "address1_1": json!({
                   "name":"address1",
               }),
               "zip_2": json!({
                   "name":"zip",
               }),
           }),
           "requested_predicates": json!({}),
        }).to_string();

        let pres_req_data: PresentationRequestData = serde_json::from_str(&indy_proof_req).unwrap();
        let proof_req = PresentationRequest::create().set_request_presentations_attach(&pres_req_data).unwrap();
        let mut proof: Prover = Prover::create("1", proof_req).unwrap();

        let selected_credentials: serde_json::Value = json!({});
        let self_attested: serde_json::Value = json!({
              "address1_1":"attested_address",
              "zip_2": "attested_zip"
        });
        let generated_proof = proof.generate_presentation(selected_credentials.to_string(), self_attested.to_string());
        assert!(generated_proof.is_ok());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_generate_proof_with_predicates() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        ::utils::libindy::anoncreds::tests::create_and_store_credential(::utils::constants::DEFAULT_SCHEMA_ATTRS, true);
        let to = time::get_time().sec;
        let indy_proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "0.1",
            "requested_attributes": {
                "address1_1": {
                    "name": "address1",
                    "restrictions": [{"issuer_did": did}],
                    "non_revoked":  {"from": 123, "to": to}
                },
                "zip_2": { "name": "zip" }
            },
            "self_attested_attr_3": json!({
                   "name":"self_attested_attr",
             }),
            "requested_predicates": json!({
                "zip_3": {"name":"zip", "p_type":">=", "p_value":18}
            }),
            "non_revoked": {"from": 098, "to": to}
        }).to_string();

        let pres_req_data: PresentationRequestData = serde_json::from_str(&indy_proof_req).unwrap();
        let proof_req = PresentationRequest::create().set_request_presentations_attach(&pres_req_data).unwrap();
        let mut proof: Prover = Prover::create("1", proof_req).unwrap();

        let all_creds: serde_json::Value = serde_json::from_str(&proof.retrieve_credentials().unwrap()).unwrap();
        let selected_credentials: serde_json::Value = json!({
           "attrs":{
              "address1_1": {
                "credential": all_creds["attrs"]["address1_1"][0],
                "tails_file": get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()
              },
              "zip_2": {
                "credential": all_creds["attrs"]["zip_2"][0],
                "tails_file": get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()
              },
           },
           "predicates":{ 
               "zip_3": {
                "credential": all_creds["attrs"]["zip_3"][0],
               }
           }
        });
        let self_attested: serde_json::Value = json!({
              "self_attested_attr_3":"attested_val"
        });
        let generated_proof = proof.generate_presentation(selected_credentials.to_string(), self_attested.to_string());
        assert!(generated_proof.is_ok());
    }
}
