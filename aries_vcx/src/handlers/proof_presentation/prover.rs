use std::collections::HashMap;

use crate::error::prelude::*;
use crate::handlers::connection::connection::Connection;
use crate::protocols::SendClosure;
use crate::libindy::utils::anoncreds;
use crate::messages::a2a::A2AMessage;
use crate::messages::proof_presentation::presentation::Presentation;
use crate::messages::proof_presentation::presentation_proposal::{PresentationPreview, PresentationProposalData};
use crate::messages::proof_presentation::presentation_request::PresentationRequest;
use crate::protocols::proof_presentation::prover::messages::ProverMessages;
use crate::protocols::proof_presentation::prover::state_machine::{ProverSM, ProverState};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Prover {
    prover_sm: ProverSM,
}

impl Prover {
    pub fn create(source_id: &str) -> VcxResult<Prover> {
        trace!("Prover::create >>> source_id: {}", source_id);
        Ok(Prover {
            prover_sm: ProverSM::new(source_id.to_string()),
        })
    }

    pub fn create_from_request(source_id: &str, presentation_request: PresentationRequest) -> VcxResult<Prover> {
        trace!("Prover::create_from_request >>> source_id: {}, presentation_request: {:?}", source_id, presentation_request);
        Ok(Prover {
            prover_sm: ProverSM::from_request(presentation_request, source_id.to_string()),
        })
    }

    pub fn get_state(&self) -> ProverState { self.prover_sm.get_state() }

    pub fn presentation_status(&self) -> u32 {
        trace!("Prover::presentation_state >>>");
        self.prover_sm.presentation_status()
    }

    pub async fn retrieve_credentials(&self) -> VcxResult<String> {
        trace!("Prover::retrieve_credentials >>>");
        let presentation_request = self.presentation_request_data()?;
        anoncreds::libindy_prover_get_credentials_for_proof_req(&presentation_request).await
    }

    pub async fn generate_presentation(&mut self, credentials: String, self_attested_attrs: String) -> VcxResult<()> {
        trace!("Prover::generate_presentation >>> credentials: {}, self_attested_attrs: {:?}", credentials, self_attested_attrs);
        self.step(ProverMessages::PreparePresentation((credentials, self_attested_attrs)), None).await
    }

    pub fn generate_presentation_msg(&self) -> VcxResult<String> {
        trace!("Prover::generate_presentation_msg >>>");
        let proof = self.prover_sm.presentation()?.to_owned();
        Ok(json!(proof).to_string())
    }

    pub async fn set_presentation(&mut self, presentation: Presentation) -> VcxResult<()> {
        trace!("Prover::set_presentation >>>");
        self.step(ProverMessages::SetPresentation(presentation), None).await
    }

    pub async fn send_proposal(&mut self, proposal_data: PresentationProposalData, send_message: SendClosure) -> VcxResult<()> {
        trace!("Prover::send_proposal >>>");
        self.step(ProverMessages::PresentationProposalSend(proposal_data), Some(send_message)).await
    }

    pub async fn send_presentation(&mut self, send_message: SendClosure) -> VcxResult<()> {
        trace!("Prover::send_presentation >>>");
        self.step(ProverMessages::SendPresentation, Some(send_message)).await
    }

    pub fn progressable_by_message(&self) -> bool {
        self.prover_sm.progressable_by_message()
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        self.prover_sm.find_message_to_handle(messages)
    }

    pub async fn handle_message(&mut self, message: ProverMessages, send_message: Option<SendClosure>) -> VcxResult<()> {
        trace!("Prover::handle_message >>> message: {:?}", message);
        self.step(message, send_message).await
    }

    pub fn presentation_request_data(&self) -> VcxResult<String> {
        self.prover_sm.presentation_request()?.request_presentations_attach.content()
    }

    pub fn get_proof_request_attachment(&self) -> VcxResult<String> {
        let data = self.prover_sm.presentation_request()?.request_presentations_attach.content()?;
        let proof_request_data: serde_json::Value = serde_json::from_str(&data)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize {:?} into PresentationRequestData: {:?}", data, err)))?;
        Ok(proof_request_data.to_string())
    }

    pub fn get_source_id(&self) -> String { self.prover_sm.source_id() }

    pub fn get_thread_id(&self) -> VcxResult<String> { self.prover_sm.get_thread_id() }

    pub async fn step(&mut self,
                      message: ProverMessages,
                      send_message: Option<SendClosure>)
                      -> VcxResult<()>
    {
        self.prover_sm = self.prover_sm.clone().step(message, send_message).await?;
        Ok(())
    }

    pub async fn decline_presentation_request(&mut self, send_message: SendClosure, reason: Option<String>, proposal: Option<String>) -> VcxResult<()> {
        trace!("Prover::decline_presentation_request >>> reason: {:?}, proposal: {:?}", reason, proposal);
        match (reason, proposal) {
            (Some(reason), None) => {
                self.step(ProverMessages::RejectPresentationRequest(reason), Some(send_message)).await
            }
            (None, Some(proposal)) => {
                let presentation_preview: PresentationPreview = serde_json::from_str(&proposal)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot serialize Presentation Preview: {:?}", err)))?;

                self.step(ProverMessages::ProposePresentation(presentation_preview), Some(send_message)).await
            }
            (None, None) => {
                return Err(VcxError::from_msg(VcxErrorKind::InvalidOption, "Either `reason` or `proposal` parameter must be specified."));
            }
            (Some(_), Some(_)) => {
                return Err(VcxError::from_msg(VcxErrorKind::InvalidOption, "Only one of `reason` or `proposal` parameters must be specified."));
            }
        }
    }

    pub async fn update_state(&mut self, connection: &Connection) -> VcxResult<ProverState> {
        trace!("Prover::update_state >>> ");
        if !self.progressable_by_message() { return Ok(self.get_state()); }
        let send_message = connection.send_message_closure()?;

        let messages = connection.get_messages().await?;
        if let Some((uid, msg)) = self.find_message_to_handle(messages) {
            self.step(msg.into(), Some(send_message)).await?;
            connection.update_message_status(&uid).await?;
        }
        Ok(self.get_state())
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use crate::error::prelude::*;
    use crate::handlers::connection::connection::Connection;
    use crate::messages::a2a::A2AMessage;

    pub async fn get_proof_request_messages(connection: &Connection) -> VcxResult<String> {
        let presentation_requests: Vec<A2AMessage> = connection.get_messages()
            .await?
            .into_iter()
            .filter_map(|(_, message)| {
                match message {
                    A2AMessage::PresentationRequest(_) => Some(message),
                    _ => None
                }
            })
            .collect();

        Ok(json!(presentation_requests).to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::libindy::utils::anoncreds::test_utils::{create_and_store_credential, create_proof};
    use crate::messages::proof_presentation::presentation_request::{PresentationRequest, PresentationRequestData};
    use crate::utils;
    use crate::utils::constants::TAILS_DIR;
    use crate::utils::devsetup::*;
    use crate::utils::get_temp_dir_path;

    use super::*;

    #[tokio::test]
    #[cfg(feature = "pool_tests")]
    async fn test_retrieve_credentials() {
        let _setup = SetupWithWalletAndAgency::init().await;

        create_and_store_credential(utils::constants::DEFAULT_SCHEMA_ATTRS, false);
        let (_, _, req, _) = create_proof();

        let pres_req_data: PresentationRequestData = serde_json::from_str(&req).unwrap();
        let proof_req = PresentationRequest::create().set_request_presentations_attach(&pres_req_data).unwrap();
        let proof: Prover = Prover::create_from_request("1", proof_req).unwrap();

        let retrieved_creds = proof.retrieve_credentials().unwrap();
        assert!(retrieved_creds.len() > 500);
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_retrieve_credentials_empty() {
        let _setup = SetupWithWalletAndAgency::init().await;

        let mut req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({}),
           "requested_predicates": json!({}),
        });

        let pres_req_data: PresentationRequestData = serde_json::from_str(&req.to_string()).unwrap();
        let proof_req = PresentationRequest::create().set_request_presentations_attach(&pres_req_data).unwrap();
        let proof: Prover = Prover::create_from_request("1", proof_req).unwrap();

        let retrieved_creds = proof.retrieve_credentials().unwrap();
        assert_eq!(retrieved_creds, "{}".to_string());

        req["requested_attributes"]["address1_1"] = json!({"name": "address1"});
        let pres_req_data: PresentationRequestData = serde_json::from_str(&req.to_string()).unwrap();
        let proof_req = PresentationRequest::create().set_request_presentations_attach(&pres_req_data).unwrap();
        let proof: Prover = Prover::create_from_request("2", proof_req).unwrap();

        let retrieved_creds = proof.retrieve_credentials().unwrap();
        assert_eq!(retrieved_creds, json!({"attrs":{"address1_1":[]}}).to_string());
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_case_for_proof_req_doesnt_matter_for_retrieve_creds() {
        let setup = SetupWithWalletAndAgency::init().await;
        create_and_store_credential(utils::constants::DEFAULT_SCHEMA_ATTRS, false);

        let mut req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({
               "zip_1": json!({
                   "name":"zip",
                   "restrictions": [json!({ "issuer_did": setup.institution_did })]
               })
           }),
           "requested_predicates": json!({}),
        });

        let pres_req_data: PresentationRequestData = serde_json::from_str(&req.to_string()).unwrap();
        let proof_req = PresentationRequest::create().set_request_presentations_attach(&pres_req_data).unwrap();
        let proof: Prover = Prover::create_from_request("1", proof_req).unwrap();

        // All lower case
        let retrieved_creds = proof.retrieve_credentials().unwrap();
        assert!(retrieved_creds.contains(r#""zip":"84000""#));
        let ret_creds_as_value: serde_json::Value = serde_json::from_str(&retrieved_creds).unwrap();
        assert_eq!(ret_creds_as_value["attrs"]["zip_1"][0]["cred_info"]["attrs"]["zip"], "84000");

        // First letter upper
        req["requested_attributes"]["zip_1"]["name"] = json!("Zip");
        let pres_req_data: PresentationRequestData = serde_json::from_str(&req.to_string()).unwrap();
        let proof_req = PresentationRequest::create().set_request_presentations_attach(&pres_req_data).unwrap();
        let proof: Prover = Prover::create_from_request("2", proof_req).unwrap();
        let retrieved_creds2 = proof.retrieve_credentials().unwrap();
        assert!(retrieved_creds2.contains(r#""zip":"84000""#));

        // Entire word upper
        req["requested_attributes"]["zip_1"]["name"] = json!("ZIP");
        let pres_req_data: PresentationRequestData = serde_json::from_str(&req.to_string()).unwrap();
        let proof_req = PresentationRequest::create().set_request_presentations_attach(&pres_req_data).unwrap();
        let proof: Prover = Prover::create_from_request("1", proof_req).unwrap();
        let retrieved_creds3 = proof.retrieve_credentials().unwrap();
        assert!(retrieved_creds3.contains(r#""zip":"84000""#));
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_retrieve_credentials_fails_with_no_proof_req() {
        let _setup = SetupLibraryWallet::init();

        let proof_req = PresentationRequest::create();
        let proof = Prover::create_from_request("1", proof_req).unwrap();
        assert_eq!(proof.retrieve_credentials().unwrap_err().kind(), VcxErrorKind::InvalidJson);
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_generate_proof() {
        let setup = SetupWithWalletAndAgency::init().await;

        create_and_store_credential(utils::constants::DEFAULT_SCHEMA_ATTRS, true);
        let to = time::get_time().sec;
        let indy_proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "0.1",
            "requested_attributes": {
                "address1_1": {
                    "name": "address1",
                    "restrictions": [{"issuer_did": setup.institution_did}],
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
        let mut proof: Prover = Prover::create_from_request("1", proof_req).unwrap();

        let all_creds: serde_json::Value = serde_json::from_str(&proof.retrieve_credentials().unwrap()).unwrap();
        let selected_credentials: serde_json::Value = json!({
           "attrs":{
              "address1_1": {
                "credential": all_creds["attrs"]["address1_1"][0],
                "tails_file": get_temp_dir_path(TAILS_DIR).to_str().unwrap().to_string()
              },
              "zip_2": {
                "credential": all_creds["attrs"]["zip_2"][0],
                "tails_file": get_temp_dir_path(TAILS_DIR).to_str().unwrap().to_string()
              },
           }
        });

        let self_attested: serde_json::Value = json!({
              "self_attested_attr_3":"attested_val"
        });

        let generated_proof = proof.generate_presentation(selected_credentials.to_string(), self_attested.to_string()).await;
        assert!(generated_proof.is_ok());
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_generate_self_attested_proof() {
        let _setup = SetupWithWalletAndAgency::init().await;

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
        let mut proof: Prover = Prover::create_from_request("1", proof_req).unwrap();

        let selected_credentials: serde_json::Value = json!({});
        let self_attested: serde_json::Value = json!({
              "address1_1":"attested_address",
              "zip_2": "attested_zip"
        });
        let generated_proof = proof.generate_presentation(selected_credentials.to_string(), self_attested.to_string()).await;
        assert!(generated_proof.is_ok());
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_generate_proof_with_predicates() {
        let setup = SetupWithWalletAndAgency::init().await;

        create_and_store_credential(utils::constants::DEFAULT_SCHEMA_ATTRS, true);
        let to = time::get_time().sec;
        let indy_proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "0.1",
            "requested_attributes": {
                "address1_1": {
                    "name": "address1",
                    "restrictions": [{"issuer_did": setup.institution_did}],
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
        let mut proof: Prover = Prover::create_from_request("1", proof_req).unwrap();

        let all_creds: serde_json::Value = serde_json::from_str(&proof.retrieve_credentials().unwrap()).unwrap();
        let selected_credentials: serde_json::Value = json!({
           "attrs":{
              "address1_1": {
                "credential": all_creds["attrs"]["address1_1"][0],
                "tails_file": get_temp_dir_path(TAILS_DIR).to_str().unwrap().to_string()
              },
              "zip_2": {
                "credential": all_creds["attrs"]["zip_2"][0],
                "tails_file": get_temp_dir_path(TAILS_DIR).to_str().unwrap().to_string()
              },
              "zip_3": {
                "credential": all_creds["attrs"]["zip_3"][0],
                "tails_file": get_temp_dir_path(TAILS_DIR).to_str().unwrap().to_string()
              },
           },
        });
        let self_attested: serde_json::Value = json!({
              "self_attested_attr_3":"attested_val"
        });
        let generated_proof = proof.generate_presentation(selected_credentials.to_string(), self_attested.to_string()).await;
        assert!(generated_proof.is_ok());
    }
}
