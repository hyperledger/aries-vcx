pub const SERIALIZE_VERSION: &'static str = "2.0";

#[cfg(test)]
pub mod test {
    use aries_vcx::agency_client::payload::PayloadKinds;
    use aries_vcx::settings;

    use aries_vcx::init::{create_agency_client_for_main_wallet, init_issuer_config, open_as_main_wallet};
    use aries_vcx::error::{VcxError, VcxErrorKind, VcxResult};

    use aries_vcx::messages::a2a::A2AMessage;
    use aries_vcx::messages::issuance::credential_offer::CredentialOffer;
    use aries_vcx::libindy::utils::wallet::*;
    use aries_vcx::libindy::utils::anoncreds;
    use aries_vcx::utils::constants;
    use aries_vcx::utils::devsetup::*;
    use aries_vcx::utils::provision::{AgencyClientConfig, AgentProvisionConfig, provision_cloud_agent};
    use aries_vcx::handlers::connection::connection::{Connection, ConnectionState};
    use aries_vcx::handlers::connection::invitee::state_machine::InviteeState;
    use aries_vcx::handlers::connection::inviter::state_machine::InviterState;
    use aries_vcx::handlers::issuance::credential_def::CredentialDef;
    use aries_vcx::handlers::issuance::issuer::issuer::{Issuer, IssuerConfig as AriesIssuerConfig, IssuerState};
    use aries_vcx::handlers::issuance::holder::holder::{Holder, HolderState};
    use aries_vcx::handlers::issuance::holder::get_credential_offer_messages;
    use aries_vcx::handlers::issuance::schema::schema::{Schema, SchemaData};
    use aries_vcx::handlers::issuance::credential_def::PublicEntityStateType;
    use aries_vcx::handlers::proof_presentation::verifier::verifier::{Verifier, VerifierState};
    use aries_vcx::handlers::proof_presentation::prover::prover::{Prover, ProverState};
    use aries_vcx::handlers::proof_presentation::prover::get_proof_request_messages;
    use aries_vcx::messages::proof_presentation::presentation_request::PresentationRequest;

    #[derive(Debug)]
    pub struct VcxAgencyMessage {
        pub uid: String,
        pub decrypted_msg: String,
    }

    fn determine_message_type(a2a_message: A2AMessage) -> PayloadKinds {
        debug!("determine_message_type >>> a2a_message: {:?}", a2a_message);
        match a2a_message.clone() {
            A2AMessage::PresentationRequest(_) => PayloadKinds::ProofRequest,
            A2AMessage::CredentialOffer(_) => PayloadKinds::CredOffer,
            A2AMessage::Credential(_) => PayloadKinds::Cred,
            A2AMessage::Presentation(_) => PayloadKinds::Proof,
            _msg => PayloadKinds::Other(String::from("aries"))
        }
    }

    fn str_message_to_a2a_message(message: &str) -> VcxResult<A2AMessage> {
        Ok(serde_json::from_str(message)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize A2A message: {}", err)))?
        )
    }

    fn str_message_to_payload_type(message: &str) -> VcxResult<PayloadKinds> {
        let a2a_message = str_message_to_a2a_message(message)?;
        Ok(determine_message_type(a2a_message))
    }

    fn download_message(did: String, filter_msg_type: PayloadKinds) -> Option<VcxAgencyMessage> {
        let mut messages = aries_vcx::agency_client::get_message::download_messages_noauth(Some(vec![did]), Some(vec![String::from("MS-103")]), None).unwrap();
        assert_eq!(1, messages.len());
        let messages = messages.pop().unwrap();

        for message in messages.msgs.into_iter() {
            let decrypted_msg = &message.decrypted_msg.unwrap();
            let msg_type = str_message_to_payload_type(decrypted_msg).unwrap();
            if filter_msg_type == msg_type {
                return Some(VcxAgencyMessage {
                    uid: message.uid,
                    decrypted_msg: decrypted_msg.clone(),
                });
            }
        }
        None
    }

    pub trait TestAgent {
        fn activate(&mut self) -> VcxResult<()>;
    }

    pub struct Faber {
        pub is_active: bool,
        pub config_wallet: WalletConfig,
        pub config_agency: AgencyClientConfig,
        pub config_issuer: IssuerConfig,
        pub connection: Connection,
        pub schema: Schema,
        pub cred_def: CredentialDef,
        pub issuer_credential: Issuer,
        pub verifier: Verifier,
    }

    impl TestAgent for Faber {
        fn activate(&mut self) -> VcxResult<()> {
            close_main_wallet()
                .unwrap_or_else(|e| warn!("Failed to close main wallet (perhaps none was open?)"));
            settings::clear_config();

            info!("activate >>> Faber opening main wallet");
            open_as_main_wallet(&self.config_wallet)?;
            info!("activate >>> Faber initiating issuer config");
            init_issuer_config(&self.config_issuer)?;
            info!("activate >>> Faber initiating agency client");
            create_agency_client_for_main_wallet(&self.config_agency)?;
            info!("activate >>> Faber done");
            Ok(())
        }
    }

    impl TestAgent for Alice {
        fn activate(&mut self) -> VcxResult<()> {
            close_main_wallet()
                .unwrap_or_else(|e| warn!("Failed to close main wallet (perhaps none was open?)"));
            settings::clear_config();

            info!("activate >>> Alice opening main wallet");
            open_as_main_wallet(&self.config_wallet)?;
            info!("activate >>> Alice initiating agency client");
            create_agency_client_for_main_wallet(&self.config_agency)?;
            info!("activate >>> Alice done");
            Ok(())
        }
    }

    impl Faber {
        pub fn setup() -> Faber {
            settings::clear_config();
            let enterprise_seed = "000000000000000000000000Trustee1";
            let config_wallet = WalletConfig {
                wallet_name: format!("faber_wallet_{}", uuid::Uuid::new_v4().to_string()),
                wallet_key: settings::DEFAULT_WALLET_KEY.into(),
                wallet_key_derivation: settings::WALLET_KDF_RAW.into(),
                wallet_type: None,
                storage_config: None,
                storage_credentials: None,
                rekey: None,
                rekey_derivation_method: None
            };
            let config_provision_agent = AgentProvisionConfig {
                agency_did: AGENCY_DID.to_string(),
                agency_verkey: AGENCY_VERKEY.to_string(),
                agency_endpoint: AGENCY_ENDPOINT.to_string(),
                agent_seed: None
            };
            create_wallet(&config_wallet).unwrap();
            open_as_main_wallet(&config_wallet).unwrap();
            let config_issuer = configure_issuer_wallet(enterprise_seed).unwrap();
            init_issuer_config(&config_issuer).unwrap();
            let config_agency = provision_cloud_agent(&config_provision_agent).unwrap();
            let faber = Faber {
                is_active: false,
                config_wallet,
                config_agency,
                config_issuer,
                schema: Schema::default(),
                cred_def: CredentialDef::default(),
                connection: Connection::create("alice", true).unwrap(),
                issuer_credential: Issuer::default(),
                verifier: Verifier::default(),
            };
            close_main_wallet().unwrap();
            faber
        }

        pub fn create_schema(&mut self) {
            self.activate().unwrap();
            let did = String::from("V4SGRU86Z58d6TV7PBUe6f");
            let data = r#"["name","date","degree", "empty_param"]"#.to_string();
            let name: String = aries_vcx::utils::random::generate_random_schema_name();
            let version: String = String::from("1.0");

            let (schema_id, schema) = anoncreds::create_schema(&name, &version, &data).unwrap();
            let payment_txn = anoncreds::publish_schema(&schema).unwrap();

            self.schema = Schema {
                source_id: "test_schema".to_string(),
                name,
                data: serde_json::from_str(&data).unwrap_or_default(),
                version,
                schema_id,
                payment_txn,
                state: PublicEntityStateType::Published,
            };
        }

        pub fn create_credential_definition(&mut self) {
            self.activate().unwrap();

            let schema_id = self.schema.get_schema_id().to_string();
            let did = String::from("V4SGRU86Z58d6TV7PBUe6f");
            let name = String::from("degree");
            let tag = String::from("tag");

            self.cred_def = CredentialDef::create(String::from("test_cred_def"), name, did.clone(), schema_id, tag, String::from("{}")).unwrap();
        }

        pub fn create_presentation_request(&self) -> Verifier {
            let requested_attrs = json!([
                {"name": "name"},
                {"name": "date"},
                {"name": "degree"},
                {"name": "empty_param", "restrictions": {"attr::empty_param::value": ""}}
            ]).to_string();

            Verifier::create(String::from("alice_degree"),
                             requested_attrs,
                             json!([]).to_string(),
                             json!({}).to_string(),
                             String::from("proof_from_alice")).unwrap()
        }

        pub fn create_invite(&mut self) -> String {
            self.activate().unwrap();
            self.connection.connect().unwrap();
            self.connection.update_state().unwrap();
            assert_eq!(ConnectionState::Inviter(InviterState::Invited), self.connection.get_state());

            json!(self.connection.get_invite_details().unwrap()).to_string()
        }

        pub fn update_state(&mut self, expected_state: u32) {
            self.activate().unwrap();
            self.connection.update_state().unwrap();
            assert_eq!(expected_state, u32::from(self.connection.get_state()));
        }

        pub fn ping(&mut self) {
            self.activate().unwrap();
            self.connection.send_ping(None).unwrap();
        }

        pub fn discovery_features(&mut self) {
            self.activate().unwrap();
            self.connection.send_discovery_features(None, None).unwrap();
        }

        pub fn connection_info(&mut self) -> serde_json::Value {
            self.activate().unwrap();
            let details = self.connection.get_connection_info().unwrap();
            serde_json::from_str(&details).unwrap()
        }

        pub fn offer_credential(&mut self) {
            self.activate().unwrap();

            let did = String::from("V4SGRU86Z58d6TV7PBUe6f");
            let credential_data = json!({
                "name": "alice",
                "date": "05-2018",
                "degree": "maths",
                "empty_param": ""
            }).to_string();

            let issuer_config = AriesIssuerConfig {
                cred_def_id: self.cred_def.get_cred_def_id(),
                rev_reg_id: self.cred_def.get_rev_reg_id(),
                tails_file: self.cred_def.get_tails_file(),
            };
            self.issuer_credential = Issuer::create(&issuer_config, &credential_data, "alice_degree").unwrap();
            self.issuer_credential.send_credential_offer(self.connection.send_message_closure().unwrap(), None).unwrap();
            self.issuer_credential.update_state(&self.connection).unwrap();
            assert_eq!(IssuerState::OfferSent, self.issuer_credential.get_state());
        }

        pub fn send_credential(&mut self) {
            self.activate().unwrap();
            self.issuer_credential.update_state(&self.connection).unwrap();
            assert_eq!(IssuerState::RequestReceived, self.issuer_credential.get_state());

            self.issuer_credential.send_credential(self.connection.send_message_closure().unwrap()).unwrap();
            self.issuer_credential.update_state(&self.connection).unwrap();
            assert_eq!(IssuerState::Finished, self.issuer_credential.get_state());
            assert_eq!(aries_vcx::messages::status::Status::Success.code(), self.issuer_credential.get_credential_status().unwrap());
        }

        pub fn request_presentation(&mut self) {
            self.activate().unwrap();
            self.verifier = self.create_presentation_request();
            assert_eq!(VerifierState::Initial, self.verifier.get_state());

            self.verifier.send_presentation_request(self.connection.send_message_closure().unwrap(), None).unwrap();
            self.verifier.update_state(&self.connection).unwrap();

            assert_eq!(VerifierState::PresentationRequestSent, self.verifier.get_state());
        }

        pub fn verify_presentation(&mut self) {
            self.activate().unwrap();
            self.update_proof_state(VerifierState::Finished, aries_vcx::messages::status::Status::Success.code())
        }

        pub fn update_proof_state(&mut self, expected_state: VerifierState, expected_status: u32) {
            self.activate().unwrap();

            self.verifier.update_state(&self.connection).unwrap();
            assert_eq!(expected_state, self.verifier.get_state());
            assert_eq!(expected_status, self.verifier.presentation_status());
        }

        pub fn teardown(&mut self) {
            self.activate().unwrap();
            close_main_wallet().unwrap();
            delete_wallet(&self.config_wallet).unwrap();
        }
    }

    pub struct Alice {
        pub is_active: bool,
        pub config_wallet: WalletConfig,
        pub config_agency: AgencyClientConfig,
        pub connection: Connection,
        pub credential: Holder,
        pub prover: Prover
    }

    impl Alice {
        pub fn setup() -> Alice {
            settings::clear_config();

            let config_wallet = WalletConfig {
                wallet_name: format!("alice_wallet_{}", uuid::Uuid::new_v4().to_string()),
                wallet_key: settings::DEFAULT_WALLET_KEY.into(),
                wallet_key_derivation: settings::WALLET_KDF_RAW.into(),
                wallet_type: None,
                storage_config: None,
                storage_credentials: None,
                rekey: None,
                rekey_derivation_method: None
            };

            let config_provision_agent = AgentProvisionConfig {
                agency_did: C_AGENCY_DID.to_string(),
                agency_verkey: C_AGENCY_VERKEY.to_string(),
                agency_endpoint: C_AGENCY_ENDPOINT.to_string(),
                agent_seed: None
            };

            create_wallet(&config_wallet).unwrap();
            open_as_main_wallet(&config_wallet).unwrap();
            let config_agency = provision_cloud_agent(&config_provision_agent).unwrap();
            let alice = Alice {
                is_active: false,
                config_wallet,
                config_agency,
                connection: Connection::create("tmp_empoty", true).unwrap(),
                credential: Holder::default(),
                prover: Prover::default()
            };
            close_main_wallet().unwrap();
            alice
        }

        pub fn accept_invite(&mut self, invite: &str) {
            self.activate().unwrap();
            self.connection = Connection::create_with_invite("faber", serde_json::from_str(invite).unwrap(), true).unwrap();
            self.connection.connect().unwrap();
            self.connection.update_state().unwrap();
            assert_eq!(ConnectionState::Invitee(InviteeState::Requested), self.connection.get_state());
        }

        pub fn update_state(&mut self, expected_state: u32) {
            self.activate().unwrap();
            self.connection.update_state().unwrap();
            assert_eq!(expected_state, u32::from(self.connection.get_state()));
        }

        pub fn download_message(&mut self, message_type: PayloadKinds) -> VcxResult<VcxAgencyMessage> {
            self.activate()?;
            let did = self.connection.pairwise_info().pw_did.to_string();
            download_message(did, message_type)
                .ok_or(VcxError::from_msg(VcxErrorKind::UnknownError, format!("Failed to download a message")))
        }

        pub fn accept_offer(&mut self) {
            self.activate().unwrap();
            let offers = get_credential_offer_messages(&self.connection).unwrap();
            let offer = serde_json::from_str::<Vec<::serde_json::Value>>(&offers).unwrap()[0].clone();
            let offer = serde_json::to_string(&offer).unwrap();
            let cred_offer: CredentialOffer = serde_json::from_str(&offer)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                                  format!("Strict `aries` protocol is enabled. Can not parse `aries` formatted Credential Offer: {}", err))).unwrap();

            self.credential = Holder::create(cred_offer, "degree").unwrap();
            assert_eq!(HolderState::OfferReceived, self.credential.get_state());

            let pw_did = self.connection.pairwise_info().pw_did.to_string();
            self.credential.send_request(pw_did, self.connection.send_message_closure().unwrap());
            assert_eq!(HolderState::RequestSent, self.credential.get_state());
        }

        pub fn accept_credential(&mut self) {
            self.activate().unwrap();
            self.credential.update_state(&self.connection).unwrap();
            assert_eq!(HolderState::Finished, self.credential.get_state());
            assert_eq!(aries_vcx::messages::status::Status::Success.code(), self.credential.get_credential_status().unwrap());
        }

        pub fn get_proof_request_messages(&mut self) -> PresentationRequest {
            self.activate().unwrap();
            let presentation_requests = get_proof_request_messages(&self.connection).unwrap();
            let presentation_request = serde_json::from_str::<Vec<::serde_json::Value>>(&presentation_requests).unwrap()[0].clone();
            let presentation_request_json = serde_json::to_string(&presentation_request).unwrap();
            let presentation_request: PresentationRequest = serde_json::from_str(&presentation_request_json).unwrap();
            presentation_request
        }

        pub fn get_proof_request_by_msg_id(&mut self, msg_id: &str) -> VcxResult<PresentationRequest> {
            self.activate().unwrap();
            match self.connection.get_message_by_id(msg_id).unwrap() {
                A2AMessage::PresentationRequest(presentation_request) => Ok(presentation_request),
                msg => {
                    Err(VcxError::from_msg(VcxErrorKind::InvalidMessages,
                                                  format!("Message of different type was received: {:?}", msg)))
                }
            }
        }

        pub fn get_credential_offer_by_msg_id(&mut self, msg_id: &str) -> VcxResult<CredentialOffer> {
            self.activate().unwrap();
            match self.connection.get_message_by_id(msg_id).unwrap() {
                A2AMessage::CredentialOffer(cred_offer) => Ok(cred_offer),
                msg => {
                    Err(VcxError::from_msg(VcxErrorKind::InvalidMessages,
                                                  format!("Message of different type was received: {:?}", msg)))
                }
            }
        }

        pub fn get_credentials_for_presentation(&mut self) -> serde_json::Value {
            let credentials = self.prover.retrieve_credentials().unwrap();
            let credentials: std::collections::HashMap<String, serde_json::Value> = serde_json::from_str(&credentials).unwrap();

            let mut use_credentials = json!({});

            for (referent, credentials) in credentials["attrs"].as_object().unwrap().iter() {
                use_credentials["attrs"][referent] = json!({
                    "credential": credentials[0]
                })
            }

            use_credentials
        }

        pub fn send_presentation(&mut self) {
            self.activate().unwrap();
            let presentation_request = self.get_proof_request_messages();

            self.prover = Prover::create("degree", presentation_request).unwrap();

            let credentials = self.get_credentials_for_presentation();

            self.prover.generate_presentation(credentials.to_string(), String::from("{}")).unwrap();
            assert_eq!(ProverState::PresentationPrepared, self.prover.get_state());

            self.prover.send_presentation(&self.connection.send_message_closure().unwrap()).unwrap();
            assert_eq!(ProverState::PresentationSent, self.prover.get_state());
        }

        pub fn decline_presentation_request(&mut self) {
            self.activate().unwrap();

            let presentation_request = self.get_proof_request_messages();
            self.prover = Prover::create("degree", presentation_request).unwrap();

            self.prover.decline_presentation_request(&self.connection.send_message_closure().unwrap(), None, None).unwrap();
        }

        pub fn propose_presentation(&mut self) {
            self.activate().unwrap();

            let presentation_request = self.get_proof_request_messages();
            self.prover = Prover::create("degree", presentation_request).unwrap();

            let proposal_data = json!({
                "attributes": [
                    {
                        "name": "first name"
                    }
                ],
                "predicates": [
                    {
                        "name": "age",
                        "predicate": ">",
                        "threshold": 18
                    }
                ]
            }).to_string();
            self.prover.decline_presentation_request(&self.connection.send_message_closure().unwrap(), None, Some(proposal_data)).unwrap();
        }

        pub fn ensure_presentation_verified(&mut self) {
            self.activate().unwrap();
            self.prover.update_state(&self.connection).unwrap();
            assert_eq!(aries_vcx::messages::status::Status::Success.code(), self.prover.presentation_status());
        }
    }

    impl Drop for Faber {
        fn drop(&mut self) {
            self.activate().unwrap_or_else(|e| error!("Failed to close main wallet while dropping Faber"));
            close_main_wallet().unwrap_or_else(|e| error!("Failed to close main wallet while dropping Faber"));
            delete_wallet(&self.config_wallet).unwrap_or_else(|e| error!("Failed to delete Faber's wallet while dropping"));
        }
    }

    impl Drop for Alice {
        fn drop(&mut self) {
            self.activate().unwrap_or_else(|e| error!("Failed to close main wallet while dropping Alice"));
            close_main_wallet().unwrap_or_else(|e| error!("Failed to close main wallet while dropping Alice"));
            delete_wallet(&self.config_wallet).unwrap_or_else(|e| error!("Failed to delete Alice's wallet while dropping"));
        }
    }
}
