#[cfg(test)]
#[cfg(feature = "test_utils")]
pub mod test_utils {
    use indy_sys::{WalletHandle, PoolHandle};

    use agency_client::agency_client::AgencyClient;
    use agency_client::api::downloaded_message::DownloadedMessage;
    use agency_client::configuration::{AgencyClientConfig, AgentProvisionConfig};
    use agency_client::MessageStatusCode;
    use aries_vcx::error::{VcxError, VcxErrorKind, VcxResult};

    use aries_vcx::global::settings;
    use aries_vcx::global::settings::init_issuer_config;
    use aries_vcx::handlers::connection::connection::{Connection, ConnectionState};
    use aries_vcx::handlers::connection::public_agent::PublicAgent;
    use aries_vcx::handlers::issuance::holder::test_utils::get_credential_offer_messages;
    use aries_vcx::handlers::issuance::holder::Holder;
    use aries_vcx::handlers::issuance::issuer::Issuer;
    use aries_vcx::handlers::proof_presentation::prover::test_utils::get_proof_request_messages;
    use aries_vcx::handlers::proof_presentation::prover::Prover;
    use aries_vcx::handlers::proof_presentation::verifier::Verifier;
    use aries_vcx::libindy::credential_def::PublicEntityStateType;
    use aries_vcx::libindy::credential_def::{CredentialDef, CredentialDefConfigBuilder};
    use aries_vcx::libindy::schema::Schema;
    use aries_vcx::libindy::utils::anoncreds;
    use aries_vcx::libindy::utils::wallet::wallet_configure_issuer;
    use aries_vcx::libindy::utils::wallet::*;
    use aries_vcx::libindy::wallet::open_wallet;
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
    use aries_vcx::utils::devsetup::*;
    use aries_vcx::utils::provision::provision_cloud_agent;

    #[derive(Debug)]
    pub struct VcxAgencyMessage {
        pub uid: String,
        pub decrypted_msg: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum PayloadKinds {
        CredOffer,
        CredReq,
        Cred,
        Proof,
        ProofRequest,
        ConnRequest,
        Other(String),
    }

    fn determine_message_type(a2a_message: A2AMessage) -> PayloadKinds {
        debug!("determine_message_type >>> a2a_message: {:?}", a2a_message);
        match a2a_message.clone() {
            A2AMessage::PresentationRequest(_) => PayloadKinds::ProofRequest,
            A2AMessage::CredentialOffer(_) => PayloadKinds::CredOffer,
            A2AMessage::Credential(_) => PayloadKinds::Cred,
            A2AMessage::Presentation(_) => PayloadKinds::Proof,
            A2AMessage::ConnectionRequest(_) => PayloadKinds::ConnRequest,
            _msg => PayloadKinds::Other(String::from("aries")),
        }
    }

    fn str_message_to_a2a_message(message: &str) -> VcxResult<A2AMessage> {
        Ok(serde_json::from_str(message).map_err(|err| {
            VcxError::from_msg(
                VcxErrorKind::InvalidJson,
                format!("Cannot deserialize A2A message: {}", err),
            )
        })?)
    }

    fn str_message_to_payload_type(message: &str) -> VcxResult<PayloadKinds> {
        let a2a_message = str_message_to_a2a_message(message)?;
        Ok(determine_message_type(a2a_message))
    }

    async fn filter_messages(
        messages: Vec<DownloadedMessage>,
        filter_msg_type: PayloadKinds,
    ) -> Option<VcxAgencyMessage> {
        for message in messages.into_iter() {
            let decrypted_msg = &message.decrypted_msg;
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
        pub wallet_handle: WalletHandle,
        pub pool_handle: PoolHandle,
        pub agency_client: AgencyClient,
    }

    impl Faber {
        pub async fn setup() -> Faber {
            settings::reset_config_values();
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
            create_wallet_with_master_secret(&config_wallet).await.unwrap();
            let wallet_handle = open_wallet(&config_wallet).await.unwrap();
            let config_issuer = wallet_configure_issuer(wallet_handle, enterprise_seed).await.unwrap();
            init_issuer_config(&config_issuer).unwrap();
            let mut agency_client = AgencyClient::new();
            let pool_handle = aries_vcx::global::pool::get_main_pool_handle().unwrap();
            let config_agency = provision_cloud_agent(&mut agency_client, wallet_handle, &config_provision_agent)
                .await
                .unwrap();
            let connection = Connection::create("faber", agency_client.get_wallet_handle(), &agency_client, true)
                .await
                .unwrap();
            let agent = PublicAgent::create(wallet_handle, pool_handle, &agency_client, "faber", &config_issuer.institution_did)
                .await
                .unwrap();
            let faber = Faber {
                wallet_handle,
                pool_handle,
                agency_client,
                is_active: false,
                config_wallet,
                config_agency,
                config_issuer,
                schema: Schema::default(),
                cred_def: CredentialDef::default(),
                connection,
                issuer_credential: Issuer::default(),
                verifier: Verifier::default(),
                agent,
            };
            faber
        }

        pub async fn create_schema(&mut self) {
            let data = r#"["name","date","degree", "empty_param"]"#.to_string();
            let name: String = aries_vcx::utils::random::generate_random_schema_name();
            let version: String = String::from("1.0");

            let (schema_id, schema) = anoncreds::create_schema(&self.config_issuer.institution_did, &name, &version, &data).await.unwrap();
            anoncreds::publish_schema(&self.config_issuer.institution_did, self.wallet_handle, self.pool_handle, &schema).await.unwrap();

            self.schema = Schema {
                source_id: "test_schema".to_string(),
                name,
                data: serde_json::from_str(&data).unwrap_or_default(),
                version,
                schema_id,
                state: PublicEntityStateType::Published,
            };
        }

        pub async fn create_nonrevocable_credential_definition(&mut self) {
            let config = CredentialDefConfigBuilder::default()
                .issuer_did("V4SGRU86Z58d6TV7PBUe6f")
                .schema_id(self.schema.get_schema_id())
                .tag("tag")
                .build()
                .unwrap();

            self.cred_def = CredentialDef::create(self.wallet_handle, self.pool_handle, String::from("test_cred_def"), config, false)
                .await
                .unwrap()
                .publish_cred_def(self.wallet_handle, self.pool_handle)
                .await
                .unwrap();
        }

        pub async fn create_presentation_request(&self) -> Verifier {
            let requested_attrs = json!([
                {"name": "name"},
                {"name": "date"},
                {"name": "degree"},
                {"name": "empty_param", "restrictions": {"attr::empty_param::value": ""}}
            ])
            .to_string();
            let presentation_request_data = PresentationRequestData::create("1")
                .await
                .unwrap()
                .set_requested_attributes_as_string(requested_attrs)
                .unwrap();
            Verifier::create_from_request(String::from("alice_degree"), &presentation_request_data).unwrap()
        }

        pub async fn create_invite(&mut self) -> String {
            self.connection
                .connect(self.wallet_handle, &self.agency_client)
                .await
                .unwrap();
            self.connection
                .find_message_and_update_state(self.wallet_handle, &self.agency_client)
                .await
                .unwrap();
            assert_eq!(
                ConnectionState::Inviter(InviterState::Invited),
                self.connection.get_state()
            );

            json!(self.connection.get_invite_details().unwrap()).to_string()
        }

        pub fn create_public_invite(&mut self) -> VcxResult<String> {
            let public_invitation = PublicInvitation::create()
                .set_label("faber")
                .set_public_did(&self.config_issuer.institution_did)?;
            Ok(json!(public_invitation).to_string())
        }

        pub async fn update_state(&mut self, expected_state: u32) {
            self.connection
                .find_message_and_update_state(self.wallet_handle, &self.agency_client)
                .await
                .unwrap();
            assert_eq!(expected_state, u32::from(self.connection.get_state()));
        }

        pub async fn handle_messages(&mut self) {
            self.connection
                .find_and_handle_message(self.wallet_handle, &self.agency_client)
                .await
                .unwrap();
        }

        pub async fn respond_messages(&mut self, expected_state: u32) {
            self.connection
                .find_and_handle_message(self.wallet_handle, &self.agency_client)
                .await
                .unwrap();
            assert_eq!(expected_state, u32::from(self.connection.get_state()));
        }

        pub async fn ping(&mut self) {
            self.connection.send_ping(self.wallet_handle, None).await.unwrap();
        }

        pub async fn discovery_features(&mut self) {
            self.connection
                .send_discovery_query(self.wallet_handle, None, None)
                .await
                .unwrap();
        }

        pub async fn connection_info(&mut self) -> serde_json::Value {
            let details = self.connection.get_connection_info(&self.agency_client).unwrap();
            serde_json::from_str(&details).unwrap()
        }

        pub async fn offer_non_revocable_credential(&mut self) {
            let credential_json = json!({
                "name": "alice",
                "date": "05-2018",
                "degree": "maths",
                "empty_param": ""
            })
            .to_string();

            let offer_info = OfferInfo {
                credential_json,
                cred_def_id: self.cred_def.get_cred_def_id(),
                rev_reg_id: None,
                tails_file: None,
            };
            self.issuer_credential = Issuer::create("alice_degree").unwrap();
            self.issuer_credential
                .build_credential_offer_msg(self.wallet_handle, offer_info, None)
                .await
                .unwrap();
            self.issuer_credential
                .send_credential_offer(self.connection.send_message_closure(self.wallet_handle).unwrap())
                .await
                .unwrap();
            self.issuer_credential
                .update_state(self.wallet_handle, &self.agency_client, &self.connection)
                .await
                .unwrap();
            assert_eq!(IssuerState::OfferSent, self.issuer_credential.get_state());
        }

        pub async fn send_credential(&mut self) {
            self.issuer_credential
                .update_state(self.wallet_handle, &self.agency_client, &self.connection)
                .await
                .unwrap();
            assert_eq!(IssuerState::RequestReceived, self.issuer_credential.get_state());

            self.issuer_credential
                .send_credential(
                    self.wallet_handle,
                    self.connection.send_message_closure(self.wallet_handle).unwrap(),
                )
                .await
                .unwrap();
            self.issuer_credential
                .update_state(self.wallet_handle, &self.agency_client, &self.connection)
                .await
                .unwrap();
            assert_eq!(IssuerState::CredentialSent, self.issuer_credential.get_state());
        }

        pub async fn request_presentation(&mut self) {
            self.verifier = self.create_presentation_request().await;
            assert_eq!(VerifierState::PresentationRequestSet, self.verifier.get_state());

            self.verifier
                .send_presentation_request(self.connection.send_message_closure(self.wallet_handle).unwrap())
                .await
                .unwrap();
            self.verifier
                .update_state(self.wallet_handle, &self.agency_client, &self.connection)
                .await
                .unwrap();

            assert_eq!(VerifierState::PresentationRequestSent, self.verifier.get_state());
        }

        pub async fn verify_presentation(&mut self) {
            self.update_proof_state(
                VerifierState::Finished,
                aries_vcx::messages::status::Status::Success.code(),
            )
            .await
        }

        pub async fn update_proof_state(&mut self, expected_state: VerifierState, expected_status: u32) {
            self.verifier
                .update_state(self.wallet_handle, &self.agency_client, &self.connection)
                .await
                .unwrap();
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
        pub wallet_handle: WalletHandle,
        pub pool_handle: PoolHandle,
        pub agency_client: AgencyClient,
    }

    impl Alice {
        pub async fn setup() -> Alice {
            settings::reset_config_values();
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
                agency_did: AGENCY_DID.to_string(),
                agency_verkey: AGENCY_VERKEY.to_string(),
                agency_endpoint: AGENCY_ENDPOINT.to_string(),
                agent_seed: None,
            };
            create_wallet_with_master_secret(&config_wallet).await.unwrap();
            let wallet_handle = open_wallet(&config_wallet).await.unwrap();
            let pool_handle = aries_vcx::global::pool::get_main_pool_handle().unwrap();
            let mut agency_client = AgencyClient::new();
            let config_agency = provision_cloud_agent(&mut agency_client, wallet_handle, &config_provision_agent)
                .await
                .unwrap();
            let connection = Connection::create("tmp_empoty", agency_client.get_wallet_handle(), &agency_client, true)
                .await
                .unwrap();
            let alice = Alice {
                wallet_handle,
                pool_handle,
                agency_client,
                is_active: false,
                config_wallet,
                config_agency,
                connection,
                credential: Holder::default(),
                prover: Prover::default(),
            };
            alice
        }

        pub async fn accept_invite(&mut self, invite: &str) {
            self.connection = Connection::create_with_invite(
                "faber",
                self.wallet_handle,
                &self.agency_client,
                serde_json::from_str(invite).unwrap(),
                true,
            )
            .await
            .unwrap();
            self.connection
                .connect(self.wallet_handle, &self.agency_client)
                .await
                .unwrap();
            self.connection
                .find_message_and_update_state(self.wallet_handle, &self.agency_client)
                .await
                .unwrap();
            assert_eq!(
                ConnectionState::Invitee(InviteeState::Requested),
                self.connection.get_state()
            );
        }

        pub async fn update_state(&mut self, expected_state: u32) {
            self.connection
                .find_message_and_update_state(self.wallet_handle, &self.agency_client)
                .await
                .unwrap();
            assert_eq!(expected_state, u32::from(self.connection.get_state()));
        }

        pub async fn handle_messages(&mut self) {
            self.connection
                .find_and_handle_message(self.wallet_handle, &self.agency_client)
                .await
                .unwrap();
        }

        pub async fn respond_messages(&mut self, expected_state: u32) {
            self.connection
                .find_and_handle_message(self.wallet_handle, &self.agency_client)
                .await
                .unwrap();
            assert_eq!(expected_state, u32::from(self.connection.get_state()));
        }

        pub async fn download_message(&mut self, message_type: PayloadKinds) -> VcxResult<VcxAgencyMessage> {
            let _did = self.connection.pairwise_info().pw_did.to_string();
            let messages = self
                .connection
                .download_messages(&self.agency_client, Some(vec![MessageStatusCode::Received]), None)
                .await
                .unwrap();
            filter_messages(messages, message_type).await.ok_or(VcxError::from_msg(
                VcxErrorKind::UnknownError,
                format!("Failed to download a message"),
            ))
        }

        pub async fn accept_offer(&mut self) {
            let offers = get_credential_offer_messages(&self.agency_client, &self.connection)
                .await
                .unwrap();
            let offer = serde_json::from_str::<Vec<::serde_json::Value>>(&offers).unwrap()[0].clone();
            let offer = serde_json::to_string(&offer).unwrap();
            let cred_offer: CredentialOffer = serde_json::from_str(&offer)
                .map_err(|err| {
                    VcxError::from_msg(
                        VcxErrorKind::InvalidJson,
                        format!(
                            "Strict `aries` protocol is enabled. Can not parse `aries` formatted Credential Offer: {}",
                            err
                        ),
                    )
                })
                .unwrap();

            self.credential = Holder::create_from_offer("degree", cred_offer).unwrap();
            assert_eq!(HolderState::OfferReceived, self.credential.get_state());

            let pw_did = self.connection.pairwise_info().pw_did.to_string();
            self.credential
                .send_request(
                    self.wallet_handle,
                    self.pool_handle,
                    pw_did,
                    self.connection.send_message_closure(self.wallet_handle).unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(HolderState::RequestSent, self.credential.get_state());
        }

        pub async fn accept_credential(&mut self) {
            self.credential
                .update_state(self.wallet_handle, self.pool_handle, &self.agency_client, &self.connection)
                .await
                .unwrap();
            assert_eq!(HolderState::Finished, self.credential.get_state());
            assert_eq!(
                aries_vcx::messages::status::Status::Success.code(),
                self.credential.get_credential_status().unwrap()
            );
        }

        pub async fn get_proof_request_messages(&mut self) -> PresentationRequest {
            let presentation_requests = get_proof_request_messages(&self.agency_client, &self.connection)
                .await
                .unwrap();
            let presentation_request =
                serde_json::from_str::<Vec<::serde_json::Value>>(&presentation_requests).unwrap()[0].clone();
            let presentation_request_json = serde_json::to_string(&presentation_request).unwrap();
            let presentation_request: PresentationRequest = serde_json::from_str(&presentation_request_json).unwrap();
            presentation_request
        }

        pub async fn get_proof_request_by_msg_id(&mut self, msg_id: &str) -> VcxResult<PresentationRequest> {
            match self
                .connection
                .get_message_by_id(msg_id, &self.agency_client)
                .await
                .unwrap()
            {
                A2AMessage::PresentationRequest(presentation_request) => Ok(presentation_request),
                msg => Err(VcxError::from_msg(
                    VcxErrorKind::InvalidMessages,
                    format!("Message of different type was received: {:?}", msg),
                )),
            }
        }

        pub async fn get_credential_offer_by_msg_id(&mut self, msg_id: &str) -> VcxResult<CredentialOffer> {
            match self
                .connection
                .get_message_by_id(msg_id, &self.agency_client)
                .await
                .unwrap()
            {
                A2AMessage::CredentialOffer(cred_offer) => Ok(cred_offer),
                msg => Err(VcxError::from_msg(
                    VcxErrorKind::InvalidMessages,
                    format!("Message of different type was received: {:?}", msg),
                )),
            }
        }

        pub async fn get_credentials_for_presentation(&mut self) -> serde_json::Value {
            let credentials = self.prover.retrieve_credentials(self.wallet_handle).await.unwrap();
            let credentials: std::collections::HashMap<String, serde_json::Value> =
                serde_json::from_str(&credentials).unwrap();

            let mut use_credentials = json!({});

            for (referent, credentials) in credentials["attrs"].as_object().unwrap().iter() {
                use_credentials["attrs"][referent] = json!({
                    "credential": credentials[0]
                })
            }

            use_credentials
        }

        pub async fn send_presentation(&mut self) {
            let presentation_request = self.get_proof_request_messages().await;

            self.prover = Prover::create_from_request("degree", presentation_request).unwrap();

            let credentials = self.get_credentials_for_presentation().await;

            self.prover
                .generate_presentation(self.wallet_handle, credentials.to_string(), String::from("{}"))
                .await
                .unwrap();
            assert_eq!(ProverState::PresentationPrepared, self.prover.get_state());

            self.prover
                .send_presentation(
                    self.wallet_handle,
                    self.connection.send_message_closure(self.wallet_handle).unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(ProverState::PresentationSent, self.prover.get_state());
        }

        pub async fn ensure_presentation_verified(&mut self) {
            self.prover
                .update_state(self.wallet_handle, &self.agency_client, &self.connection)
                .await
                .unwrap();
            assert_eq!(
                aries_vcx::messages::status::Status::Success.code(),
                self.prover.presentation_status()
            );
        }
    }

    impl Drop for Faber {
        fn drop(&mut self) {
            futures::executor::block_on(close_wallet(self.wallet_handle))
                .unwrap_or_else(|_| error!("Failed to close Faber's wallet while dropping Faber"));
            futures::executor::block_on(delete_wallet(&self.config_wallet))
                .unwrap_or_else(|_| error!("Failed to delete Faber's wallet while dropping"));
        }
    }

    impl Drop for Alice {
        fn drop(&mut self) {
            futures::executor::block_on(close_wallet(self.wallet_handle))
                .unwrap_or_else(|_| error!("Failed to close Alice's wallet while dropping Alice"));
            futures::executor::block_on(delete_wallet(&self.config_wallet))
                .unwrap_or_else(|_| error!("Failed to delete Alice's wallet while dropping"));
        }
    }
}
