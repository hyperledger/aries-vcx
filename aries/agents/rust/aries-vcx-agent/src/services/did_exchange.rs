use std::sync::Arc;

use aries_vcx::{
    messages::{
        msg_fields::protocols::{
            did_exchange::{
                complete::Complete, problem_report::ProblemReport, request::Request,
                response::Response,
            },
            out_of_band::invitation::Invitation as OobInvitation,
        },
        AriesMessage,
    },
    protocols::did_exchange::{
        resolve_enc_key_from_invitation,
        state_machine::{
            create_our_did_document,
            generic::{GenericDidExchange, ThinState},
        },
    },
    transport::Transport,
    utils::encryption_envelope::EncryptionEnvelope,
};
use aries_vcx_core::wallet::{base_wallet::BaseWallet, indy::IndySdkWallet};
use did_peer::peer_did::{numalgos::numalgo2::Numalgo2, PeerDid};
use did_resolver_registry::ResolverRegistry;
use did_resolver_sov::did_resolver::did_doc::schema::did_doc::DidDocument;
use url::Url;

use super::connection::ServiceEndpoint;
use crate::{
    http::VcxHttpClient,
    storage::{object_cache::ObjectCache, Storage},
    AgentError, AgentErrorKind, AgentResult,
};

pub struct ServiceDidExchange {
    wallet: Arc<IndySdkWallet>,
    resolver_registry: Arc<ResolverRegistry>,
    service_endpoint: ServiceEndpoint,
    did_exchange: Arc<ObjectCache<GenericDidExchange>>,
    public_did: String,
}

impl ServiceDidExchange {
    pub fn new(
        wallet: Arc<IndySdkWallet>,
        resolver_registry: Arc<ResolverRegistry>,
        service_endpoint: ServiceEndpoint,
        public_did: String,
    ) -> Self {
        Self {
            wallet,
            service_endpoint,
            resolver_registry,
            did_exchange: Arc::new(ObjectCache::new("did-exchange")),
            public_did,
        }
    }

    pub async fn send_request(&self, their_did: String) -> AgentResult<String> {
        let (our_did_document, _our_verkey) =
            create_our_did_document(self.wallet.as_ref(), self.service_endpoint.clone(), vec![])
                .await?;

        let our_peer_did = PeerDid::<Numalgo2>::from_did_doc(our_did_document.clone())?;
        let (requester, request) = GenericDidExchange::construct_request(
            self.resolver_registry.clone(),
            &format!("did:sov:{}", their_did).parse()?,
            &our_peer_did,
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
            .send_message(encryption_envelope.0, get_first_endpoint(ddo_their)?)
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
            resolve_enc_key_from_invitation(&invitation, &self.resolver_registry).await?;
        let (our_did_document, _our_verkey) =
            create_our_did_document(self.wallet.as_ref(), self.service_endpoint.clone(), vec![])
                .await?;
        let peer_did_invitee = PeerDid::<Numalgo2>::from_did_doc(our_did_document.clone())?;

        let (responder, response) = GenericDidExchange::handle_request(
            self.wallet.as_ref(),
            self.resolver_registry.clone(),
            request,
            &peer_did_invitee,
            invitation_key,
        )
        .await?;
        let ddo_their = responder.their_did_doc();
        let ddo_our = responder.our_did_document();
        let encryption_envelope =
            pairwise_encrypt(ddo_our, ddo_their, self.wallet.as_ref(), &response.into()).await?;
        VcxHttpClient
            .send_message(encryption_envelope.0, get_first_endpoint(ddo_their)?)
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
            .send_message(encryption_envelope.0, get_first_endpoint(ddo_their)?)
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

    pub fn invitation_id(&self, _thread_id: &str) -> AgentResult<String> {
        unimplemented!()
    }

    pub fn public_did(&self) -> &str {
        self.public_did.as_ref()
    }

    pub fn get_state(&self, thread_id: &str) -> AgentResult<ThinState> {
        Ok(self.did_exchange.get(thread_id)?.get_state())
    }
}

pub fn get_first_endpoint(did_document: &DidDocument) -> AgentResult<Url> {
    let service = did_document.service().first().ok_or(AgentError::from_msg(
        AgentErrorKind::InvalidState,
        "No service found",
    ))?;
    Ok(service.service_endpoint().clone())
}

pub async fn pairwise_encrypt(
    our_did_doc: &DidDocument,
    their_did_doc: &DidDocument,
    wallet: &impl BaseWallet,
    message: &AriesMessage,
) -> AgentResult<EncryptionEnvelope> {
    let service = our_did_doc
        .service()
        .first()
        .ok_or_else(|| {
            AgentError::from_msg(
                AgentErrorKind::InvalidState,
                "No Service object found on our did document",
            )
        })?
        .clone();
    // todo: hacky, assuming we have full base58 key inlined which we possibly don't
    //       The recipient key might have to be dereferenced
    let sender_vk = service
        .extra_field_recipient_keys()
        .map_err(|err| {
            AgentError::from_msg(
                AgentErrorKind::InvalidState,
                &format!(
                    "Recipient key field found in our did document but had unexpected format, \
                     err: {err:?}"
                ),
            )
        })?
        .first()
        .ok_or_else(|| {
            AgentError::from_msg(
                AgentErrorKind::InvalidState,
                "Recipient key field but did not have any keys",
            )
        })?
        .clone();

    let service = their_did_doc
        .service()
        .first()
        .ok_or_else(|| AgentError::from_msg(AgentErrorKind::InvalidState, "No service found"))?;
    // todo: hacky, assuming we have full base58 key inlined which we probably don't
    let recipient_key = service
        .extra_field_recipient_keys()
        .map_err(|err| {
            AgentError::from_msg(
                AgentErrorKind::InvalidState,
                &format!("No recipient_keys found: {}", err),
            )
        })?
        .first()
        .ok_or_else(|| {
            AgentError::from_msg(AgentErrorKind::InvalidState, "No recipient_keys found")
        })?
        .clone();

    // todo: again, not considering possibility of having didurl as value, assuming inlined key
    let routing_keys = service.extra_field_routing_keys().map_err(|err| {
        AgentError::from_msg(
            AgentErrorKind::InvalidState,
            &format!("No routing_keys found: {}", err),
        )
    })?;

    EncryptionEnvelope::create2(
        wallet,
        serde_json::json!(message).to_string().as_bytes(),
        Some(&sender_vk.to_string()),
        recipient_key.to_string(),
        routing_keys.iter().map(|k| k.to_string()).collect(),
    )
    .await
    .map_err(|err| err.into())
}
