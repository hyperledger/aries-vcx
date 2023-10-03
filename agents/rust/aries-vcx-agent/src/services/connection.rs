use std::sync::{Arc, Mutex};

use aries_vcx::{
    core::profile::{profile::Profile, vdrtools_profile::VdrtoolsProfile},
    handlers::util::AnyInvitation,
    messages::msg_fields::protocols::{
        connection::{request::Request, response::Response},
        notification::ack::Ack,
    },
    protocols::connection::{
        pairwise_info::PairwiseInfo, Connection, GenericConnection, State, ThinState,
    },
};
use url::Url;

use crate::{
    error::*,
    http_client::HttpClient,
    storage::{object_cache::ObjectCache, Storage},
};

pub type ServiceEndpoint = Url;

pub struct ServiceConnections {
    profile: Arc<VdrtoolsProfile>,
    service_endpoint: ServiceEndpoint,
    connections: Arc<ObjectCache<GenericConnection>>,
}

impl ServiceConnections {
    pub fn new(profile: Arc<VdrtoolsProfile>, service_endpoint: ServiceEndpoint) -> Self {
        Self {
            profile,
            service_endpoint,
            connections: Arc::new(ObjectCache::new("connections")),
        }
    }

    pub async fn create_invitation(
        &self,
        pw_info: Option<PairwiseInfo>,
    ) -> AgentResult<AnyInvitation> {
        let pw_info = pw_info.unwrap_or(PairwiseInfo::create(self.profile.wallet()).await?);
        let inviter = Connection::new_inviter("".to_owned(), pw_info)
            .create_invitation(vec![], self.service_endpoint.clone());
        let invite = inviter.get_invitation().clone();
        let thread_id = inviter.thread_id().to_owned();

        self.connections.insert(&thread_id, inviter.into())?;

        Ok(invite)
    }

    pub async fn receive_invitation(&self, invite: AnyInvitation) -> AgentResult<String> {
        let pairwise_info = PairwiseInfo::create(self.profile.wallet()).await?;
        let invitee = Connection::new_invitee("".to_owned(), pairwise_info)
            .accept_invitation(self.profile.ledger_read(), invite)
            .await?;

        let thread_id = invitee.thread_id().to_owned();

        self.connections.insert(&thread_id, invitee.into())
    }

    pub async fn send_request(&self, thread_id: &str) -> AgentResult<()> {
        let invitee: Connection<_, _> = self.connections.get(thread_id)?.try_into()?;
        let invitee = invitee
            .prepare_request(self.service_endpoint.clone(), vec![])
            .await?;
        let request = invitee.get_request().clone();
        invitee
            .send_message(self.profile.wallet(), &request.into(), &HttpClient)
            .await?;
        self.connections.insert(thread_id, invitee.into())?;
        Ok(())
    }

    pub async fn accept_request(&self, thread_id: &str, request: Request) -> AgentResult<()> {
        let inviter = self.connections.get(thread_id)?;

        let inviter = match inviter.state() {
            ThinState::Inviter(State::Initial) => Connection::try_from(inviter)
                .map_err(From::from)
                .map(|c| c.into_invited(&request.id)),
            ThinState::Inviter(State::Invited) => Connection::try_from(inviter).map_err(From::from),
            s => Err(AgentError::from_msg(
                AgentErrorKind::GenericAriesVcxError,
                &format!(
                    "Connection with handle {} cannot process a request; State: {:?}",
                    thread_id, s
                ),
            )),
        }?;

        let inviter = inviter
            .handle_request(
                self.profile.wallet(),
                request,
                self.service_endpoint.clone(),
                vec![],
            )
            .await?;

        self.connections.insert(thread_id, inviter.into())?;

        Ok(())
    }

    pub async fn send_response(&self, thread_id: &str) -> AgentResult<()> {
        let inviter: Connection<_, _> = self.connections.get(thread_id)?.try_into()?;
        let response = inviter.get_connection_response_msg();
        inviter
            .send_message(self.profile.wallet(), &response.into(), &HttpClient)
            .await?;

        self.connections.insert(thread_id, inviter.into())?;

        Ok(())
    }

    pub async fn accept_response(&self, thread_id: &str, response: Response) -> AgentResult<()> {
        let invitee: Connection<_, _> = self.connections.get(thread_id)?.try_into()?;
        let invitee = invitee
            .handle_response(self.profile.wallet(), response)
            .await?;

        self.connections.insert(thread_id, invitee.into())?;

        Ok(())
    }

    pub async fn send_ack(&self, thread_id: &str) -> AgentResult<()> {
        let invitee: Connection<_, _> = self.connections.get(thread_id)?.try_into()?;
        invitee
            .send_message(
                self.profile.wallet(),
                &invitee.get_ack().into(),
                &HttpClient,
            )
            .await?;

        self.connections.insert(thread_id, invitee.into())?;

        Ok(())
    }

    pub async fn process_ack(&self, thread_id: &str, ack: Ack) -> AgentResult<()> {
        let inviter: Connection<_, _> = self.connections.get(thread_id)?.try_into()?;
        let inviter = inviter.acknowledge_connection(&ack.into())?;

        self.connections.insert(thread_id, inviter.into())?;

        Ok(())
    }

    pub fn get_state(&self, thread_id: &str) -> AgentResult<ThinState> {
        Ok(self.connections.get(thread_id)?.state())
    }

    pub(in crate::services) fn get_by_id(&self, thread_id: &str) -> AgentResult<GenericConnection> {
        self.connections.get(thread_id)
    }

    pub fn get_by_their_vk(&self, their_vk: &str) -> AgentResult<Vec<String>> {
        let their_vk = their_vk.to_string();
        let f = |(id, m): (&String, &Mutex<GenericConnection>)| -> Option<String> {
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
