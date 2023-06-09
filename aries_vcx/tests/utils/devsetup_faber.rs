use agency_client::agency_client::AgencyClient;
use agency_client::configuration::{AgencyClientConfig, AgentProvisionConfig};
use aries_vcx::common::ledger::transactions::write_endpoint_legacy;
use aries_vcx::common::primitives::credential_definition::{CredentialDef, CredentialDefConfigBuilder};
use aries_vcx::common::primitives::credential_schema::Schema;
use aries_vcx::common::proofs::proof_request::PresentationRequestData;
use aries_vcx::core::profile::profile::Profile;
use aries_vcx::core::profile::vdrtools_profile::VdrtoolsProfile;
use aries_vcx::errors::error::VcxResult;
use aries_vcx::global::settings;
use aries_vcx::global::settings::init_issuer_config;
use aries_vcx::handlers::connection::mediated_connection::{ConnectionState, MediatedConnection};
use aries_vcx::handlers::issuance::issuer::Issuer;
use aries_vcx::handlers::proof_presentation::verifier::Verifier;
use aries_vcx::handlers::revocation_notification::sender::RevocationNotificationSender;
use aries_vcx::handlers::util::OfferInfo;
use aries_vcx::protocols::connection::pairwise_info::PairwiseInfo;
use aries_vcx::protocols::issuance::issuer::state_machine::IssuerState;
use aries_vcx::protocols::mediated_connection::inviter::state_machine::InviterState;
use aries_vcx::protocols::proof_presentation::verifier::state_machine::VerifierState;
use aries_vcx::protocols::proof_presentation::verifier::verification_status::PresentationVerificationStatus;
use aries_vcx::protocols::revocation_notification::sender::state_machine::SenderConfigBuilder;
use aries_vcx::utils::devsetup::{SetupPoolDirectory, SetupProfile, AGENCY_DID, AGENCY_ENDPOINT, AGENCY_VERKEY};
use aries_vcx::utils::provision::provision_cloud_agent;
use aries_vcx_core::indy::wallet::{
    create_wallet_with_master_secret, open_wallet, wallet_configure_issuer, IssuerConfig, WalletConfig,
};
#[cfg(feature = "modular_libs")]
use aries_vcx_core::ledger::request_submitter::vdr_ledger::LedgerPoolConfig;
use aries_vcx_core::{PoolHandle, WalletHandle};
use diddoc_legacy::aries::service::AriesService;
use futures::future::BoxFuture;
use messages::decorators::please_ack::AckOn;
use messages::msg_fields::protocols::connection::invitation::public::{PublicInvitation, PublicInvitationContent};
use messages::msg_fields::protocols::revocation::ack::AckRevoke;
use messages::AriesMessage;
use serde_json::json;
use std::sync::Arc;

pub struct Faber {
    pub profile: Arc<dyn Profile>,
    pub is_active: bool,
    pub config_agency: AgencyClientConfig,
    pub institution_did: String,
    pub rev_not_sender: RevocationNotificationSender,
    pub connection: MediatedConnection,
    pub schema: Schema,
    pub cred_def: CredentialDef,
    pub issuer_credential: Issuer,
    pub verifier: Verifier,
    pub pairwise_info: PairwiseInfo,
    pub agency_client: AgencyClient,
    pub teardown: Arc<dyn Fn() -> BoxFuture<'static, ()>>,
}

pub async fn create_faber(genesis_file_path: String) -> Faber {
    let profile_setup = SetupProfile::build_profile(genesis_file_path).await;
    let SetupProfile {
        institution_did,
        profile,
        teardown,
    } = profile_setup;
    Faber::setup(profile, institution_did, teardown).await
}

impl Faber {
    pub async fn setup(
        profile: Arc<dyn Profile>,
        institution_did: String,
        teardown: Arc<dyn Fn() -> BoxFuture<'static, ()>>,
    ) -> Faber {
        settings::reset_config_values_ariesvcx().unwrap();

        let config_provision_agent = AgentProvisionConfig {
            agency_did: AGENCY_DID.to_string(),
            agency_verkey: AGENCY_VERKEY.to_string(),
            agency_endpoint: AGENCY_ENDPOINT.parse().unwrap(),
            agent_seed: None,
        };

        // todo: can delete following?
        init_issuer_config(&institution_did).unwrap();
        let mut agency_client = AgencyClient::new();
        let config_agency = provision_cloud_agent(&mut agency_client, profile.inject_wallet(), &config_provision_agent)
            .await
            .unwrap();
        let connection = MediatedConnection::create("faber", &profile.inject_wallet(), &agency_client, true)
            .await
            .unwrap();

        let pairwise_info = PairwiseInfo::create(&profile.inject_wallet()).await.unwrap();
        let service = AriesService::create()
            .set_service_endpoint(agency_client.get_agency_url_full().unwrap())
            .set_recipient_keys(vec![pairwise_info.pw_vk.clone()]);
        write_endpoint_legacy(&profile.inject_indy_ledger_write(), &institution_did, &service)
            .await
            .unwrap();

        let rev_not_sender = RevocationNotificationSender::build();

        let faber = Faber {
            profile,
            agency_client,
            is_active: false,
            config_agency,
            institution_did,
            schema: Schema::default(),
            cred_def: CredentialDef::default(),
            connection,
            issuer_credential: Issuer::default(),
            verifier: Verifier::default(),
            rev_not_sender,
            pairwise_info,
            teardown,
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
            &self.institution_did,
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
        let content = PublicInvitationContent::new("faber".to_owned(), self.institution_did.clone());
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

impl Drop for Faber {
    fn drop(&mut self) {
        futures::executor::block_on((self.teardown)());
    }
}
