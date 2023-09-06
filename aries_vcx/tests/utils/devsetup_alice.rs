use std::sync::Arc;

use agency_client::agency_client::AgencyClient;
use agency_client::configuration::{AgencyClientConfig, AgentProvisionConfig};
use aries_vcx::common::ledger::transactions::into_did_doc;
use aries_vcx::core::profile::profile::Profile;
use aries_vcx::global::settings::DEFAULT_LINK_SECRET_ALIAS;
use aries_vcx::handlers::connection::mediated_connection::{ConnectionState, MediatedConnection};
use aries_vcx::handlers::issuance::holder::Holder;
use aries_vcx::handlers::proof_presentation::prover::Prover;
use aries_vcx::handlers::revocation_notification::receiver::RevocationNotificationReceiver;
use aries_vcx::handlers::util::AnyInvitation;
use aries_vcx::protocols::mediated_connection::invitee::state_machine::InviteeState;
use aries_vcx::utils::devsetup::{
    dev_build_featured_profile, dev_setup_wallet_indy, AGENCY_DID, AGENCY_ENDPOINT, AGENCY_VERKEY,
};
use aries_vcx::utils::provision::provision_cloud_agent;
use aries_vcx::utils::random::generate_random_seed;
use aries_vcx_core::wallet::indy::IndySdkWallet;

pub struct Alice {
    pub profile: Arc<dyn Profile>,
    pub is_active: bool,
    pub config_agency: AgencyClientConfig,
    pub connection: MediatedConnection,
    pub credential: Holder,
    pub rev_not_receiver: Option<RevocationNotificationReceiver>,
    pub prover: Prover,
    pub agency_client: AgencyClient,
    pub genesis_file_path: String,
}

pub async fn create_alice(genesis_file_path: String) -> Alice {
    let (_public_did, wallet_handle) = dev_setup_wallet_indy(&generate_random_seed()).await;
    let wallet = Arc::new(IndySdkWallet::new(wallet_handle));
    let profile = dev_build_featured_profile(genesis_file_path.clone(), wallet).await;
    profile
        .inject_anoncreds()
        .prover_create_link_secret(DEFAULT_LINK_SECRET_ALIAS)
        .await
        .unwrap();
    Alice::setup(profile, genesis_file_path).await
}

impl Alice {
    pub async fn setup(profile: Arc<dyn Profile>, genesis_file_path: String) -> Alice {
        let config_provision_agent = AgentProvisionConfig {
            agency_did: AGENCY_DID.to_string(),
            agency_verkey: AGENCY_VERKEY.to_string(),
            agency_endpoint: AGENCY_ENDPOINT.parse().unwrap(),
            agent_seed: None,
        };
        let mut agency_client = AgencyClient::new();
        let config_agency = provision_cloud_agent(&mut agency_client, profile.inject_wallet(), &config_provision_agent)
            .await
            .unwrap();
        let connection = MediatedConnection::create("tmp_empoty", &profile.inject_wallet(), &agency_client, true)
            .await
            .unwrap();
        let alice = Alice {
            genesis_file_path,
            profile,
            agency_client,
            is_active: false,
            config_agency,
            connection,
            credential: Holder::create("test").unwrap(),
            prover: Prover::default(),
            rev_not_receiver: None,
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
}
