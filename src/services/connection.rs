use std::sync::Arc;

use crate::error::*;
use crate::storage::in_memory::ObjectCache;
use aries_vcx::messages::connection::invite::Invitation;
use aries_vcx::messages::connection::request::Request;
use aries_vcx::messages::issuance::credential_offer::CredentialOffer;
use aries_vcx::messages::issuance::credential_proposal::CredentialProposal;
use aries_vcx::messages::proof_presentation::presentation_proposal::PresentationProposal;
use aries_vcx::messages::proof_presentation::presentation_request::PresentationRequest;
use aries_vcx::{
    agency_client::{agency_client::AgencyClient, configuration::AgencyClientConfig},
    handlers::connection::connection::{Connection, ConnectionState},
    indy::ledger::transactions::into_did_doc,
    messages::a2a::A2AMessage,
    vdrtools_sys::{PoolHandle, WalletHandle},
};

pub struct ServiceConnections {
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    config_agency_client: AgencyClientConfig,
    connections: Arc<ObjectCache<Connection>>,
}

impl ServiceConnections {
    pub fn new(
        wallet_handle: WalletHandle,
        pool_handle: PoolHandle,
        config_agency_client: AgencyClientConfig,
    ) -> Self {
        Self {
            wallet_handle,
            pool_handle,
            config_agency_client,
            connections: Arc::new(ObjectCache::new("connections")),
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
            Connection::create("", self.wallet_handle, &self.agency_client()?, true).await?;
        connection
            .connect(self.wallet_handle, &self.agency_client()?)
            .await?;
        let invite = connection
            .get_invite_details()
            .ok_or_else(|| AgentError::from_kind(AgentErrorKind::InviteDetails))?
            .clone();
        self.connections
            .set(&connection.get_thread_id(), connection)?;
        Ok(invite)
    }

    pub async fn receive_invitation(&self, invite: Invitation) -> AgentResult<String> {
        let ddo = into_did_doc(self.pool_handle, &invite).await?;
        let connection = Connection::create_with_invite(
            "",
            self.wallet_handle,
            &self.agency_client()?,
            invite,
            ddo,
            true,
        )
        .await?;
        self.connections
            .set(&connection.get_thread_id(), connection)
    }

    pub async fn send_request(&self, id: &str) -> AgentResult<()> {
        let mut connection = self.connections.get(id)?;
        connection
            .connect(self.wallet_handle, &self.agency_client()?)
            .await?;
        connection
            .find_message_and_update_state(self.wallet_handle, &self.agency_client()?)
            .await?;
        self.connections.set(id, connection)?;
        Ok(())
    }

    pub async fn accept_request(&self, id: &str, request: Request) -> AgentResult<()> {
        let mut connection = self.connections.get(id)?;
        connection
            .process_request(self.wallet_handle, &self.agency_client()?, request)
            .await?;
        connection.send_response(self.wallet_handle).await?;
        self.connections.set(id, connection)?;
        Ok(())
    }

    pub async fn send_ping(&self, id: &str) -> AgentResult<()> {
        let mut connection = self.connections.get(id)?;
        connection.send_ping(self.wallet_handle, None).await?;
        self.connections.set(id, connection)?;
        Ok(())
    }

    pub fn get_state(&self, id: &str) -> AgentResult<ConnectionState> {
        Ok(self.connections.get(id)?.get_state())
    }

    pub async fn update_state(&self, id: &str) -> AgentResult<ConnectionState> {
        let mut connection = self.connections.get(id)?;
        connection
            .find_message_and_update_state(self.wallet_handle, &self.agency_client()?)
            .await?;
        self.connections.set(id, connection)?;
        Ok(self.connections.get(id)?.get_state())
    }

    pub(in crate::services) fn get_by_id(&self, id: &str) -> AgentResult<Connection> {
        self.connections.get(id)
    }

    pub fn exists_by_id(&self, id: &str) -> bool {
        self.connections.has_id(id)
    }

    // TODO: Make the following functions generic
    pub async fn get_connection_requests(&self, id: &str) -> AgentResult<Vec<Request>> {
        let connection = self.connections.get(id)?;
        let agency_client = self.agency_client()?;
        let mut requests = Vec::<Request>::new();
        for (uid, message) in connection.get_messages_noauth(&agency_client).await?.into_iter() {
            if let A2AMessage::ConnectionRequest(request) = message {
                connection
                    .update_message_status(&uid, &agency_client)
                    .await
                    .ok();
                requests.push(request);
            }
        }
        Ok(requests)
    }

    pub async fn get_credential_proposals(&self, id: &str) -> AgentResult<Vec<CredentialProposal>> {
        let connection = self.connections.get(id)?;
        let agency_client = self.agency_client()?;
        let mut proposals = Vec::<CredentialProposal>::new();
        for (uid, message) in connection.get_messages(&agency_client).await?.into_iter() {
            if let A2AMessage::CredentialProposal(proposal) = message {
                connection
                    .update_message_status(&uid, &agency_client)
                    .await
                    .ok();
                proposals.push(proposal);
            }
        }
        Ok(proposals)
    }

    pub async fn get_credential_offers(&self, id: &str) -> AgentResult<Vec<CredentialOffer>> {
        let connection = self.connections.get(id)?;
        let agency_client = self.agency_client()?;
        let mut offers = Vec::<CredentialOffer>::new();
        for (uid, message) in connection.get_messages(&agency_client).await?.into_iter() {
            if let A2AMessage::CredentialOffer(offer) = message {
                connection
                    .update_message_status(&uid, &agency_client)
                    .await
                    .ok();
                offers.push(offer);
            }
        }
        Ok(offers)
    }

    pub async fn get_proof_requests(&self, id: &str) -> AgentResult<Vec<PresentationRequest>> {
        let connection = self.connections.get(id)?;
        let agency_client = self.agency_client()?;
        let mut requests = Vec::<PresentationRequest>::new();
        for (uid, message) in connection.get_messages(&agency_client).await?.into_iter() {
            if let A2AMessage::PresentationRequest(request) = message {
                connection
                    .update_message_status(&uid, &agency_client)
                    .await
                    .ok();
                requests.push(request);
            }
        }
        Ok(requests)
    }

    pub async fn get_proof_proposals(&self, id: &str) -> AgentResult<Vec<PresentationProposal>> {
        let connection = self.connections.get(id)?;
        let agency_client = self.agency_client()?;
        let mut proposals = Vec::<PresentationProposal>::new();
        for (uid, message) in connection.get_messages(&agency_client).await?.into_iter() {
            if let A2AMessage::PresentationProposal(proposal) = message {
                connection
                    .update_message_status(&uid, &agency_client)
                    .await
                    .ok();
                proposals.push(proposal);
            }
        }
        Ok(proposals)
    }

    pub async fn get_all_proof_requests(&self) -> AgentResult<Vec<(PresentationRequest, String)>> {
        let agency_client = self.agency_client()?;
        let mut requests = Vec::<(PresentationRequest, String)>::new();
        for connection in self.connections.get_all()? {
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
