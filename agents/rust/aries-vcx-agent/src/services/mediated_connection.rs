use std::sync::Arc;

use crate::error::*;
use crate::storage::object_cache::ObjectCache;
use crate::storage::Storage;
use aries_vcx::common::ledger::transactions::into_did_doc;
use aries_vcx::core::profile::profile::Profile;
use aries_vcx::handlers::util::AnyInvitation;
use aries_vcx::messages::msg_fields::protocols::connection::request::Request;
use aries_vcx::messages::msg_fields::protocols::connection::Connection;
use aries_vcx::messages::msg_fields::protocols::cred_issuance::offer_credential::OfferCredential;
use aries_vcx::messages::msg_fields::protocols::cred_issuance::propose_credential::ProposeCredential;
use aries_vcx::messages::msg_fields::protocols::cred_issuance::CredentialIssuance;
use aries_vcx::messages::msg_fields::protocols::present_proof::propose::ProposePresentation;
use aries_vcx::messages::msg_fields::protocols::present_proof::PresentProof;
use aries_vcx::{
    agency_client::{agency_client::AgencyClient, configuration::AgencyClientConfig},
    handlers::connection::mediated_connection::{ConnectionState, MediatedConnection},
};
use aries_vcx_core::wallet::agency_client_wallet::ToBaseAgencyClientWallet;

pub struct ServiceMediatedConnections {
    profile: Arc<dyn Profile>,
    config_agency_client: AgencyClientConfig,
    mediated_connections: Arc<ObjectCache<MediatedConnection>>,
}

impl ServiceMediatedConnections {
    pub fn new(profile: Arc<dyn Profile>, config_agency_client: AgencyClientConfig) -> Self {
        Self {
            profile,
            config_agency_client,
            mediated_connections: Arc::new(ObjectCache::new("mediated-connections")),
        }
    }

    fn agency_client(&self) -> AgentResult<AgencyClient> {
        AgencyClient::new()
            .configure(
                self.profile.inject_wallet().to_base_agency_client_wallet(),
                &self.config_agency_client,
            )
            .map_err(|err| {
                AgentError::from_msg(
                    AgentErrorKind::GenericAriesVcxError,
                    &format!("Failed to configure agency client: {}", err),
                )
            })
    }

    pub async fn create_invitation(&self) -> AgentResult<AnyInvitation> {
        let mut connection = MediatedConnection::create("", &self.profile, &self.agency_client()?, true).await?;
        connection.connect(&self.profile, &self.agency_client()?, None).await?;
        let invite = connection
            .get_invite_details()
            .ok_or_else(|| AgentError::from_kind(AgentErrorKind::InviteDetails))?
            .clone();
        self.mediated_connections
            .insert(&connection.get_thread_id(), connection)?;
        Ok(invite)
    }

    pub async fn receive_invitation(&self, invite: AnyInvitation) -> AgentResult<String> {
        let ddo = into_did_doc(&self.profile, &invite).await?;
        let connection =
            MediatedConnection::create_with_invite("", &self.profile, &self.agency_client()?, invite, ddo, true)
                .await?;
        self.mediated_connections
            .insert(&connection.get_thread_id(), connection)
    }

    pub async fn send_request(&self, thread_id: &str) -> AgentResult<()> {
        let mut connection = self.mediated_connections.get(thread_id)?;
        connection.connect(&self.profile, &self.agency_client()?, None).await?;
        connection
            .find_message_and_update_state(&self.profile, &self.agency_client()?)
            .await?;
        self.mediated_connections.insert(thread_id, connection)?;
        Ok(())
    }

    pub async fn accept_request(&self, thread_id: &str, request: Request) -> AgentResult<()> {
        let mut connection = self.mediated_connections.get(thread_id)?;
        connection
            .process_request(&self.profile, &self.agency_client()?, request)
            .await?;
        connection.send_response(&self.profile).await?;
        self.mediated_connections.insert(thread_id, connection)?;
        Ok(())
    }

    pub async fn send_ping(&self, thread_id: &str) -> AgentResult<()> {
        let mut connection = self.mediated_connections.get(thread_id)?;
        connection.send_ping(&self.profile, None).await?;
        self.mediated_connections.insert(thread_id, connection)?;
        Ok(())
    }

    pub fn get_state(&self, thread_id: &str) -> AgentResult<ConnectionState> {
        Ok(self.mediated_connections.get(thread_id)?.get_state())
    }

    pub async fn update_state(&self, thread_id: &str) -> AgentResult<ConnectionState> {
        let mut connection = self.mediated_connections.get(thread_id)?;
        connection
            .find_message_and_update_state(&self.profile, &self.agency_client()?)
            .await?;
        self.mediated_connections.insert(thread_id, connection)?;
        Ok(self.mediated_connections.get(thread_id)?.get_state())
    }

    pub fn exists_by_id(&self, thread_id: &str) -> bool {
        self.mediated_connections.contains_key(thread_id)
    }
}

macro_rules! get_messages (($msg_type:ty, $a2a_msg:ident, $var:ident, $name:ident) => (
    impl ServiceMediatedConnections {
        pub async fn $name(&self, thread_id: &str) -> AgentResult<Vec<$msg_type>> {
            let connection = self.mediated_connections.get(thread_id)?;
            let agency_client = self.agency_client()?;
            let mut messages = Vec::<$msg_type>::new();
            for (uid, message) in connection.get_messages_noauth(&agency_client).await?.into_iter() {
                if let aries_vcx::messages::AriesMessage::$a2a_msg($a2a_msg::$var(message)) = message {
                    connection
                        .update_message_status(&uid, &agency_client)
                        .await
                        .ok();
                    messages.push(message);
                }
            }
            Ok(messages)
        }
    }
));

get_messages!(Request, Connection, Request, get_connection_requests);
get_messages!(
    ProposeCredential,
    CredentialIssuance,
    ProposeCredential,
    get_credential_proposals
);
get_messages!(
    OfferCredential,
    CredentialIssuance,
    OfferCredential,
    get_credential_offers
);
get_messages!(
    ProposePresentation,
    PresentProof,
    ProposePresentation,
    get_proof_proposals
);
