use std::sync::Arc;

use serde_json::json;

use agency_client::agency_client::AgencyClient;
use agency_client::configuration::{AgencyClientConfig, AgentProvisionConfig};
use aries_vcx::common::ledger::transactions::write_endpoint_legacy;
use aries_vcx::common::primitives::credential_schema::Schema;
use aries_vcx::core::profile::profile::Profile;
use aries_vcx::errors::error::VcxResult;
use aries_vcx::global::settings;
use aries_vcx::global::settings::{init_issuer_config, DEFAULT_LINK_SECRET_ALIAS};
use aries_vcx::handlers::connection::mediated_connection::{ConnectionState, MediatedConnection};
use aries_vcx::protocols::connection::pairwise_info::PairwiseInfo;
use aries_vcx::protocols::mediated_connection::inviter::state_machine::InviterState;
use aries_vcx::utils::constants::TRUSTEE_SEED;
use aries_vcx::utils::devsetup::{
    dev_build_featured_profile, dev_setup_wallet_indy, AGENCY_DID, AGENCY_ENDPOINT, AGENCY_VERKEY,
};
use aries_vcx::utils::provision::provision_cloud_agent;
use aries_vcx::utils::random::generate_random_seed;
use aries_vcx_core::wallet::indy::wallet::get_verkey_from_wallet;
use aries_vcx_core::wallet::indy::IndySdkWallet;
use diddoc_legacy::aries::service::AriesService;
use messages::msg_fields::protocols::connection::invitation::public::{PublicInvitation, PublicInvitationContent};
use messages::AriesMessage;

pub struct Faber {
    pub profile: Arc<dyn Profile>,
    pub config_agency: AgencyClientConfig,
    pub institution_did: String,
    pub connection: MediatedConnection,
    pub schema: Schema,
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

        let faber = Faber {
            genesis_file_path,
            profile,
            agency_client,
            config_agency,
            institution_did,
            schema: Schema::default(),
            connection,
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
}
