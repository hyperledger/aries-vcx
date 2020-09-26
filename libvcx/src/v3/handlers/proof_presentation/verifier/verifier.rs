use std::convert::TryInto;

use ::{connection};
use error::prelude::*;
use v3::handlers::proof_presentation::verifier::messages::VerifierMessages;
use v3::handlers::proof_presentation::verifier::states::VerifierSM;
use v3::messages::a2a::A2AMessage;
use v3::messages::proof_presentation::presentation::Presentation;
use v3::messages::proof_presentation::presentation_request::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Verifier {
    verifier_sm: VerifierSM
}

impl Verifier {
    pub fn create(source_id: String,
                  requested_attrs: String,
                  requested_predicates: String,
                  revocation_details: String,
                  name: String) -> VcxResult<Verifier> {
        trace!("Verifier::create >>> source_id: {:?}, requested_attrs: {:?}, requested_predicates: {:?}, revocation_details: {:?}, name: {:?}",
               source_id, requested_attrs, requested_predicates, revocation_details, name);

        let presentation_request =
            PresentationRequestData::create()
                .set_name(name)
                .set_requested_attributes(requested_attrs)?
                .set_requested_predicates(requested_predicates)?
                .set_not_revoked_interval(revocation_details)?
                .set_nonce()?;

        Ok(Verifier {
            verifier_sm: VerifierSM::new(presentation_request, source_id),
        })
    }

    pub fn get_source_id(&self) -> String { self.verifier_sm.source_id() }

    pub fn state(&self) -> u32 {
        trace!("Verifier::state >>>");
        self.verifier_sm.state()
    }

    pub fn presentation_status(&self) -> u32 {
        trace!("Verifier::presentation_state >>>");
        self.verifier_sm.presentation_status()
    }

    pub fn update_state(&mut self, message: Option<&str>, connection_handle: Option<u32>) -> VcxResult<()> {
        trace!("Verifier::update_state >>> message: {:?}", message);

        if !self.verifier_sm.has_transitions() { return Ok(()); }

        let connection_handle = connection_handle.unwrap_or(self.verifier_sm.connection_handle()?);
        self.verifier_sm.set_connection_handle(connection_handle);

        if let Some(message_) = message {
            return self.update_state_with_message(message_);
        }

        let messages = connection::get_messages(connection_handle)?;

        if let Some((uid, message)) = self.verifier_sm.find_message_to_handle(messages) {
            self.handle_message(message.into())?;
            connection::update_message_status(connection_handle, uid)?;
        };

        Ok(())
    }

    pub fn update_state_with_message(&mut self, message: &str) -> VcxResult<()> {
        trace!("Verifier::update_state_with_message >>> message: {:?}", message);

        let message: A2AMessage = ::serde_json::from_str(&message)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidOption, format!("Cannot update state with message: Message deserialization failed: {:?}", err)))?;

        self.handle_message(message.into())?;

        Ok(())
    }

    pub fn handle_message(&mut self, message: VerifierMessages) -> VcxResult<()> {
        trace!("Verifier::handle_message >>> message: {:?}", message);
        self.step(message)
    }

    pub fn verify_presentation(&mut self, presentation: Presentation) -> VcxResult<()> {
        trace!("Verifier::verify_presentation >>> presentation: {:?}", presentation);
        self.step(VerifierMessages::VerifyPresentation(presentation))
    }

    pub fn send_presentation_request(&mut self, connection_handle: u32) -> VcxResult<()> {
        trace!("Verifier::send_presentation_request >>> connection_handle: {:?}", connection_handle);
        self.step(VerifierMessages::SendPresentationRequest(connection_handle))
    }

    pub fn generate_presentation_request_msg(&self) -> VcxResult<String> {
        trace!("Verifier::generate_presentation_request_msg >>>");

        let proof_request = self.verifier_sm.presentation_request()?;

        Ok(json!(proof_request).to_string())
    }

    pub fn get_presentation(&self) -> VcxResult<String> {
        trace!("Verifier::get_presentation >>>");

        let proof = self.verifier_sm.presentation()?;
        Ok(json!(proof).to_string())
    }

    pub fn step(&mut self, message: VerifierMessages) -> VcxResult<()> {
        self.verifier_sm = self.verifier_sm.clone().step(message)?;
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use api::VcxStateType;
    use connection::tests::build_test_connection_inviter_requested;
    use utils::constants::{REQUESTED_ATTRS, REQUESTED_PREDICATES, PROOF_REJECT_RESPONSE_STR_V2};
    use utils::devsetup::*;
    use settings;

    use super::*;
    use utils::mockdata::mockdata_proof::ARIES_PROOF_PRESENTATION;
    use v3::messages::status::Status;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_proof_validation_with_predicate() {
        let _setup = SetupAriesMocks::init();

        let connection_handle = build_test_connection_inviter_requested();

        let mut proof = Verifier::create("1".to_string(),
                                         REQUESTED_ATTRS.to_owned(),
                                         REQUESTED_PREDICATES.to_owned(),
                                         r#"{"support_revocation":false}"#.to_string(),
                                         "Optional".to_owned()).unwrap();

        proof.send_presentation_request(connection_handle).unwrap();

        assert_eq!(proof.state(), VcxStateType::VcxStateOfferSent as u32);

        proof.update_state_with_message(ARIES_PROOF_PRESENTATION).unwrap();

        assert_eq!(proof.state(), VcxStateType::VcxStateAccepted as u32);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_proof_self_attested_proof_validation() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();
        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
        settings::set_config_value(settings::CONFIG_PROTOCOL_TYPE, "4.0");

        let connection_handle = build_test_connection_inviter_requested();

        let mut ver_proof = Verifier::create("1".to_string(),
                                         json!([
                                            json!({
                                                "name":"address1",
                                                "self_attest_allowed": true,
                                            }),
                                            json!({
                                                "name":"zip",
                                                "self_attest_allowed": true,
                                            }),
                                         ]).to_string(),
                                         json!([]).to_string(),
                                         r#"{"support_revocation":false}"#.to_string(),
                                         "Optional".to_owned()).unwrap();

        let proof_req_json = serde_json::to_string(ver_proof.verifier_sm.presentation_request_data().unwrap()).unwrap();

        ::utils::libindy::anoncreds::libindy_prover_get_credentials_for_proof_req(&proof_req_json).unwrap();

        let prover_proof_json = ::utils::libindy::anoncreds::libindy_prover_create_proof(
            &proof_req_json,
            &json!({
              "self_attested_attributes":{
                 "attribute_0": "my_self_attested_address",
                 "attribute_1": "my_self_attested_zip"
              },
              "requested_attributes":{},
              "requested_predicates":{}
            }).to_string(),
            "main",
            &json!({}).to_string(),
            &json!({}).to_string(),
            None).unwrap();

        ver_proof.send_presentation_request(connection_handle).unwrap();
        assert_eq!(ver_proof.state(), VcxStateType::VcxStateOfferSent as u32);

        let presentation = Presentation::create().set_presentations_attach(prover_proof_json).unwrap();
        ver_proof.verify_presentation(presentation);
        assert_eq!(ver_proof.state(), VcxStateType::VcxStateAccepted as u32);
    }

    // TODO: Should fail but passes for some reason
    #[test]
    #[cfg(feature = "general_test")]
    // #[cfg(feature = "to_restore")]
    fn test_proof_resetrictions() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();
        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");
        settings::set_config_value(settings::CONFIG_PROTOCOL_TYPE, "4.0");

        let connection_handle = build_test_connection_inviter_requested();

        let mut ver_proof = Verifier::create("1".to_string(),
                                         json!([
                                            json!({
                                                "name":"address1",
                                                "restrictions": [{ "issuer_did": "Not Here" }],
                                            }),
                                            json!({
                                                "name":"zip",
                                            }),
                                            json!({
                                                "name":"self_attest",
                                                "self_attest_allowed": true,
                                            }),
                                         ]).to_string(),
                                         json!([]).to_string(),
                                         r#"{"support_revocation":true}"#.to_string(),
                                         "Optional".to_owned()).unwrap();

        let proof_req_json = serde_json::to_string(ver_proof.verifier_sm.presentation_request_data().unwrap()).unwrap();
        println!("{:?}", proof_req_json);

        ::utils::libindy::anoncreds::libindy_prover_get_credentials_for_proof_req(&proof_req_json).unwrap();

        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let (schema_id, schema_json, cred_def_id, cred_def_json, _offer, _req, _req_meta, cred_id, _, _)
            = ::utils::libindy::anoncreds::tests::create_and_store_credential(::utils::constants::DEFAULT_SCHEMA_ATTRS, false);
        let cred_def_json: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();

        let prover_proof_json = ::utils::libindy::anoncreds::libindy_prover_create_proof(
            &proof_req_json,
            &json!({
                "self_attested_attributes":{
                   "attribute_2": "my_self_attested_val"
                },
                "requested_attributes":{
                   "attribute_0": {"cred_id": cred_id, "revealed": true},
                   "attribute_1": {"cred_id": cred_id, "revealed": true}
                },
                "requested_predicates":{}
            }).to_string(),
            "main",
            &json!({schema_id: schema_json}).to_string(),
            &json!({cred_def_id: cred_def_json}).to_string(),
            None).unwrap();
        println!("{:?}", prover_proof_json);

        ver_proof.send_presentation_request(connection_handle).unwrap();
        assert_eq!(ver_proof.state(), VcxStateType::VcxStateOfferSent as u32);

        let presentation = Presentation::create().set_presentations_attach(prover_proof_json).unwrap();
        ver_proof.verify_presentation(presentation);
        assert_eq!(ver_proof.state(), VcxStateType::VcxStateNone as u32);
        assert_eq!(ver_proof.presentation_status(), 2);
        // TODO: Remove restriction, should pass
    }


    #[test]
    #[cfg(feature = "general_test")]
    fn test_send_presentation_request() {
        let _setup = SetupAriesMocks::init();

        let connection_handle = build_test_connection_inviter_requested();

        let mut proof = Verifier::create("1".to_string(),
                                         REQUESTED_ATTRS.to_owned(),
                                         REQUESTED_PREDICATES.to_owned(),
                                         r#"{"support_revocation":false}"#.to_string(),
                                         "Optional".to_owned()).unwrap();

        proof.send_presentation_request(connection_handle).unwrap();

        assert_eq!(proof.state(), VcxStateType::VcxStateOfferSent as u32);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_update_state_with_reject_message() {
        let _setup = SetupAriesMocks::init();

        let connection_handle = build_test_connection_inviter_requested();

        let mut proof = Verifier::create("1".to_string(),
                                         REQUESTED_ATTRS.to_owned(),
                                         REQUESTED_PREDICATES.to_owned(),
                                         r#"{"support_revocation":false}"#.to_string(),
                                         "Optional".to_owned()).unwrap();

        proof.send_presentation_request(connection_handle);

        proof.update_state(Some(PROOF_REJECT_RESPONSE_STR_V2), Some(connection_handle)).unwrap();
        assert_eq!(proof.state(), VcxStateType::VcxStateNone as u32);
    }
}
