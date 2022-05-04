#[cfg(test)]
pub mod test {
    use aries_vcx::agency_client::payload::PayloadKinds;
    use aries_vcx::error::{VcxError, VcxErrorKind, VcxResult};
    use aries_vcx::handlers::connection::connection::{Connection, ConnectionState};
    use aries_vcx::handlers::connection::public_agent::PublicAgent;
    use aries_vcx::handlers::issuance::holder::Holder;
    use aries_vcx::handlers::issuance::holder::test_utils::get_credential_offer_messages;
    use aries_vcx::handlers::issuance::issuer::Issuer;
    use aries_vcx::handlers::proof_presentation::prover::Prover;
    use aries_vcx::handlers::proof_presentation::prover::test_utils::get_proof_request_messages;
    use aries_vcx::handlers::proof_presentation::verifier::Verifier;
    use aries_vcx::init::{create_agency_client_for_main_wallet, init_issuer_config, open_as_main_wallet};
    use aries_vcx::libindy::credential_def::{CredentialDef, CredentialDefConfigBuilder, RevocationDetails};
    use aries_vcx::libindy::credential_def::PublicEntityStateType;
    use aries_vcx::libindy::schema::Schema;
    use aries_vcx::libindy::utils::anoncreds;
    use aries_vcx::libindy::utils::wallet::*;
    use aries_vcx::messages::a2a::A2AMessage;
    use aries_vcx::messages::connection::invite::PublicInvitation;
    use aries_vcx::messages::issuance::credential_offer::CredentialOffer;
    use aries_vcx::messages::issuance::credential_offer::OfferInfo;
    use aries_vcx::messages::proof_presentation::presentation_request::{PresentationRequest, PresentationRequestData};
    use aries_vcx::protocols::connection::invitee::state_machine::InviteeState;
    use aries_vcx::protocols::connection::inviter::state_machine::InviterState;
    use aries_vcx::protocols::issuance::holder::state_machine::HolderState;
    use aries_vcx::protocols::issuance::issuer::state_machine::IssuerState;
    use aries_vcx::protocols::proof_presentation::prover::state_machine::ProverState;
    use aries_vcx::protocols::proof_presentation::verifier::state_machine::VerifierState;
    use aries_vcx::settings;
    use aries_vcx::utils::devsetup::*;
    use aries_vcx::utils::provision::{AgencyClientConfig, AgentProvisionConfig, provision_cloud_agent};

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
            A2AMessage::ConnectionRequest(_) => PayloadKinds::ConnRequest,
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

    async fn download_message(did: String, filter_msg_type: PayloadKinds) -> Option<VcxAgencyMessage> {
        let mut messages = aries_vcx::agency_client::get_message::download_messages_noauth(Some(vec![did]), Some(vec![String::from("MS-103")]), None).await.unwrap();
        assert_eq!(1, messages.len());
        let messages = messages.pop().unwrap();

        for message in messages.msgs.into_iter() {
            let decrypted_msg = &message.decrypted_msg.unwrap();
            let msg_type = str_message_to_payload_type(decrypted_msg).unwrap();
            if filter_msg_type == msg_type {
                return Some(VcxAgencyMessage {
                    uid: message.uid,
                    decrypted_msg: decrypted_msg.to_string(),
                });
            }
        }
        None
    }

    #[async_trait::async_trait]
    pub trait TestAgent {
        async fn activate(&mut self) -> VcxResult<()>;
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
        pub agent: PublicAgent,
    }


    #[async_trait::async_trait]
    impl TestAgent for Faber {
        async fn activate(&mut self) -> VcxResult<()> {
            close_main_wallet()
                .await
                .unwrap_or_else(|_| warn!("Failed to close main wallet (perhaps none was open?)"));
            settings::clear_config();

            info!("activate >>> Faber opening main wallet");
            open_as_main_wallet(&self.config_wallet).await?;
            info!("activate >>> Faber initiating issuer config");
            init_issuer_config(&self.config_issuer)?;
            info!("activate >>> Faber initiating agency client");
            create_agency_client_for_main_wallet(&self.config_agency)?;
            info!("activate >>> Faber done");
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl TestAgent for Alice {
        async fn activate(&mut self) -> VcxResult<()> {
            close_main_wallet()
                .await
                .unwrap_or_else(|_| warn!("Failed to close main wallet (perhaps none was open?)"));
            settings::clear_config();

            info!("activate >>> Alice opening main wallet");
            open_as_main_wallet(&self.config_wallet).await?;
            info!("activate >>> Alice initiating agency client");
            create_agency_client_for_main_wallet(&self.config_agency)?;
            info!("activate >>> Alice done");
            Ok(())
        }
    }

    impl Faber {
        pub async fn setup() -> Faber {
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
                rekey_derivation_method: None,
            };
            let config_provision_agent = AgentProvisionConfig {
                agency_did: AGENCY_DID.to_string(),
                agency_verkey: AGENCY_VERKEY.to_string(),
                agency_endpoint: AGENCY_ENDPOINT.to_string(),
                agent_seed: None,
            };
            create_wallet(&config_wallet).await.unwrap();
            open_as_main_wallet(&config_wallet).await.unwrap();
            let config_issuer = configure_issuer_wallet(enterprise_seed).await.unwrap();
            init_issuer_config(&config_issuer).unwrap();
            let config_agency = provision_cloud_agent(&config_provision_agent).await.unwrap();
            let institution_did = config_issuer.clone().institution_did;
            let faber = Faber {
                is_active: false,
                config_wallet,
                config_agency,
                config_issuer,
                schema: Schema::default(),
                cred_def: CredentialDef::default(),
                connection: Connection::create("faber", true).await.unwrap(),
                issuer_credential: Issuer::default(),
                verifier: Verifier::default(),
                agent: PublicAgent::create("faber", &institution_did).await.unwrap(),
            };
            close_main_wallet().await.unwrap();
            faber
        }

        pub async fn create_schema(&mut self) {
            self.activate().await.unwrap();
            let data = r#"["name","date","degree", "empty_param"]"#.to_string();
            let name: String = aries_vcx::utils::random::generate_random_schema_name();
            let version: String = String::from("1.0");

            let (schema_id, schema) = anoncreds::create_schema(&name, &version, &data).await.unwrap();
            anoncreds::publish_schema(&schema).await.unwrap();

            self.schema = Schema {
                source_id: "test_schema".to_string(),
                name,
                data: serde_json::from_str(&data).unwrap_or_default(),
                version,
                schema_id,
                state: PublicEntityStateType::Published,
            };
        }

        pub async fn create_credential_definition(&mut self) {
            self.activate().await.unwrap();

            let config = CredentialDefConfigBuilder::default()
                .issuer_did("V4SGRU86Z58d6TV7PBUe6f")
                .schema_id(self.schema.get_schema_id())
                .tag("tag")
                .build()
                .unwrap();

            self.cred_def = CredentialDef::create_and_store(String::from("test_cred_def"), config, RevocationDetails::default()).await.unwrap()
                .publish_cred_def().await.unwrap();
        }

        pub async fn create_presentation_request(&self) -> Verifier {
            let requested_attrs = json!([
                {"name": "name"},
                {"name": "date"},
                {"name": "degree"},
                {"name": "empty_param", "restrictions": {"attr::empty_param::value": ""}}
            ]).to_string();
            let presentation_request_data =
                PresentationRequestData::create("1").await.unwrap()
                    .set_requested_attributes_as_string(requested_attrs).unwrap();
            Verifier::create_from_request(String::from("alice_degree"), &presentation_request_data).unwrap()
        }

        pub async fn create_invite(&mut self) -> String {
            self.activate().await.unwrap();
            self.connection.connect().await.unwrap();
            self.connection.update_state().await.unwrap();
            assert_eq!(ConnectionState::Inviter(InviterState::Invited), self.connection.get_state());

            json!(self.connection.get_invite_details().unwrap()).to_string()
        }

        pub fn create_public_invite(&mut self) -> VcxResult<String> {
            let public_invitation = PublicInvitation::create()
                .set_label("faber")
                .set_public_did(&self.config_issuer.institution_did)?;
            Ok(json!(public_invitation).to_string())
        }

        pub async fn update_state(&mut self, expected_state: u32) {
            self.activate().await.unwrap();
            self.connection.update_state().await.unwrap();
            assert_eq!(expected_state, u32::from(self.connection.get_state()));
        }

        pub async fn ping(&mut self) {
            self.activate().await.unwrap();
            self.connection.send_ping(None).await.unwrap();
        }

        pub async fn discovery_features(&mut self) {
            self.activate().await.unwrap();
            self.connection.send_discovery_features(None, None).await.unwrap();
        }

        pub async fn connection_info(&mut self) -> serde_json::Value {
            self.activate().await.unwrap();
            let details = self.connection.get_connection_info().unwrap();
            serde_json::from_str(&details).unwrap()
        }

        pub async fn offer_credential(&mut self) {
            self.activate().await.unwrap();

            let credential_json = json!({
                "name": "alice",
                "date": "05-2018",
                "degree": "maths",
                "empty_param": ""
            }).to_string();

            let offer_info = OfferInfo {
                credential_json,
                cred_def_id: self.cred_def.get_cred_def_id(),
                rev_reg_id: self.cred_def.get_rev_reg_id(),
                tails_file: self.cred_def.get_tails_dir(),
            };
            self.issuer_credential = Issuer::create("alice_degree").unwrap();
            self.issuer_credential.build_credential_offer_msg(offer_info, None).await.unwrap();
            self.issuer_credential.send_credential_offer(self.connection.send_message_closure().unwrap()).await.unwrap();
            self.issuer_credential.update_state(&self.connection).await.unwrap();
            assert_eq!(IssuerState::OfferSent, self.issuer_credential.get_state());
        }

        pub async fn send_credential(&mut self) {
            self.activate().await.unwrap();
            self.issuer_credential.update_state(&self.connection).await.unwrap();
            assert_eq!(IssuerState::RequestReceived, self.issuer_credential.get_state());

            self.issuer_credential.send_credential(self.connection.send_message_closure().unwrap()).await.unwrap();
            self.issuer_credential.update_state(&self.connection).await.unwrap();
            assert_eq!(IssuerState::CredentialSent, self.issuer_credential.get_state());
        }

        pub async fn request_presentation(&mut self) {
            self.activate().await.unwrap();
            self.verifier = self.create_presentation_request().await;
            assert_eq!(VerifierState::PresentationRequestSet, self.verifier.get_state());

            self.verifier.send_presentation_request(self.connection.send_message_closure().unwrap()).await.unwrap();
            self.verifier.update_state(&self.connection).await.unwrap();

            assert_eq!(VerifierState::PresentationRequestSent, self.verifier.get_state());
        }

        pub async fn verify_presentation(&mut self) {
            self.activate().await.unwrap();
            self.update_proof_state(VerifierState::Finished, aries_vcx::messages::status::Status::Success.code()).await
        }

        pub async fn update_proof_state(&mut self, expected_state: VerifierState, expected_status: u32) {
            self.activate().await.unwrap();

            self.verifier.update_state(&self.connection).await.unwrap();
            assert_eq!(expected_state, self.verifier.get_state());
            assert_eq!(expected_status, self.verifier.get_presentation_status());
        }
    }

    pub struct Alice {
        pub is_active: bool,
        pub config_wallet: WalletConfig,
        pub config_agency: AgencyClientConfig,
        pub connection: Connection,
        pub credential: Holder,
        pub prover: Prover,
    }

    impl Alice {
        pub async fn setup() -> Alice {
            settings::clear_config();

            let config_wallet = WalletConfig {
                wallet_name: format!("alice_wallet_{}", uuid::Uuid::new_v4().to_string()),
                wallet_key: settings::DEFAULT_WALLET_KEY.into(),
                wallet_key_derivation: settings::WALLET_KDF_RAW.into(),
                wallet_type: None,
                storage_config: None,
                storage_credentials: None,
                rekey: None,
                rekey_derivation_method: None,
            };

            let config_provision_agent = AgentProvisionConfig {
                agency_did: C_AGENCY_DID.to_string(),
                agency_verkey: C_AGENCY_VERKEY.to_string(),
                agency_endpoint: C_AGENCY_ENDPOINT.to_string(),
                agent_seed: None,
            };

            create_wallet(&config_wallet).await.unwrap();
            open_as_main_wallet(&config_wallet).await.unwrap();
            let config_agency = provision_cloud_agent(&config_provision_agent).await.unwrap();
            let alice = Alice {
                is_active: false,
                config_wallet,
                config_agency,
                connection: Connection::create("tmp_empoty", true).await.unwrap(),
                credential: Holder::default(),
                prover: Prover::default(),
            };
            close_main_wallet().await.unwrap();
            alice
        }

        pub async fn accept_invite(&mut self, invite: &str) {
            self.activate().await.unwrap();
            self.connection = Connection::create_with_invite("faber", serde_json::from_str(invite).unwrap(), true).await.unwrap();
            self.connection.connect().await.unwrap();
            self.connection.update_state().await.unwrap();
            assert_eq!(ConnectionState::Invitee(InviteeState::Requested), self.connection.get_state());
        }

        pub async fn update_state(&mut self, expected_state: u32) {
            self.activate().await.unwrap();
            self.connection.update_state().await.unwrap();
            assert_eq!(expected_state, u32::from(self.connection.get_state()));
        }

        pub async fn download_message(&mut self, message_type: PayloadKinds) -> VcxResult<VcxAgencyMessage> {
            self.activate().await?;
            let did = self.connection.pairwise_info().pw_did.to_string();
            download_message(did, message_type)
                .await
                .ok_or(VcxError::from_msg(VcxErrorKind::UnknownError, format!("Failed to download a message")))
        }

        pub async fn accept_offer(&mut self) {
            self.activate().await.unwrap();
            let offers = get_credential_offer_messages(&self.connection).await.unwrap();
            let offer = serde_json::from_str::<Vec<::serde_json::Value>>(&offers).unwrap()[0].clone();
            let offer = serde_json::to_string(&offer).unwrap();
            let cred_offer: CredentialOffer = serde_json::from_str(&offer)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                                  format!("Strict `aries` protocol is enabled. Can not parse `aries` formatted Credential Offer: {}", err))).unwrap();

            self.credential = Holder::create_from_offer("degree", cred_offer).unwrap();
            assert_eq!(HolderState::OfferReceived, self.credential.get_state());

            let pw_did = self.connection.pairwise_info().pw_did.to_string();
            self.credential.send_request(pw_did, self.connection.send_message_closure().unwrap()).await.unwrap();
            assert_eq!(HolderState::RequestSent, self.credential.get_state());
        }

        pub async fn accept_credential(&mut self) {
            self.activate().await.unwrap();
            self.credential.update_state(&self.connection).await.unwrap();
            assert_eq!(HolderState::Finished, self.credential.get_state());
            assert_eq!(aries_vcx::messages::status::Status::Success.code(), self.credential.get_credential_status().unwrap());
        }

        pub async fn get_proof_request_messages(&mut self) -> PresentationRequest {
            self.activate().await.unwrap();
            let presentation_requests = get_proof_request_messages(&self.connection).await.unwrap();
            let presentation_request = serde_json::from_str::<Vec<::serde_json::Value>>(&presentation_requests).unwrap()[0].clone();
            let presentation_request_json = serde_json::to_string(&presentation_request).unwrap();
            let presentation_request: PresentationRequest = serde_json::from_str(&presentation_request_json).unwrap();
            presentation_request
        }

        pub async fn get_proof_request_by_msg_id(&mut self, msg_id: &str) -> VcxResult<PresentationRequest> {
            self.activate().await.unwrap();
            match self.connection.get_message_by_id(msg_id).await.unwrap() {
                A2AMessage::PresentationRequest(presentation_request) => Ok(presentation_request),
                msg => {
                    Err(VcxError::from_msg(VcxErrorKind::InvalidMessages,
                                           format!("Message of different type was received: {:?}", msg)))
                }
            }
        }

        pub async fn get_credential_offer_by_msg_id(&mut self, msg_id: &str) -> VcxResult<CredentialOffer> {
            self.activate().await.unwrap();
            match self.connection.get_message_by_id(msg_id).await.unwrap() {
                A2AMessage::CredentialOffer(cred_offer) => Ok(cred_offer),
                msg => {
                    Err(VcxError::from_msg(VcxErrorKind::InvalidMessages,
                                           format!("Message of different type was received: {:?}", msg)))
                }
            }
        }

        pub async fn get_credentials_for_presentation(&mut self) -> serde_json::Value {
            let credentials = self.prover.retrieve_credentials().await.unwrap();
            let credentials: std::collections::HashMap<String, serde_json::Value> = serde_json::from_str(&credentials).unwrap();

            let mut use_credentials = json!({});

            for (referent, credentials) in credentials["attrs"].as_object().unwrap().iter() {
                use_credentials["attrs"][referent] = json!({
                    "credential": credentials[0]
                })
            }

            use_credentials
        }

        pub async fn send_presentation(&mut self) {
            self.activate().await.unwrap();
            let presentation_request = self.get_proof_request_messages().await;

            self.prover = Prover::create_from_request("degree", presentation_request).unwrap();

            let credentials = self.get_credentials_for_presentation().await;

            self.prover.generate_presentation(credentials.to_string(), String::from("{}")).await.unwrap();
            assert_eq!(ProverState::PresentationPrepared, self.prover.get_state());

            self.prover.send_presentation(self.connection.send_message_closure().unwrap()).await.unwrap();
            assert_eq!(ProverState::PresentationSent, self.prover.get_state());
        }

        pub async fn ensure_presentation_verified(&mut self) {
            self.activate().await.unwrap();
            self.prover.update_state(&self.connection).await.unwrap();
            assert_eq!(aries_vcx::messages::status::Status::Success.code(), self.prover.presentation_status());
        }
    }

    impl Drop for Faber {
        fn drop(&mut self) {
            futures::executor::block_on(self.activate()).unwrap_or_else(|_| error!("Failed to close main wallet while dropping Faber"));
            futures::executor::block_on(close_main_wallet()).unwrap_or_else(|_| error!("Failed to close main wallet while dropping Faber"));
            futures::executor::block_on(delete_wallet(&self.config_wallet)).unwrap_or_else(|_| error!("Failed to delete Faber's wallet while dropping"));
        }
    }

    impl Drop for Alice {
        fn drop(&mut self) {
            futures::executor::block_on(self.activate()).unwrap_or_else(|_| error!("Failed to close main wallet while dropping Alice"));
            futures::executor::block_on(close_main_wallet()).unwrap_or_else(|_| error!("Failed to close main wallet while dropping Alice"));
            futures::executor::block_on(delete_wallet(&self.config_wallet)).unwrap_or_else(|_| error!("Failed to delete Alice's wallet while dropping"));
        }
    }
}
