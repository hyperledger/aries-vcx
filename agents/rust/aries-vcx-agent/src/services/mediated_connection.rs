use std::sync::Arc;

use crate::error::*;
use crate::storage::object_cache::ObjectCache;
use aries_vcx::messages::connection::invite::Invitation;
use aries_vcx::messages::connection::request::Request;
use aries_vcx::messages::issuance::credential_offer::CredentialOffer;
use aries_vcx::messages::issuance::credential_proposal::CredentialProposal;
use aries_vcx::messages::proof_presentation::presentation_proposal::PresentationProposal;
use aries_vcx::messages::proof_presentation::presentation_request::PresentationRequest;
use aries_vcx::{
    agency_client::{agency_client::AgencyClient, configuration::AgencyClientConfig},
    handlers::connection::mediated_connection::{MediatedConnection, ConnectionState},
    indy::ledger::transactions::into_did_doc,
    messages::a2a::A2AMessage,
    vdrtools::{PoolHandle, WalletHandle},
};

pub struct ServiceMediatedConnections {
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    config_agency_client: AgencyClientConfig,
    mediated_connections: Arc<ObjectCache<MediatedConnection>>,
}

impl ServiceMediatedConnections {
    pub fn new(
        wallet_handle: WalletHandle,
        pool_handle: PoolHandle,
        config_agency_client: AgencyClientConfig,
    ) -> Self {
        Self {
            wallet_handle,
            pool_handle,
            config_agency_client,
            mediated_connections: Arc::new(ObjectCache::new("mediated-connections")),
        }
    }

    fn agency_client(&self) -> AgentResult<AgencyClient> {
        AgencyClient::new()
            .configure(self.wallet_handle, &self.config_agency_client)
            .map_err(|err| {
                AgentError::from_msg(
                    AgentErrorKind::GenericAriesVcxError,
                    &format!("Failed to configure agency client: {}", err),
                )
            })
    }

    pub async fn create_invitation(&self) -> AgentResult<Invitation> {
        let mut connection =
            MediatedConnection::create("", self.wallet_handle, &self.agency_client()?, true).await?;
        connection
            .connect(self.wallet_handle, &self.agency_client()?)
            .await?;
        let invite = connection
            .get_invite_details()
            .ok_or_else(|| AgentError::from_kind(AgentErrorKind::InviteDetails))?
            .clone();
        self.mediated_connections
            .insert(&connection.get_thread_id(), connection)?;
        Ok(invite)
    }

    pub async fn receive_invitation(&self, invite: Invitation) -> AgentResult<String> {
        let ddo = into_did_doc(self.pool_handle, &invite).await?;
        let connection = MediatedConnection::create_with_invite(
            "",
            self.wallet_handle,
            &self.agency_client()?,
            invite,
            ddo,
            true,
        )
        .await?;
        self.mediated_connections
            .insert(&connection.get_thread_id(), connection)
    }

    pub async fn send_request(&self, thread_id: &str) -> AgentResult<()> {
        let mut connection = self.mediated_connections.get(thread_id)?;
        connection
            .connect(self.wallet_handle, &self.agency_client()?)
            .await?;
        connection
            .find_message_and_update_state(self.wallet_handle, &self.agency_client()?)
            .await?;
        self.mediated_connections.insert(thread_id, connection)?;
        Ok(())
    }

    pub async fn accept_request(&self, thread_id: &str, request: Request) -> AgentResult<()> {
        let mut connection = self.mediated_connections.get(thread_id)?;
        connection
            .process_request(self.wallet_handle, &self.agency_client()?, request)
            .await?;
        connection.send_response(self.wallet_handle).await?;
        self.mediated_connections.insert(thread_id, connection)?;
        Ok(())
    }

    pub async fn send_ping(&self, thread_id: &str) -> AgentResult<()> {
        let mut connection = self.mediated_connections.get(thread_id)?;
        connection.send_ping(self.wallet_handle, None).await?;
        self.mediated_connections.insert(thread_id, connection)?;
        Ok(())
    }

    pub fn get_state(&self, thread_id: &str) -> AgentResult<ConnectionState> {
        Ok(self.mediated_connections.get(thread_id)?.get_state())
    }

    pub async fn update_state(&self, thread_id: &str) -> AgentResult<ConnectionState> {
        let mut connection = self.mediated_connections.get(thread_id)?;
        connection
            .find_message_and_update_state(self.wallet_handle, &self.agency_client()?)
            .await?;
        self.mediated_connections.insert(thread_id, connection)?;
        Ok(self.mediated_connections.get(thread_id)?.get_state())
    }

    pub fn exists_by_id(&self, thread_id: &str) -> bool {
        self.mediated_connections.contains_key(thread_id)
    }

    pub async fn get_all_proof_requests(&self) -> AgentResult<Vec<(PresentationRequest, String)>> {
        let agency_client = self.agency_client()?;
        let mut requests = Vec::<(PresentationRequest, String)>::new();
        for connection in self.mediated_connections.get_all()? {
            for (uid, message) in connection.get_messages(&agency_client).await?.into_iter() {
                if let A2AMessage::PresentationRequest(request) = message {
                    connection
                        .update_message_status(&uid, &agency_client)
                        .await
                        .ok();
                    requests.push((request, connection.get_thread_id()));
                }
            }
        }
        Ok(requests)
    }
}

macro_rules! get_messages (($msg_type:ty, $a2a_msg:ident, $name:ident) => (
    impl ServiceMediatedConnections {
        pub async fn $name(&self, thread_id: &str) -> AgentResult<Vec<$msg_type>> {
            let connection = self.mediated_connections.get(thread_id)?;
            let agency_client = self.agency_client()?;
            let mut messages = Vec::<$msg_type>::new();
            for (uid, message) in connection.get_messages_noauth(&agency_client).await?.into_iter() {
                if let A2AMessage::$a2a_msg(message) = message {
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

get_messages!(Request, ConnectionRequest, get_connection_requests);
get_messages!(CredentialProposal, CredentialProposal, get_credential_proposals);
get_messages!(CredentialOffer, CredentialOffer, get_credential_offers);
get_messages!(PresentationProposal, PresentationProposal, get_proof_proposals);
