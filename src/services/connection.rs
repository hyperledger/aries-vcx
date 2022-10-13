use std::sync::Arc;

use crate::agent::Agent;
use crate::error::*;
use crate::storage::in_memory::ObjectCache;
use aries_vcx::messages::connection::invite::{Invitation, PairwiseInvitation};
use aries_vcx::messages::connection::request::Request;
use aries_vcx::messages::issuance::credential_offer::CredentialOffer;
use aries_vcx::messages::issuance::credential_proposal::CredentialProposal;
use aries_vcx::messages::proof_presentation::presentation_request::PresentationRequest;
use aries_vcx::{
    agency_client::{agency_client::AgencyClient, configuration::AgencyClientConfig},
    handlers::connection::connection::{Connection, ConnectionState},
    indy::ledger::transactions::into_did_doc,
    messages::a2a::A2AMessage,
    protocols::connection::inviter::state_machine::InviterState,
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
            .ok_or(AgentError::from_kind(AgentErrorKind::InviteDetails))?
            .clone();
        self.connections
            .add(&connection.get_thread_id(), connection)?;
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
            .add(&connection.get_thread_id(), connection)
    }

    pub async fn send_request(&self, id: &str) -> AgentResult<()> {
        let mut connection = self.connections.get_cloned(id)?;
        connection
            .connect(self.wallet_handle, &self.agency_client()?)
            .await?;
        connection
            .find_message_and_update_state(self.wallet_handle, &self.agency_client()?)
            .await?;
        self.connections.add(id, connection)?;
        Ok(())
    }

    pub async fn accept_request(&self, id: &str, request: Request) -> AgentResult<()> {
        let mut connection = self.connections.get_cloned(id)?;
        connection
            .process_request(self.wallet_handle, &self.agency_client()?, request)
            .await?;
        self.connections.add(id, connection)?;
        Ok(())
    }

    pub async fn send_ping(&self, id: &str) -> AgentResult<()> {
        let mut connection = self.connections.get_cloned(id)?;
        connection.send_ping(self.wallet_handle, None).await?;
        self.connections.add(id, connection)?;
        Ok(())
    }

    pub fn get_state(&self, id: &str) -> AgentResult<ConnectionState> {
        Ok(self.connections.get_cloned(id)?.get_state())
    }

    pub async fn update_state(&self, id: &str) -> AgentResult<ConnectionState> {
        let mut connection = self.connections.get_cloned(id)?;
        connection
            .find_message_and_update_state(self.wallet_handle, &self.agency_client()?)
            .await?;
        self.connections.add(id, connection)?;
        Ok(self.connections.get_cloned(id)?.get_state())
    }

    // TODO: Probably should not expose Connection. This can be existence check
    // and other services should reference Connection storage. Or this should be
    // exposed only to other services.
    pub fn get_by_id(&self, id: &str) -> AgentResult<Connection> {
        self.connections.get_cloned(id)
    }

    // TODO: Make the following functions generic
    pub async fn get_connection_requests(&self, id: &str) -> AgentResult<Vec<Request>> {
        let connection = self.connections.get_cloned(id)?;
        let agency_client = self.agency_client()?;
        Ok(connection
            .get_messages_noauth(&agency_client)
            .await?
            .into_iter()
            .filter_map(|(_, message)| match message {
                A2AMessage::ConnectionRequest(request) => Some(request),
                _ => None,
            })
            .collect())
    }

    pub async fn get_credential_proposals(&self, id: &str) -> AgentResult<Vec<CredentialProposal>> {
        let connection = self.connections.get_cloned(id)?;
        let agency_client = self.agency_client()?;
        let mut proposals = Vec::<CredentialProposal>::new();
        for (uid, message) in connection.get_messages(&agency_client).await?.into_iter() {
            match message {
                A2AMessage::CredentialProposal(proposal) => {
                    connection
                        .update_message_status(&uid, &agency_client)
                        .await
                        .ok();
                    proposals.push(proposal);
                }
                _ => {}
            }
        }
        Ok(proposals)
    }

    pub async fn get_credential_offers(&self, id: &str) -> AgentResult<Vec<CredentialOffer>> {
        let connection = self.connections.get_cloned(id)?;
        let agency_client = self.agency_client()?;
        let mut offers = Vec::<CredentialOffer>::new();
        for (uid, message) in connection.get_messages(&agency_client).await?.into_iter() {
            match message {
                A2AMessage::CredentialOffer(offer) => {
                    connection
                        .update_message_status(&uid, &agency_client)
                        .await
                        .ok();
                    offers.push(offer);
                }
                _ => {}
            }
        }
        Ok(offers)
    }

    pub async fn get_proof_requests(&self, id: &str) -> AgentResult<Vec<PresentationRequest>> {
        let connection = self.connections.get_cloned(id)?;
        let agency_client = self.agency_client()?;
        let mut requests = Vec::<PresentationRequest>::new();
        for (uid, message) in connection.get_messages(&agency_client).await?.into_iter() {
            match message {
                A2AMessage::PresentationRequest(request) => {
                    connection
                        .update_message_status(&uid, &agency_client)
                        .await
                        .ok();
                    requests.push(request);
                }
                _ => {}
            }
        }
        Ok(requests)
    }
}
