use std::sync::Arc;

use aries_vcx::{
    messages::msg_fields::protocols::{
        did_exchange::{
            complete::Complete, problem_report::ProblemReport, request::Request, response::Response,
        },
        out_of_band::invitation::Invitation as OobInvitation,
    },
    protocols::did_exchange::{
        resolve_key_from_invitation,
        state_machine::generic::{GenericDidExchange, ThinState},
    },
    transport::Transport,
};
use aries_vcx_core::{ledger::indy_vdr_ledger::DefaultIndyLedgerRead, wallet::indy::IndySdkWallet};
use did_resolver_registry::ResolverRegistry;

use super::connection::ServiceEndpoint;
use crate::{
    helper::{get_their_endpoint, pairwise_encrypt},
    http::VcxHttpClient,
    storage::{object_cache::ObjectCache, Storage},
    AgentError, AgentErrorKind, AgentResult,
};

pub struct ServiceDidExchange {
    ledger_read: Arc<DefaultIndyLedgerRead>,
    wallet: Arc<IndySdkWallet>,
    resolver_registry: Arc<ResolverRegistry>,
    service_endpoint: ServiceEndpoint,
    did_exchange: Arc<ObjectCache<GenericDidExchange>>,
    public_did: String,
}

impl ServiceDidExchange {
    pub fn new(
        ledger_read: Arc<DefaultIndyLedgerRead>,
        wallet: Arc<IndySdkWallet>,
        resolver_registry: Arc<ResolverRegistry>,
        service_endpoint: ServiceEndpoint,
        public_did: String,
    ) -> Self {
        Self {
            ledger_read,
            wallet,
            service_endpoint,
            resolver_registry,
            did_exchange: Arc::new(ObjectCache::new("did-exchange")),
            public_did,
        }
    }

    pub async fn send_request_public(&self, their_did: String) -> AgentResult<String> {
        let (requester, request) = GenericDidExchange::construct_request_public(
            self.ledger_read.as_ref(),
            format!("did:sov:{}", their_did).parse()?,
            format!("did:sov:{}", self.public_did).parse()?,
        )
        .await?;
        let request_id = request
            .clone()
            .decorators
            .thread
            .ok_or_else(|| {
                AgentError::from_msg(
                    AgentErrorKind::InvalidState,
                    "Request did not contain a thread id",
                )
            })?
            .thid;
        let ddo_their = requester.their_did_doc();
        let ddo_our = requester.our_did_document();
        let encryption_envelope =
            pairwise_encrypt(ddo_our, ddo_their, self.wallet.as_ref(), &request.into()).await?;
        VcxHttpClient
            .send_message(encryption_envelope.0, get_their_endpoint(ddo_their)?)
            .await?;
        self.did_exchange.insert(&request_id, requester.clone())
    }

    pub async fn send_response(
        &self,
        request: Request,
        invitation: OobInvitation,
    ) -> AgentResult<String> {
        // TODO: We should fetch the out of band invite associated with the request.
        // We don't want to be sending response if we don't know if there is any invitation
        // associated with the request.
        let request_id = request
            .clone()
            .decorators
            .thread
            .ok_or_else(|| {
                AgentError::from_msg(
                    AgentErrorKind::InvalidState,
                    "Request did not contain a thread id",
                )
            })?
            .thid;
        let invitation_key =
            resolve_key_from_invitation(&invitation, &self.resolver_registry).await?;
        let (responder, response) = GenericDidExchange::handle_request(
            self.wallet.as_ref(),
            self.resolver_registry.clone(),
            request,
            self.service_endpoint.clone(),
            vec![],
            invitation.id.clone(),
            invitation_key,
        )
        .await?;
        let ddo_their = responder.their_did_doc();
        let ddo_our = responder.our_did_document();
        let encryption_envelope =
            pairwise_encrypt(ddo_our, ddo_their, self.wallet.as_ref(), &response.into()).await?;
        VcxHttpClient
            .send_message(encryption_envelope.0, get_their_endpoint(ddo_their)?)
            .await?;
        self.did_exchange.insert(&request_id, responder.clone())
    }

    pub async fn send_complete(&self, response: Response) -> AgentResult<String> {
        let thread_id = response.decorators.thread.thid.clone();
        let (requester, complete) = self
            .did_exchange
            .get(&thread_id)?
            .handle_response(response)
            .await?;
        let ddo_their = requester.their_did_doc();
        let ddo_our = requester.our_did_document();
        let encryption_envelope =
            pairwise_encrypt(ddo_our, ddo_their, self.wallet.as_ref(), &complete.into()).await?;
        VcxHttpClient
            .send_message(encryption_envelope.0, get_their_endpoint(ddo_their)?)
            .await?;
        self.did_exchange.insert(&thread_id, requester.clone())
    }

    pub fn receive_complete(&self, complete: Complete) -> AgentResult<String> {
        let thread_id = complete.decorators.thread.thid.clone();
        let requester = self
            .did_exchange
            .get(&thread_id)?
            .handle_complete(complete)?;
        self.did_exchange.insert(&thread_id, requester)
    }

    pub fn receive_problem_report(&self, problem_report: ProblemReport) -> AgentResult<String> {
        let thread_id = problem_report.decorators.thread.thid.clone();
        let requester = self
            .did_exchange
            .get(&thread_id)?
            .handle_problem_report(problem_report)?;
        self.did_exchange.insert(&thread_id, requester)
    }

    pub fn exists_by_id(&self, thread_id: &str) -> bool {
        self.did_exchange.contains_key(thread_id)
    }

    pub fn invitation_id(&self, thread_id: &str) -> AgentResult<String> {
        Ok(self
            .did_exchange
            .get(thread_id)?
            .invitation_id()
            .to_string())
    }

    pub fn public_did(&self) -> &str {
        self.public_did.as_ref()
    }

    pub fn get_state(&self, thread_id: &str) -> AgentResult<ThinState> {
        Ok(self.did_exchange.get(thread_id)?.get_state())
    }
}
