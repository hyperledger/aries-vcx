use std::collections::HashMap;
use std::sync::Arc;

use crate::utils::devsetup_util::{
    get_credential_offer_messages, get_proof_request_messages, holder_update_with_mediator, prover_update_with_mediator,
};
use agency_client::agency_client::AgencyClient;
use agency_client::configuration::{AgencyClientConfig, AgentProvisionConfig};
use agency_client::MessageStatusCode;
use aries_vcx::common::ledger::transactions::into_did_doc;
use aries_vcx::core::profile::profile::Profile;
use aries_vcx::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};
use aries_vcx::global::settings::DEFAULT_LINK_SECRET_ALIAS;
use aries_vcx::handlers::connection::mediated_connection::{ConnectionState, MediatedConnection};
use aries_vcx::handlers::issuance::holder::Holder;
use aries_vcx::handlers::proof_presentation::prover::Prover;
use aries_vcx::handlers::proof_presentation::types::SelectedCredentials;
use aries_vcx::handlers::revocation_notification::receiver::RevocationNotificationReceiver;
use aries_vcx::handlers::util::{AnyInvitation, Status};
use aries_vcx::protocols::issuance::holder::state_machine::HolderState;
use aries_vcx::protocols::mediated_connection::invitee::state_machine::InviteeState;
use aries_vcx::protocols::proof_presentation::prover::state_machine::ProverState;
use aries_vcx::utils::devsetup::{
    dev_build_featured_profile, dev_setup_wallet_indy, AGENCY_DID, AGENCY_ENDPOINT, AGENCY_VERKEY,
};
use aries_vcx::utils::provision::provision_cloud_agent;
use aries_vcx::utils::random::generate_random_seed;
use aries_vcx_core::wallet::indy::IndySdkWallet;
use messages::msg_fields::protocols::cred_issuance::offer_credential::OfferCredential;
use messages::msg_fields::protocols::cred_issuance::CredentialIssuance;
use messages::msg_fields::protocols::present_proof::request::RequestPresentation;
use messages::msg_fields::protocols::present_proof::PresentProof;
use messages::msg_fields::protocols::revocation::revoke::Revoke;
use messages::AriesMessage;

use crate::utils::devsetup_util::test_utils::{filter_messages, PayloadKinds, VcxAgencyMessage};

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
        holder_update_with_mediator(
            &mut self.credential,
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

        let send_closure = self
            .connection
            .send_message_closure(self.profile.inject_wallet())
            .await
            .unwrap();
        let message = self.prover.set_presentation().await.unwrap();
        send_closure(message).await.unwrap();
        assert_eq!(ProverState::PresentationSet, self.prover.get_state());
    }

    pub async fn ensure_presentation_verified(&mut self) {
        prover_update_with_mediator(&mut self.prover, &self.agency_client, &self.connection)
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
