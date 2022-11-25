use std::sync::{Arc, Mutex};

use crate::error::*;
use crate::storage::object_cache::ObjectCache;
use aries_vcx::core::profile::profile::Profile;
use aries_vcx::handlers::connection::connection::{Connection, ConnectionState};
use aries_vcx::messages::a2a::A2AMessage;
use aries_vcx::messages::ack::Ack;
use aries_vcx::messages::connection::invite::Invitation;
use aries_vcx::messages::connection::request::Request;
use aries_vcx::messages::connection::response::SignedResponse;
use aries_vcx::xyz::ledger::transactions::into_did_doc;

pub type ServiceEndpoint = String;

pub struct ServiceConnections {
    profile: Arc<dyn Profile>,
    service_endpoint: ServiceEndpoint,
    connections: Arc<ObjectCache<Connection>>,
}

impl ServiceConnections {
    pub fn new(profile: Arc<dyn Profile>, service_endpoint: ServiceEndpoint) -> Self {
        Self {
            profile,
            service_endpoint,
            connections: Arc::new(ObjectCache::new("connections")),
        }
    }

    pub async fn create_invitation(&self) -> AgentResult<Invitation> {
        let inviter = Connection::create_inviter(&self.profile)
            .await?
            .create_invite(self.service_endpoint.clone(), vec![])
            .await?;
        let invite = inviter
            .get_invite_details()
            .ok_or_else(|| AgentError::from_kind(AgentErrorKind::InviteDetails))?
            .clone();
        self.connections.set(&inviter.get_thread_id(), inviter)?;
        Ok(invite)
    }

    pub async fn receive_invitation(&self, invite: Invitation) -> AgentResult<String> {
        let did_doc = into_did_doc(&self.profile, &invite).await?;
        let invitee = Connection::create_invitee(&self.profile, did_doc)
            .await?
            .process_invite(invite)?;
        self.connections.set(&invitee.get_thread_id(), invitee)
    }

    pub async fn send_request(&self, thread_id: &str) -> AgentResult<()> {
        let invitee = self
            .connections
            .get(thread_id)?
            .send_request(&self.profile, self.service_endpoint.clone(), vec![], None)
            .await?;
        self.connections.set(thread_id, invitee)?;
        Ok(())
    }

    pub async fn accept_request(&self, thread_id: &str, request: Request) -> AgentResult<()> {
        let inviter = self
            .connections
            .get(thread_id)?
            .process_request(&self.profile, request, self.service_endpoint.clone(), vec![], None)
            .await?;
        self.connections.set(thread_id, inviter)?;
        Ok(())
    }

    pub async fn send_response(&self, thread_id: &str) -> AgentResult<()> {
        let inviter = self
            .connections
            .get(thread_id)?
            .send_response(&self.profile, None)
            .await?;
        self.connections.set(thread_id, inviter)?;
        Ok(())
    }

    pub async fn accept_response(&self, thread_id: &str, response: SignedResponse) -> AgentResult<()> {
        let invitee = self
            .connections
            .get(thread_id)?
            .process_response(&self.profile, response, None)
            .await?;
        self.connections.set(thread_id, invitee)?;
        Ok(())
    }

    pub async fn send_ack(&self, thread_id: &str) -> AgentResult<()> {
        let invitee = self
            .connections
            .get(thread_id)?
            .send_ack(&self.profile, None)
            .await?;
        self.connections.set(thread_id, invitee)?;
        Ok(())
    }

    pub async fn process_ack(&self, thread_id: &str, ack: Ack) -> AgentResult<()> {
        let inviter = self
            .connections
            .get(thread_id)?
            .process_ack(A2AMessage::Ack(ack))
            .await?;
        self.connections.set(thread_id, inviter)?;
        Ok(())
    }

    pub fn get_state(&self, thread_id: &str) -> AgentResult<ConnectionState> {
        Ok(self.connections.get(thread_id)?.get_state())
    }

    pub(in crate::services) fn get_by_id(&self, thread_id: &str) -> AgentResult<Connection> {
        self.connections.get(thread_id)
    }

    pub fn get_by_their_vk(&self, their_vk: &str) -> AgentResult<Vec<String>> {
        let their_vk = their_vk.to_string();
        let f = |(id, m): (&String, &Mutex<Connection>)| -> Option<String> {
            let connection = m.lock().unwrap();
            match connection.remote_vk() {
                Ok(remote_vk) if remote_vk == their_vk => Some(id.to_string()),
                _ => None
            }
        };
        self.connections.find_by(f)
    }


    pub fn exists_by_id(&self, thread_id: &str) -> bool {
        self.connections.has_id(thread_id)
    }
}
