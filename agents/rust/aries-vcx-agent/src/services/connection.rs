use std::sync::{Arc, Mutex};

use crate::error::*;
use crate::http_client::HttpClient;
use crate::storage::object_cache::ObjectCache;
use crate::storage::Storage;
use aries_vcx::core::profile::profile::Profile;
use aries_vcx::messages::a2a::A2AMessage;
use aries_vcx::messages::concepts::ack::Ack;
use aries_vcx::messages::protocols::connection::invite::Invitation;
use aries_vcx::messages::protocols::connection::request::Request;
use aries_vcx::messages::protocols::connection::response::SignedResponse;
use aries_vcx::protocols::typestate_con::pairwise_info::PairwiseInfo;
use aries_vcx::protocols::typestate_con::{Connection, State, VagueConnection};

pub type ServiceEndpoint = String;

pub struct ServiceConnections {
    profile: Arc<dyn Profile>,
    service_endpoint: ServiceEndpoint,
    connections: Arc<ObjectCache<VagueConnection>>,
}

impl ServiceConnections {
    pub fn new(profile: Arc<dyn Profile>, service_endpoint: ServiceEndpoint) -> Self {
        Self {
            profile,
            service_endpoint,
            connections: Arc::new(ObjectCache::new("connections")),
        }
    }

    pub async fn create_invitation(&self, pw_info: Option<PairwiseInfo>) -> AgentResult<Invitation> {
        let pw_info = pw_info.unwrap_or(PairwiseInfo::create(&self.profile.inject_wallet()).await?);
        let inviter = Connection::new_inviter("".to_owned(), pw_info, vec![], self.service_endpoint.clone());
        let invite = inviter.get_invitation().clone();
        let thread_id = inviter.thread_id().to_owned();

        self.connections.insert(&thread_id, inviter.into())?;

        Ok(invite)
    }

    pub async fn receive_invitation(&self, invite: Invitation) -> AgentResult<String> {
        let pairwise_info = PairwiseInfo::create(&self.profile.inject_wallet()).await?;
        let invitee = Connection::new_invitee("".to_owned(), pairwise_info)
            .accept_invitation(&self.profile, &invite)
            .await?;

        let thread_id = invitee.thread_id().to_owned();

        self.connections.insert(&thread_id, invitee.into())
    }

    pub async fn send_request(&self, thread_id: &str) -> AgentResult<()> {
        let invitee: Connection<_, _> = self.connections.get(thread_id)?.try_into()?;
        let invitee = invitee
            .send_request(
                &self.profile.inject_wallet(),
                self.service_endpoint.clone(),
                vec![],
                &HttpClient,
            )
            .await?;

        self.connections.insert(thread_id, invitee.into())?;
        Ok(())
    }

    pub async fn accept_request(&self, thread_id: &str, request: Request) -> AgentResult<()> {
        let inviter: Connection<_, _> = self.connections.get(thread_id)?.try_into()?;
        let inviter = inviter
            .handle_request(
                &self.profile.inject_wallet(),
                request,
                self.service_endpoint.clone(),
                vec![],
                &HttpClient,
            )
            .await?;

        self.connections.insert(thread_id, inviter.into())?;

        Ok(())
    }

    pub async fn send_response(&self, thread_id: &str) -> AgentResult<()> {
        let inviter: Connection<_, _> = self.connections.get(thread_id)?.try_into()?;
        let inviter = inviter
            .send_response(&self.profile.inject_wallet(), &HttpClient)
            .await?;

        self.connections.insert(thread_id, inviter.into())?;

        Ok(())
    }

    pub async fn accept_response(&self, thread_id: &str, response: SignedResponse) -> AgentResult<()> {
        let invitee: Connection<_, _> = self.connections.get(thread_id)?.try_into()?;
        let invitee = invitee
            .handle_response(&self.profile.inject_wallet(), response, &HttpClient)
            .await?;

        self.connections.insert(thread_id, invitee.into())?;

        Ok(())
    }

    pub async fn send_ack(&self, thread_id: &str) -> AgentResult<()> {
        let invitee: Connection<_, _> = self.connections.get(thread_id)?.try_into()?;
        let invitee = invitee.send_ack(&self.profile.inject_wallet(), &HttpClient).await?;

        self.connections.insert(thread_id, invitee.into())?;

        Ok(())
    }

    pub async fn process_ack(&self, thread_id: &str, ack: Ack) -> AgentResult<()> {
        let inviter: Connection<_, _> = self.connections.get(thread_id)?.try_into()?;
        let inviter = inviter.acknowledge_connection(&A2AMessage::Ack(ack))?;

        self.connections.insert(thread_id, inviter.into())?;

        Ok(())
    }

    pub fn get_state(&self, thread_id: &str) -> AgentResult<State> {
        Ok(self.connections.get(thread_id)?.state())
    }

    pub(in crate::services) fn get_by_id(&self, thread_id: &str) -> AgentResult<VagueConnection> {
        self.connections.get(thread_id)
    }

    pub fn get_by_their_vk(&self, their_vk: &str) -> AgentResult<Vec<String>> {
        let their_vk = their_vk.to_string();
        let f = |(id, m): (&String, &Mutex<VagueConnection>)| -> Option<String> {
            let connection = m.lock().unwrap();
            match connection.remote_vk() {
                Ok(remote_vk) if remote_vk == their_vk => Some(id.to_string()),
                _ => None,
            }
        };
        self.connections.find_by(f)
    }

    pub fn exists_by_id(&self, thread_id: &str) -> bool {
        self.connections.contains_key(thread_id)
    }
}
