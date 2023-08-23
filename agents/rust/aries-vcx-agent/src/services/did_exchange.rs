use std::sync::Arc;

use aries_vcx::{
    core::profile::profile::Profile,
    messages::msg_fields::protocols::{
        did_exchange::{complete::Complete, request::Request, response::Response},
        out_of_band::invitation::Invitation as OobInvitation,
    },
    protocols::{
        connection::wrap_and_send_msg,
        did_exchange::{
            resolve_key_from_invitation,
            state_machine::{
                generic::{GenericDidExchange, ThinState},
                requester::{ConstructRequestConfig, PairwiseConstructRequestConfig, PublicConstructRequestConfig},
                responder::ReceiveRequestConfig,
            },
        },
    },
    utils::from_did_doc_sov_to_legacy,
};
use did_resolver_registry::ResolverRegistry;

use crate::{
    http_client::HttpClient,
    storage::{object_cache::ObjectCache, Storage},
    AgentError, AgentErrorKind, AgentResult,
};

use super::connection::ServiceEndpoint;

pub struct ServiceDidExchange {
    profile: Arc<dyn Profile>,
    resolver_registry: Arc<ResolverRegistry>,
    service_endpoint: ServiceEndpoint,
    did_exchange: Arc<ObjectCache<GenericDidExchange>>,
    public_did: String,
}

impl ServiceDidExchange {
    pub fn new(
        profile: Arc<dyn Profile>,
        resolver_registry: Arc<ResolverRegistry>,
        service_endpoint: ServiceEndpoint,
        public_did: String,
    ) -> Self {
        Self {
            profile,
            service_endpoint,
            resolver_registry,
            did_exchange: Arc::new(ObjectCache::new("did-exchange")),
            public_did,
        }
    }

    pub async fn send_request_public(&self, their_did: String) -> AgentResult<String> {
        let config = ConstructRequestConfig::Public(PublicConstructRequestConfig {
            wallet: self.profile.inject_wallet(),
            ledger: self.profile.inject_indy_ledger_read(),
            their_did: format!("did:sov:{}", their_did).parse()?,
            our_did: format!("did:sov:{}", self.public_did).parse()?,
        });
        let (requester, request) = GenericDidExchange::construct_request(config).await?;
        wrap_and_send_msg(
            &self.profile.inject_wallet(),
            &request.clone().into(),
            &requester
                .our_did_document()
                .resolved_key_agreement()
                .next()
                .ok_or_else(|| AgentError::from_msg(AgentErrorKind::InvalidState, "No key agreement method found"))?
                .public_key()?
                .base58(),
            &from_did_doc_sov_to_legacy(requester.their_did_doc().clone())?,
            &HttpClient,
        )
        .await?;
        let request_id = request
            .decorators
            .thread
            .ok_or_else(|| AgentError::from_msg(AgentErrorKind::InvalidState, "Request did not contain a thread id"))?
            .thid;
        self.did_exchange.insert(&request_id, requester.clone().into())
    }

    pub async fn send_request_pairwise(&self, invitation: OobInvitation) -> AgentResult<String> {
        let config = ConstructRequestConfig::Pairwise(PairwiseConstructRequestConfig {
            wallet: self.profile.inject_wallet(),
            invitation: invitation.clone(),
            resolver_registry: self.resolver_registry.clone(),
            service_endpoint: self.service_endpoint.clone(),
            routing_keys: vec![],
        });
        let (requester, request) = GenericDidExchange::construct_request(config).await?;
        wrap_and_send_msg(
            &self.profile.inject_wallet(),
            &request.clone().into(),
            &requester
                .our_did_document()
                .resolved_key_agreement()
                .next()
                .ok_or_else(|| AgentError::from_msg(AgentErrorKind::InvalidState, "No key agreement method found"))?
                .public_key()?
                .base58(),
            &from_did_doc_sov_to_legacy(requester.their_did_doc().clone())?,
            &HttpClient,
        )
        .await?;
        let request_id = request
            .decorators
            .thread
            .ok_or_else(|| AgentError::from_msg(AgentErrorKind::InvalidState, "Request did not contain a thread id"))?
            .thid;
        self.did_exchange.insert(&request_id, requester.clone().into())
    }

    pub async fn send_response(&self, request: Request, invitation: OobInvitation) -> AgentResult<String> {
        // TODO: We should fetch the out of band invite associated with the request.
        // We don't want to be sending response if we don't know if there is any invitation
        // associated with the request.
        let request_id = request
            .clone()
            .decorators
            .thread
            .ok_or_else(|| AgentError::from_msg(AgentErrorKind::InvalidState, "Request did not contain a thread id"))?
            .thid;
        let invitation_key = resolve_key_from_invitation(&invitation, &self.resolver_registry).await?;
        let (responder, response) = GenericDidExchange::handle_request(ReceiveRequestConfig {
            wallet: self.profile.inject_wallet(),
            resolver_registry: self.resolver_registry.clone(),
            request,
            service_endpoint: self.service_endpoint.clone(),
            routing_keys: vec![],
            invitation_id: invitation.id.clone(),
            invitation_key,
        })
        .await?;
        wrap_and_send_msg(
            &self.profile.inject_wallet(),
            &response.clone().into(),
            &responder
                .our_did_document()
                .resolved_key_agreement()
                .next()
                .ok_or_else(|| AgentError::from_msg(AgentErrorKind::InvalidState, "No key agreement method found"))?
                .public_key()?
                .base58(),
            &from_did_doc_sov_to_legacy(responder.their_did_doc().clone())?,
            &HttpClient,
        )
        .await?;
        self.did_exchange.insert(&request_id, responder.clone().into())
    }

    pub async fn send_complete(&self, thread_id: &str, response: Response) -> AgentResult<String> {
        let (requester, complete) = self.did_exchange.get(thread_id)?.handle_response(response).await?;
        wrap_and_send_msg(
            &self.profile.inject_wallet(),
            &complete.clone().into(),
            &requester
                .our_did_document()
                .resolved_key_agreement()
                .next()
                .ok_or_else(|| AgentError::from_msg(AgentErrorKind::InvalidState, "No key agreement method found"))?
                .public_key()?
                .base58(),
            &from_did_doc_sov_to_legacy(requester.their_did_doc().clone())?,
            &HttpClient,
        )
        .await?;
        self.did_exchange.insert(thread_id, requester.clone().into())
    }

    pub async fn receive_complete(&self, thread_id: &str, complete: Complete) -> AgentResult<String> {
        let requester = self.did_exchange.get(thread_id)?.handle_complete(complete)?;
        self.did_exchange.insert(thread_id, requester)
    }

    pub fn exists_by_id(&self, thread_id: &str) -> bool {
        self.did_exchange.contains_key(thread_id)
    }

    pub fn public_did(&self) -> &str {
        self.public_did.as_ref()
    }

    pub fn get_state(&self, thread_id: &str) -> AgentResult<ThinState> {
        Ok(self.did_exchange.get(thread_id)?.get_state())
    }
}
