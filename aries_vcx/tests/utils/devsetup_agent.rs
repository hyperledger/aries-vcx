#[cfg(test)]
pub mod test_utils {
    use std::collections::HashMap;
    use std::sync::Arc;

    #[cfg(feature = "modular_libs")]
    use aries_vcx::core::profile::modular_libs_profile::ModularLibsProfile;
    use aries_vcx::core::profile::profile::Profile;
    use aries_vcx::core::profile::vdrtools_profile::VdrtoolsProfile;
    use aries_vcx::handlers::proof_presentation::types::SelectedCredentials;
    use aries_vcx::handlers::revocation_notification::receiver::RevocationNotificationReceiver;
    use aries_vcx::handlers::revocation_notification::sender::RevocationNotificationSender;
    use aries_vcx::handlers::util::{AnyInvitation, OfferInfo, Status};
    use aries_vcx::protocols::mediated_connection::pairwise_info::PairwiseInfo;
    use aries_vcx::protocols::revocation_notification::sender::state_machine::SenderConfigBuilder;
    use aries_vcx_core::indy::wallet::{
        close_wallet, create_wallet_with_master_secret, delete_wallet, open_wallet, wallet_configure_issuer,
        IssuerConfig, WalletConfig,
    };
    #[cfg(feature = "modular_libs")]
    use aries_vcx_core::ledger::request_submitter::vdr_ledger::LedgerPoolConfig;
    use aries_vcx_core::wallet::base_wallet::BaseWallet;
    use aries_vcx_core::wallet::indy_wallet::IndySdkWallet;
    use aries_vcx_core::{PoolHandle, WalletHandle};
    use diddoc_legacy::aries::service::AriesService;
    use futures::future::BoxFuture;

    use agency_client::agency_client::AgencyClient;
    use agency_client::api::downloaded_message::DownloadedMessage;
    use agency_client::configuration::{AgencyClientConfig, AgentProvisionConfig};
    use agency_client::MessageStatusCode;
    use aries_vcx::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

    use aries_vcx::common::ledger::transactions::{into_did_doc, write_endpoint_legacy};
    use aries_vcx::common::primitives::credential_definition::CredentialDef;
    use aries_vcx::common::primitives::credential_definition::CredentialDefConfigBuilder;
    use aries_vcx::common::primitives::credential_schema::Schema;
    use aries_vcx::common::proofs::proof_request::PresentationRequestData;
    use aries_vcx::global::settings;
    use aries_vcx::global::settings::init_issuer_config;
    use aries_vcx::handlers::connection::mediated_connection::{ConnectionState, MediatedConnection};
    use aries_vcx::handlers::issuance::holder::test_utils::get_credential_offer_messages;
    use aries_vcx::handlers::issuance::holder::Holder;
    use aries_vcx::handlers::issuance::issuer::Issuer;
    use aries_vcx::handlers::proof_presentation::prover::test_utils::get_proof_request_messages;
    use aries_vcx::handlers::proof_presentation::prover::Prover;
    use aries_vcx::handlers::proof_presentation::verifier::Verifier;
    use aries_vcx::protocols::issuance::holder::state_machine::HolderState;
    use aries_vcx::protocols::issuance::issuer::state_machine::IssuerState;
    use aries_vcx::protocols::mediated_connection::invitee::state_machine::InviteeState;
    use aries_vcx::protocols::mediated_connection::inviter::state_machine::InviterState;
    use aries_vcx::protocols::proof_presentation::prover::state_machine::ProverState;
    use aries_vcx::protocols::proof_presentation::verifier::state_machine::VerifierState;
    use aries_vcx::protocols::proof_presentation::verifier::verification_status::PresentationVerificationStatus;
    use aries_vcx::utils::devsetup::*;
    use aries_vcx::utils::provision::provision_cloud_agent;
    use messages::decorators::please_ack::AckOn;
    use messages::msg_fields::protocols::connection::invitation::{PublicInvitation, PublicInvitationContent};
    use messages::msg_fields::protocols::connection::Connection;
    use messages::msg_fields::protocols::cred_issuance::offer_credential::OfferCredential;
    use messages::msg_fields::protocols::cred_issuance::CredentialIssuance;
    use messages::msg_fields::protocols::present_proof::request::RequestPresentation;
    use messages::msg_fields::protocols::present_proof::PresentProof;
    use messages::msg_fields::protocols::revocation::ack::AckRevoke;
    use messages::msg_fields::protocols::revocation::revoke::Revoke;
    use messages::AriesMessage;

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

    fn determine_message_type(a2a_message: AriesMessage) -> PayloadKinds {
        debug!("determine_message_type >>> a2a_message: {:?}", a2a_message);
        match a2a_message.clone() {
            AriesMessage::PresentProof(PresentProof::RequestPresentation(_)) => PayloadKinds::ProofRequest,
            AriesMessage::CredentialIssuance(CredentialIssuance::OfferCredential(_)) => PayloadKinds::CredOffer,
            AriesMessage::CredentialIssuance(CredentialIssuance::IssueCredential(_)) => PayloadKinds::Cred,
            AriesMessage::PresentProof(PresentProof::Presentation(_)) => PayloadKinds::Proof,
            AriesMessage::Connection(Connection::Request(_)) => PayloadKinds::ConnRequest,
            _msg => PayloadKinds::Other(String::from("aries")),
        }
    }

    fn str_message_to_a2a_message(message: &str) -> VcxResult<AriesMessage> {
        Ok(serde_json::from_str(message).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
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
        pub profile: Arc<dyn Profile>,
        pub is_active: bool,
        pub config_agency: AgencyClientConfig,
        pub config_issuer: IssuerConfig,
        pub rev_not_sender: RevocationNotificationSender,
        pub connection: MediatedConnection,
        pub schema: Schema,
        pub cred_def: CredentialDef,
        pub issuer_credential: Issuer,
        pub verifier: Verifier,
        pub pairwise_info: PairwiseInfo,
        pub agency_client: AgencyClient,
        pub(self) teardown: Arc<dyn Fn() -> BoxFuture<'static, ()> + Send + Sync>,
    }

    impl Faber {
        pub async fn setup(pool_handle: PoolHandle) -> Faber {
            settings::reset_config_values().unwrap();
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
                agency_endpoint: AGENCY_ENDPOINT.parse().unwrap(),
                agent_seed: None,
            };
            create_wallet_with_master_secret(&config_wallet).await.unwrap();
            let wallet_handle = open_wallet(&config_wallet).await.unwrap();

            let indy_profile = VdrtoolsProfile::init(wallet_handle, pool_handle);
            let profile: Arc<dyn Profile> = Arc::new(indy_profile);

            let config_issuer = wallet_configure_issuer(wallet_handle, enterprise_seed).await.unwrap();
            init_issuer_config(&config_issuer.institution_did).unwrap();
            let mut agency_client = AgencyClient::new();
            let config_agency =
                provision_cloud_agent(&mut agency_client, profile.inject_wallet(), &config_provision_agent)
                    .await
                    .unwrap();
            let connection = MediatedConnection::create("faber", &profile.inject_wallet(), &agency_client, true)
                .await
                .unwrap();

            let pairwise_info = PairwiseInfo::create(&profile.inject_wallet()).await.unwrap();
            let service = AriesService::create()
                .set_service_endpoint(agency_client.get_agency_url_full().unwrap())
                .set_recipient_keys(vec![pairwise_info.pw_vk.clone()]);
            write_endpoint_legacy(
                &profile.inject_indy_ledger_write(),
                &config_issuer.institution_did,
                &service,
            )
            .await
            .unwrap();

            let rev_not_sender = RevocationNotificationSender::build();

            let faber = Faber {
                profile,
                agency_client,
                is_active: false,
                config_agency,
                config_issuer,
                schema: Schema::default(),
                cred_def: CredentialDef::default(),
                connection,
                issuer_credential: Issuer::default(),
                verifier: Verifier::default(),
                rev_not_sender,
                pairwise_info,
                teardown: Arc::new(move || Box::pin(teardown_indy_wallet(wallet_handle, config_wallet.clone()))),
            };
            faber
        }

        pub async fn create_schema(&mut self) {
            let data = vec!["name", "date", "degree", "empty_param"]
                .iter()
                .map(|s| s.to_string())
                .collect();
            let name: String = aries_vcx::utils::random::generate_random_schema_name();
            let version: String = String::from("1.0");

            self.schema = Schema::create(
                &self.profile.inject_anoncreds(),
                "",
                &self.config_issuer.institution_did,
                &name,
                &version,
                &data,
            )
            .await
            .unwrap()
            .publish(&self.profile.inject_anoncreds_ledger_write(), None)
            .await
            .unwrap();
        }

        pub async fn create_nonrevocable_credential_definition(&mut self) {
            let config = CredentialDefConfigBuilder::default()
                .issuer_did("V4SGRU86Z58d6TV7PBUe6f")
                .schema_id(self.schema.get_schema_id())
                .tag("tag")
                .build()
                .unwrap();

            self.cred_def = CredentialDef::create(
                &self.profile.inject_anoncreds_ledger_read(),
                &self.profile.inject_anoncreds(),
                String::from("test_cred_def"),
                config,
                false,
            )
            .await
            .unwrap()
            .publish_cred_def(
                &self.profile.inject_anoncreds_ledger_read(),
                &self.profile.inject_anoncreds_ledger_write(),
            )
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
            let presentation_request_data = PresentationRequestData::create(&self.profile.inject_anoncreds(), "1")
                .await
                .unwrap()
                .set_requested_attributes_as_string(requested_attrs)
                .unwrap();
            Verifier::create_from_request(String::from("alice_degree"), &presentation_request_data).unwrap()
        }

        pub async fn create_invite(&mut self) -> String {
            self.connection
                .connect(&self.profile.inject_wallet(), &self.agency_client, None)
                .await
                .unwrap();
            self.connection
                .find_message_and_update_state(&self.profile.inject_wallet(), &self.agency_client)
                .await
                .unwrap();
            assert_eq!(
                ConnectionState::Inviter(InviterState::Invited),
                self.connection.get_state()
            );

            json!(self.connection.get_invite_details().unwrap()).to_string()
        }

        pub fn create_public_invite(&mut self) -> VcxResult<String> {
            let id = "test_invite_id";
            let content = PublicInvitationContent::new("faber".to_owned(), self.config_issuer.institution_did.clone());
            let public_invitation = PublicInvitation::new(id.to_owned(), content);
            Ok(json!(AriesMessage::from(public_invitation)).to_string())
        }

        pub async fn update_state(&mut self, expected_state: u32) {
            self.connection
                .find_message_and_update_state(&self.profile.inject_wallet(), &self.agency_client)
                .await
                .unwrap();
            assert_eq!(expected_state, u32::from(self.connection.get_state()));
        }

        pub async fn handle_messages(&mut self) {
            self.connection
                .find_and_handle_message(&self.profile.inject_wallet(), &self.agency_client)
                .await
                .unwrap();
        }

        pub async fn respond_messages(&mut self, expected_state: u32) {
            self.connection
                .find_and_handle_message(&self.profile.inject_wallet(), &self.agency_client)
                .await
                .unwrap();
            assert_eq!(expected_state, u32::from(self.connection.get_state()));
        }

        pub async fn ping(&mut self) {
            self.connection
                .send_ping(self.profile.inject_wallet(), None)
                .await
                .unwrap();
        }

        pub async fn discovery_features(&mut self) {
            self.connection
                .send_discovery_query(&self.profile.inject_wallet(), None, None)
                .await
                .unwrap();
        }

        pub async fn connection_info(&mut self) -> serde_json::Value {
            let details = self.connection.get_connection_info(&self.agency_client).await.unwrap();
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
                .build_credential_offer_msg(&self.profile.inject_anoncreds(), offer_info, None)
                .await
                .unwrap();
            self.issuer_credential
                .send_credential_offer(
                    self.connection
                        .send_message_closure(self.profile.inject_wallet())
                        .await
                        .unwrap(),
                )
                .await
                .unwrap();
            self.issuer_credential
                .update_state(
                    &self.profile.inject_wallet(),
                    &self.profile.inject_anoncreds(),
                    &self.agency_client,
                    &self.connection,
                )
                .await
                .unwrap();
            assert_eq!(IssuerState::OfferSent, self.issuer_credential.get_state());
        }

        pub async fn send_credential(&mut self) {
            self.issuer_credential
                .update_state(
                    &self.profile.inject_wallet(),
                    &self.profile.inject_anoncreds(),
                    &self.agency_client,
                    &self.connection,
                )
                .await
                .unwrap();
            assert_eq!(IssuerState::RequestReceived, self.issuer_credential.get_state());

            self.issuer_credential
                .send_credential(
                    &self.profile.inject_anoncreds(),
                    self.connection
                        .send_message_closure(self.profile.inject_wallet())
                        .await
                        .unwrap(),
                )
                .await
                .unwrap();
            self.issuer_credential
                .update_state(
                    &self.profile.inject_wallet(),
                    &self.profile.inject_anoncreds(),
                    &self.agency_client,
                    &self.connection,
                )
                .await
                .unwrap();
            assert_eq!(IssuerState::CredentialSent, self.issuer_credential.get_state());
        }

        pub async fn request_presentation(&mut self) {
            self.verifier = self.create_presentation_request().await;
            assert_eq!(VerifierState::PresentationRequestSet, self.verifier.get_state());

            self.verifier
                .send_presentation_request(
                    self.connection
                        .send_message_closure(self.profile.inject_wallet())
                        .await
                        .unwrap(),
                )
                .await
                .unwrap();
            self.verifier
                .update_state(
                    &self.profile.inject_wallet(),
                    &self.profile.inject_anoncreds_ledger_read(),
                    &self.profile.inject_anoncreds(),
                    &self.agency_client,
                    &self.connection,
                )
                .await
                .unwrap();

            assert_eq!(VerifierState::PresentationRequestSent, self.verifier.get_state());
        }

        pub async fn verify_presentation(&mut self) {
            self.update_proof_state(VerifierState::Finished, PresentationVerificationStatus::Valid)
                .await
        }

        pub async fn update_proof_state(
            &mut self,
            expected_state: VerifierState,
            expected_verification_status: PresentationVerificationStatus,
        ) {
            self.verifier
                .update_state(
                    &self.profile.inject_wallet(),
                    &self.profile.inject_anoncreds_ledger_read(),
                    &self.profile.inject_anoncreds(),
                    &self.agency_client,
                    &self.connection,
                )
                .await
                .unwrap();
            assert_eq!(expected_state, self.verifier.get_state());
            assert_eq!(expected_verification_status, self.verifier.get_verification_status());
        }

        pub async fn send_revocation_notification(&mut self, ack_on: Vec<AckOn>) {
            let config = SenderConfigBuilder::default()
                .ack_on(ack_on)
                .rev_reg_id(self.issuer_credential.get_rev_reg_id().unwrap())
                .cred_rev_id(self.issuer_credential.get_rev_id().unwrap())
                .build()
                .unwrap();
            let send_message = self
                .connection
                .send_message_closure(self.profile.inject_wallet())
                .await
                .unwrap();
            self.rev_not_sender = self
                .rev_not_sender
                .clone()
                .send_revocation_notification(config, send_message)
                .await
                .unwrap();
        }

        pub async fn handle_revocation_notification_ack(&mut self, ack: AckRevoke) {
            self.rev_not_sender = self
                .rev_not_sender
                .clone()
                .handle_revocation_notification_ack(ack)
                .await
                .unwrap();
        }
    }

    pub struct Alice {
        pub profile: Arc<dyn Profile>,
        pub is_active: bool,
        pub config_agency: AgencyClientConfig,
        pub connection: MediatedConnection,
        pub credential: Holder,
        pub rev_not_receiver: Option<RevocationNotificationReceiver>,
        pub prover: Prover,
        pub agency_client: AgencyClient,
        pub(self) teardown: Arc<dyn Fn() -> BoxFuture<'static, ()> + Send + Sync>,
    }

    pub async fn create_test_alice_instance(setup: &SetupPool) -> Alice {
        #[cfg(any(not(feature = "modular_libs"), feature = "migration"))]
        let (alice_profile, teardown) = {
            info!("create_test_alice_instance >> using indy profile");
            Alice::setup_indy_profile(setup.pool_handle).await
        };

        #[cfg(all(feature = "modular_libs", not(feature = "migration")))]
        let (alice_profile, teardown) = {
            let genesis_file_path = setup.genesis_file_path.clone();
            let config = LedgerPoolConfig { genesis_file_path };
            info!("create_test_alice_instance >> using modular profile");
            Alice::setup_modular_profile(config).await
        };

        Alice::setup(alice_profile, teardown).await
    }

    impl Alice {
        async fn setup_indy_wallet() -> (WalletHandle, WalletConfig) {
            settings::reset_config_values().unwrap();
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
            create_wallet_with_master_secret(&config_wallet).await.unwrap();
            let wallet_handle = open_wallet(&config_wallet).await.unwrap();

            (wallet_handle, config_wallet)
        }

        #[cfg(feature = "modular_libs")]
        pub async fn setup_modular_profile(
            ledger_pool_config: LedgerPoolConfig,
        ) -> (Arc<dyn Profile>, Arc<dyn Fn() -> BoxFuture<'static, ()> + Send + Sync>) {
            let (wallet_handle, config_wallet) = Alice::setup_indy_wallet().await;

            let wallet: Arc<dyn BaseWallet> = Arc::new(IndySdkWallet::new(wallet_handle));

            let profile = Arc::new(ModularLibsProfile::init(wallet, ledger_pool_config).unwrap());

            // set up anoncreds link/master secret
            Arc::clone(&profile)
                .inject_anoncreds()
                .prover_create_link_secret(settings::DEFAULT_LINK_SECRET_ALIAS)
                .await
                .unwrap();

            (
                profile,
                Arc::new(move || Box::pin(teardown_indy_wallet(wallet_handle, config_wallet.clone()))),
            )
        }

        pub async fn setup_indy_profile(
            pool_handle: PoolHandle,
        ) -> (Arc<dyn Profile>, Arc<dyn Fn() -> BoxFuture<'static, ()> + Send + Sync>) {
            let (wallet_handle, config_wallet) = Alice::setup_indy_wallet().await;

            let indy_profile = VdrtoolsProfile::init(wallet_handle, pool_handle);

            (
                Arc::new(indy_profile),
                Arc::new(move || Box::pin(teardown_indy_wallet(wallet_handle, config_wallet.clone()))),
            )
        }

        pub async fn setup(
            profile: Arc<dyn Profile>,
            teardown: Arc<dyn Fn() -> BoxFuture<'static, ()> + Send + Sync>,
        ) -> Alice {
            let config_provision_agent = AgentProvisionConfig {
                agency_did: AGENCY_DID.to_string(),
                agency_verkey: AGENCY_VERKEY.to_string(),
                agency_endpoint: AGENCY_ENDPOINT.parse().unwrap(),
                agent_seed: None,
            };
            let mut agency_client = AgencyClient::new();
            let config_agency =
                provision_cloud_agent(&mut agency_client, profile.inject_wallet(), &config_provision_agent)
                    .await
                    .unwrap();
            let connection = MediatedConnection::create("tmp_empoty", &profile.inject_wallet(), &agency_client, true)
                .await
                .unwrap();
            let alice = Alice {
                profile,
                agency_client,
                is_active: false,
                config_agency,
                connection,
                credential: Holder::create("test").unwrap(),
                prover: Prover::default(),
                rev_not_receiver: None,
                teardown,
            };
            alice
        }

        pub async fn accept_invite(&mut self, invite: &str) {
            let invite: AnyInvitation = serde_json::from_str(invite).unwrap();
            let ddo = into_did_doc(&self.profile.inject_indy_ledger_read(), &invite)
                .await
                .unwrap();
            self.connection = MediatedConnection::create_with_invite(
                "faber",
                &self.profile.inject_wallet(),
                &self.agency_client,
                invite,
                ddo,
                true,
            )
            .await
            .unwrap();
            self.connection
                .connect(&self.profile.inject_wallet(), &self.agency_client, None)
                .await
                .unwrap();
            self.connection
                .find_message_and_update_state(&self.profile.inject_wallet(), &self.agency_client)
                .await
                .unwrap();
            assert_eq!(
                ConnectionState::Invitee(InviteeState::Requested),
                self.connection.get_state()
            );
        }

        pub async fn update_state(&mut self, expected_state: u32) {
            self.connection
                .find_message_and_update_state(&self.profile.inject_wallet(), &self.agency_client)
                .await
                .unwrap();
            assert_eq!(expected_state, u32::from(self.connection.get_state()));
        }

        pub async fn handle_messages(&mut self) {
            self.connection
                .find_and_handle_message(&self.profile.inject_wallet(), &self.agency_client)
                .await
                .unwrap();
        }

        pub async fn respond_messages(&mut self, expected_state: u32) {
            self.connection
                .find_and_handle_message(&self.profile.inject_wallet(), &self.agency_client)
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
            filter_messages(messages, message_type)
                .await
                .ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::UnknownError,
                    format!("Failed to download a message"),
                ))
        }

        pub async fn accept_offer(&mut self) {
            let offers = get_credential_offer_messages(&self.agency_client, &self.connection)
                .await
                .unwrap();
            let offer = serde_json::from_str::<Vec<::serde_json::Value>>(&offers).unwrap()[0].clone();
            let offer = serde_json::to_string(&offer).unwrap();
            let cred_offer: OfferCredential = serde_json::from_str(&offer)
                .map_err(|err| {
                    AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidJson,
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
                    &self.profile.inject_anoncreds_ledger_read(),
                    &self.profile.inject_anoncreds(),
                    pw_did,
                    self.connection
                        .send_message_closure(self.profile.inject_wallet())
                        .await
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(HolderState::RequestSent, self.credential.get_state());
        }

        pub async fn accept_credential(&mut self) {
            self.credential
                .update_state(
                    &self.profile.inject_anoncreds_ledger_read(),
                    &self.profile.inject_anoncreds(),
                    &self.profile.inject_wallet(),
                    &self.agency_client,
                    &self.connection,
                )
                .await
                .unwrap();
            assert_eq!(HolderState::Finished, self.credential.get_state());
            assert_eq!(Status::Success.code(), self.credential.get_credential_status().unwrap());
        }

        pub async fn get_proof_request_messages(&mut self) -> RequestPresentation {
            let presentation_requests = get_proof_request_messages(&self.agency_client, &self.connection)
                .await
                .unwrap();
            let presentation_request =
                serde_json::from_str::<Vec<::serde_json::Value>>(&presentation_requests).unwrap()[0].clone();
            let presentation_request_json = serde_json::to_string(&presentation_request).unwrap();
            let presentation_request: RequestPresentation = serde_json::from_str(&presentation_request_json).unwrap();
            presentation_request
        }

        pub async fn get_proof_request_by_msg_id(&mut self, msg_id: &str) -> VcxResult<RequestPresentation> {
            match self
                .connection
                .get_message_by_id(msg_id, &self.agency_client)
                .await
                .unwrap()
            {
                AriesMessage::PresentProof(PresentProof::RequestPresentation(presentation_request)) => {
                    Ok(presentation_request)
                }
                msg => Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidMessages,
                    format!("Message of different type was received: {:?}", msg),
                )),
            }
        }

        pub async fn get_credential_offer_by_msg_id(&mut self, msg_id: &str) -> VcxResult<OfferCredential> {
            match self
                .connection
                .get_message_by_id(msg_id, &self.agency_client)
                .await
                .unwrap()
            {
                AriesMessage::CredentialIssuance(CredentialIssuance::OfferCredential(cred_offer)) => Ok(cred_offer),
                msg => Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidMessages,
                    format!("Message of different type was received: {:?}", msg),
                )),
            }
        }

        pub async fn get_credentials_for_presentation(&mut self) -> SelectedCredentials {
            let credentials = self
                .prover
                .retrieve_credentials(&self.profile.inject_anoncreds())
                .await
                .unwrap();

            let mut use_credentials = SelectedCredentials {
                credential_for_referent: HashMap::new(),
            };

            for (referent, credentials) in credentials.credentials_by_referent {
                use_credentials.select_credential_for_referent_from_retrieved(referent, credentials[0].clone(), None);
            }

            use_credentials
        }

        pub async fn send_presentation(&mut self) {
            let presentation_request = self.get_proof_request_messages().await;

            self.prover = Prover::create_from_request("degree", presentation_request).unwrap();

            let credentials = self.get_credentials_for_presentation().await;

            self.prover
                .generate_presentation(
                    &self.profile.inject_anoncreds_ledger_read(),
                    &self.profile.inject_anoncreds(),
                    credentials,
                    HashMap::new(),
                )
                .await
                .unwrap();
            assert_eq!(ProverState::PresentationPrepared, self.prover.get_state());

            self.prover
                .send_presentation(
                    self.connection
                        .send_message_closure(self.profile.inject_wallet())
                        .await
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(ProverState::PresentationSent, self.prover.get_state());
        }

        pub async fn ensure_presentation_verified(&mut self) {
            self.prover
                .update_state(
                    &self.profile.inject_anoncreds_ledger_read(),
                    &self.profile.inject_anoncreds(),
                    &self.profile.inject_wallet(),
                    &self.agency_client,
                    &self.connection,
                )
                .await
                .unwrap();
            assert_eq!(Status::Success.code(), self.prover.presentation_status());
        }

        pub async fn receive_revocation_notification(&mut self, rev_not: Revoke) {
            let rev_reg_id = self.credential.get_rev_reg_id().unwrap();
            let cred_rev_id = self
                .credential
                .get_cred_rev_id(&self.profile.inject_anoncreds())
                .await
                .unwrap();
            let send_message = self
                .connection
                .send_message_closure(self.profile.inject_wallet())
                .await
                .unwrap();
            let rev_not_receiver = RevocationNotificationReceiver::build(rev_reg_id, cred_rev_id)
                .handle_revocation_notification(rev_not, send_message)
                .await
                .unwrap();
            self.rev_not_receiver = Some(rev_not_receiver);
        }
    }

    async fn teardown_indy_wallet(wallet_handle: WalletHandle, config_wallet: WalletConfig) {
        info!("Closing test indy wallet: {:?}", config_wallet);
        close_wallet(wallet_handle)
            .await
            .unwrap_or_else(|_| error!("Failed to close wallet while teardowning: {:?}", config_wallet));
        info!("Deleting test indy wallet: {:?}", config_wallet);
        delete_wallet(&config_wallet)
            .await
            .unwrap_or_else(|_| error!("Failed to delete wallet while teardowning: {:?}", config_wallet));
    }

    impl Drop for Faber {
        fn drop(&mut self) {
            futures::executor::block_on((self.teardown)());
        }
    }

    impl Drop for Alice {
        fn drop(&mut self) {
            futures::executor::block_on((self.teardown)());
        }
    }
}
