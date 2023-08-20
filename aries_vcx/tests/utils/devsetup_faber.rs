use std::sync::Arc;

use futures::future::BoxFuture;
use serde_json::json;

use crate::utils::devsetup_util::issuer_update_with_mediator;
use crate::utils::devsetup_util::verifier_update_with_mediator;
use agency_client::agency_client::AgencyClient;
use agency_client::configuration::{AgencyClientConfig, AgentProvisionConfig};
use aries_vcx::common::ledger::transactions::write_endpoint_legacy;
use aries_vcx::common::primitives::credential_definition::{CredentialDef, CredentialDefConfigBuilder};
use aries_vcx::common::primitives::credential_schema::Schema;
use aries_vcx::common::proofs::proof_request::PresentationRequestData;
use aries_vcx::core::profile::profile::Profile;
use aries_vcx::errors::error::VcxResult;
use aries_vcx::global::settings;
use aries_vcx::global::settings::{init_issuer_config, DEFAULT_LINK_SECRET_ALIAS};
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
use aries_vcx::utils::constants::TRUSTEE_SEED;
use aries_vcx::utils::devsetup::{
    dev_build_featured_profile, dev_setup_wallet_indy, AGENCY_DID, AGENCY_ENDPOINT, AGENCY_VERKEY,
};
use aries_vcx::utils::provision::provision_cloud_agent;
use aries_vcx::utils::random::generate_random_seed;
use aries_vcx_core::wallet::indy::wallet::get_verkey_from_wallet;
use aries_vcx_core::wallet::indy::IndySdkWallet;
use diddoc_legacy::aries::service::AriesService;
use messages::decorators::please_ack::AckOn;
use messages::msg_fields::protocols::connection::invitation::public::{PublicInvitation, PublicInvitationContent};
use messages::msg_fields::protocols::revocation::ack::AckRevoke;
use messages::AriesMessage;

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
    // todo: get rid of this, if we need vkey somewhere, we can get it from wallet, we can instead store public_did
    pub pairwise_info: PairwiseInfo,
    pub agency_client: AgencyClient,
    pub genesis_file_path: String,
}

pub async fn create_faber_trustee(genesis_file_path: String) -> Faber {
    let (public_did, wallet_handle) = dev_setup_wallet_indy(TRUSTEE_SEED).await;
    let wallet = Arc::new(IndySdkWallet::new(wallet_handle));
    let profile = dev_build_featured_profile(genesis_file_path.clone(), wallet).await;
    profile
        .inject_anoncreds()
        .prover_create_link_secret(DEFAULT_LINK_SECRET_ALIAS)
        .await
        .unwrap();
    let faber = Faber::setup(profile, genesis_file_path, public_did).await;

    let service = AriesService::create()
        .set_service_endpoint(faber.agency_client.get_agency_url_full().unwrap())
        .set_recipient_keys(vec![faber.pairwise_info.pw_vk.clone()]);
    write_endpoint_legacy(&faber.profile.inject_indy_ledger_write(), &faber.public_did(), &service)
        .await
        .unwrap();
    faber
}

pub async fn create_faber(genesis_file_path: String) -> Faber {
    let (public_did, wallet_handle) = dev_setup_wallet_indy(&generate_random_seed()).await;
    let wallet = Arc::new(IndySdkWallet::new(wallet_handle));
    let profile = dev_build_featured_profile(genesis_file_path.clone(), wallet).await;
    profile
        .inject_anoncreds()
        .prover_create_link_secret(DEFAULT_LINK_SECRET_ALIAS)
        .await
        .unwrap();
    Faber::setup(profile, genesis_file_path, public_did).await
}

impl Faber {
    pub async fn setup(profile: Arc<dyn Profile>, genesis_file_path: String, institution_did: String) -> Faber {
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

        let rev_not_sender = RevocationNotificationSender::build();

        let faber = Faber {
            genesis_file_path,
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
        };
        faber
    }

    pub fn public_did(&self) -> &str {
        &self.institution_did
    }

    pub async fn get_verkey_from_wallet(&self, did: &str) -> String {
        get_verkey_from_wallet(self.profile.inject_wallet().get_wallet_handle(), did)
            .await
            .unwrap()
    }

    pub async fn create_schema(&mut self) -> VcxResult<()> {
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
        .await?
        .publish(&self.profile.inject_anoncreds_ledger_write(), None)
        .await?;
        Ok(())
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
        issuer_update_with_mediator(&mut self.issuer_credential, &self.agency_client, &self.connection)
            .await
            .unwrap();
        assert_eq!(IssuerState::OfferSet, self.issuer_credential.get_state());
    }

    pub async fn send_credential(&mut self) {
        issuer_update_with_mediator(&mut self.issuer_credential, &self.agency_client, &self.connection)
            .await
            .unwrap();
        assert_eq!(IssuerState::RequestReceived, self.issuer_credential.get_state());

        self.issuer_credential
            .build_credential(&self.profile.inject_anoncreds())
            .await
            .unwrap();
        let send_closure = self
            .connection
            .send_message_closure(self.profile.inject_wallet())
            .await
            .unwrap();
        self.issuer_credential.send_credential(send_closure).await;
        issuer_update_with_mediator(&mut self.issuer_credential, &self.agency_client, &self.connection)
            .await
            .unwrap();
        assert_eq!(IssuerState::CredentialSet, self.issuer_credential.get_state());
    }

    pub async fn request_presentation(&mut self) {
        self.verifier = self.create_presentation_request().await;
        assert_eq!(VerifierState::PresentationRequestSet, self.verifier.get_state());

        let send_closure = self
            .connection
            .send_message_closure(self.profile.inject_wallet())
            .await
            .unwrap();
        let message = self.verifier.mark_presentation_request_sent().unwrap();
        send_closure(message).await.unwrap();
        verifier_update_with_mediator(
            &mut self.verifier,
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
        verifier_update_with_mediator(
            &mut self.verifier,
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
