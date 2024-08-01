use std::sync::Arc;

use aries_vcx::{
    did_doc::schema::{service::typed::ServiceType, types::uri::Uri},
    did_parser_nom::Did,
    messages::{
        msg_fields::protocols::{
            did_exchange::v1_x::{
                complete::Complete, problem_report::ProblemReport, request::AnyRequest,
                response::AnyResponse,
            },
            out_of_band::invitation::Invitation as OobInvitation,
        },
        msg_types::protocols::did_exchange::DidExchangeTypeV1,
        AriesMessage,
    },
    protocols::did_exchange::{
        resolve_enc_key_from_did_doc, resolve_enc_key_from_invitation,
        state_machine::{
            generic::{GenericDidExchange, ThinState},
            helpers::create_peer_did_4,
        },
    },
    transport::Transport,
    utils::encryption_envelope::EncryptionEnvelope,
};
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;
use did_resolver_registry::ResolverRegistry;
use did_resolver_sov::did_resolver::did_doc::schema::did_doc::DidDocument;
use public_key::{Key, KeyType};
use url::Url;

use crate::{
    http::VcxHttpClient,
    storage::{agent_storage_inmem::AgentStorageInMem, AgentStorage},
    AgentError, AgentErrorKind, AgentResult,
};

// todo: break down into requester and responder services?
pub struct DidcommHandlerDidExchange<T> {
    wallet: Arc<T>,
    resolver_registry: Arc<ResolverRegistry>,
    service_endpoint: Url,
    did_exchange: Arc<AgentStorageInMem<(GenericDidExchange, Option<AriesMessage>)>>,
    public_did: String,
}

impl<T: BaseWallet> DidcommHandlerDidExchange<T> {
    pub fn new(
        wallet: Arc<T>,
        resolver_registry: Arc<ResolverRegistry>,
        service_endpoint: Url,
        public_did: String,
    ) -> Self {
        Self {
            wallet,
            service_endpoint,
            resolver_registry,
            did_exchange: Arc::new(AgentStorageInMem::new("did-exchange")),
            public_did,
        }
    }

    pub async fn handle_msg_invitation(
        &self,
        their_did: String,
        invitation_id: Option<String>,
        version: DidExchangeTypeV1,
    ) -> AgentResult<(String, Option<String>, String)> {
        // todo: type the return type
        let (our_peer_did, _our_verkey) =
            create_peer_did_4(self.wallet.as_ref(), self.service_endpoint.clone(), vec![]).await?;
        let our_did = our_peer_did.did().to_string();

        let their_did: Did = their_did.parse()?;
        let (requester, request) = GenericDidExchange::construct_request(
            &self.resolver_registry,
            invitation_id,
            &their_did,
            &our_peer_did,
            "".to_owned(),
            version,
        )
        .await?;

        // TODO: decouple this from AATH. The reason why we identify the requester's did-exchange
        // protocol with pthid is because that's what AATH expects when calling GET
        // /agent/command/did-exchange/{id} where {id} is actually {pthid}.
        // We should have internal strategy to manage threads ourselves, and build necessary
        // extensions/mappings/accommodations in AATH backchannel
        warn!("send_request >>> request: {:?}", request);
        let req_thread = request.inner().decorators.thread.as_ref().ok_or_else(|| {
            AgentError::from_msg(
                AgentErrorKind::InvalidState,
                "Request did not contain a thread",
            )
        })?;
        let pthid = req_thread.pthid.clone();
        // todo: messages must provide easier way to access this without all the shenanigans
        let thid = req_thread.thid.clone();

        let ddo_their = requester.their_did_doc();
        let ddo_our = requester.our_did_document();
        let service = ddo_their.get_service_of_type(&ServiceType::DIDCommV1)?;
        let encryption_envelope = pairwise_encrypt(
            ddo_our,
            ddo_their,
            self.wallet.as_ref(),
            &request.into(),
            service.id(),
        )
        .await?;
        // todo: hack; There's issue on AATH level https://github.com/hyperledger/aries-agent-test-harness/issues/784
        //       but if AATH can not be changed and both thid and pthid are used to track instance
        //       of protocol then we need to update storage to enable identification by
        //       multiple IDs (both thid, pthid (or arbitrary other))
        self.did_exchange.insert(&thid, (requester.clone(), None))?;
        VcxHttpClient
            .send_message(encryption_envelope.0, service.service_endpoint())
            .await?;
        Ok((thid, pthid, our_did))
    }

    // todo: whether invitation exists should handle the framework based on (p)thread matching
    //       rather than being supplied by upper layers
    pub async fn handle_msg_request(
        &self,
        request: AnyRequest,
        inviter_key: String,
        invitation: Option<OobInvitation>,
    ) -> AgentResult<(String, Option<String>, String, String)> {
        // todo: type the return type
        // Todo: messages should expose fallible API to get thid (for any aries msg). It's common
        //       pattern
        let thread = request.inner().decorators.thread.as_ref();

        let thid = thread
            .ok_or_else(|| {
                AgentError::from_msg(
                    AgentErrorKind::InvalidState,
                    "Request did not contain a thread id",
                )
            })?
            .thid
            .clone();

        let inviter_key = Key::from_base58(&inviter_key, KeyType::Ed25519)?;

        let invitation_key = match invitation {
            None => inviter_key,
            Some(invitation) => {
                resolve_enc_key_from_invitation(&invitation, &self.resolver_registry).await?
            }
        };

        let (our_peer_did, _our_verkey) =
            create_peer_did_4(self.wallet.as_ref(), self.service_endpoint.clone(), vec![]).await?;

        let pthid = thread
            .ok_or_else(|| {
                AgentError::from_msg(
                    AgentErrorKind::InvalidState,
                    "Request did not contain a thread",
                )
            })?
            .pthid
            .clone();

        let (responder, response) = GenericDidExchange::handle_request(
            self.wallet.as_ref(),
            &self.resolver_registry,
            request,
            &our_peer_did,
            invitation_key,
        )
        .await?;
        self.did_exchange
            .insert(&thid, (responder.clone(), Some(response.into())))?;

        let our_did = responder.our_did_document().id().to_string();
        let their_did = responder.their_did_doc().id().to_string();

        Ok((thid, pthid, our_did, their_did))
    }

    // todo: perhaps injectable transports? Or just return the message let the caller send it?
    //       The transports abstraction could understand https, wss, didcomm etc.
    pub async fn send_response(&self, thid: String) -> AgentResult<String> {
        info!("ServiceDidExchange::send_response >>> thid: {}", thid);
        let (responder, aries_msg) = self.did_exchange.get(&thid)?;
        let aries_msg: AriesMessage = aries_msg.unwrap();
        debug!(
            "ServiceDidExchange::send_response >>> successfully found state machine and a message \
             to be send"
        );

        let ddo_their = responder.their_did_doc();
        let ddo_our = responder.our_did_document();
        let service = ddo_their.get_service_of_type(&ServiceType::DIDCommV1)?;
        let encryption_envelope = pairwise_encrypt(
            ddo_our,
            ddo_their,
            self.wallet.as_ref(),
            &aries_msg,
            service.id(),
        )
        .await?;
        VcxHttpClient
            .send_message(encryption_envelope.0, service.service_endpoint())
            .await?;
        info!("ServiceDidExchange::send_response <<< successfully sent response");
        Ok(thid)
    }

    // todo: break down into "process_response" and "send_complete"
    pub async fn handle_msg_response(&self, response: AnyResponse) -> AgentResult<String> {
        let thread = match response {
            AnyResponse::V1_0(ref inner) => &inner.decorators.thread,
            AnyResponse::V1_1(ref inner) => &inner.decorators.thread,
        };
        let thid = thread.thid.clone();

        let (requester, _) = self.did_exchange.get(&thid)?;

        let inviter_ddo = requester.their_did_doc();
        let inviter_key = resolve_enc_key_from_did_doc(inviter_ddo)?;

        let (requester, complete) = requester
            .handle_response(
                self.wallet.as_ref(),
                &inviter_key,
                response,
                &self.resolver_registry,
            )
            .await?;
        let ddo_their = requester.their_did_doc();
        let ddo_our = requester.our_did_document();
        let service = ddo_their.get_service_of_type(&ServiceType::DIDCommV1)?;
        let encryption_envelope = pairwise_encrypt(
            ddo_our,
            ddo_their,
            self.wallet.as_ref(),
            &complete.into(),
            service.id(),
        )
        .await?;
        self.did_exchange.insert(&thid, (requester.clone(), None))?;
        VcxHttpClient
            .send_message(encryption_envelope.0, service.service_endpoint())
            .await?;
        Ok(thid)
    }

    pub fn handle_msg_complete(&self, complete: Complete) -> AgentResult<String> {
        let thread_id = complete.decorators.thread.thid.clone();
        let (requester, _) = self.did_exchange.get(&thread_id)?;
        let requester = requester.handle_complete(complete)?;
        self.did_exchange.insert(&thread_id, (requester, None))
    }

    pub fn receive_problem_report(&self, problem_report: ProblemReport) -> AgentResult<String> {
        let thread_id = problem_report.decorators.thread.thid.clone();
        let (requester, _) = self.did_exchange.get(&thread_id)?;
        let requester = requester.handle_problem_report(problem_report)?;
        self.did_exchange.insert(&thread_id, (requester, None))
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

    pub fn get_state(&self, thid: &str) -> AgentResult<ThinState> {
        let (protocol, _) = self.did_exchange.get(thid)?;
        Ok(protocol.get_state())
    }
}

pub async fn pairwise_encrypt(
    our_did_doc: &DidDocument,
    their_did_doc: &DidDocument,
    wallet: &impl BaseWallet,
    message: &AriesMessage,
    their_service_id: &Uri,
) -> AgentResult<EncryptionEnvelope> {
    EncryptionEnvelope::create(
        wallet,
        serde_json::json!(message).to_string().as_bytes(),
        our_did_doc,
        their_did_doc,
        their_service_id,
    )
    .await
    .map_err(|err| {
        AgentError::from_msg(
            AgentErrorKind::InvalidState,
            &format!("Failed to pairwise encrypt message due err: {}", err),
        )
    })
}
