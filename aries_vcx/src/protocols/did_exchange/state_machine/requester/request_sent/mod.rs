use std::sync::Arc;

use aries_vcx_core::{ledger::base_ledger::IndyLedgerRead, wallet::base_wallet::BaseWallet};
use chrono::Utc;
use did_parser::Did;
use did_peer::resolver::PeerDidResolver;
use did_resolver::traits::resolvable::DidResolvable;
use did_resolver_registry::ResolverRegistry;
use helpers::{construct_request, did_doc_from_did, verify_handshake_protocol};
use messages::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::{
        did_exchange::{
            complete::{Complete as CompleteMessage, Complete, CompleteDecorators},
            request::Request,
            response::Response,
        },
        out_of_band::invitation::Invitation,
    },
};
use url::Url;
use uuid::Uuid;

use super::DidExchangeRequester;
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    handlers::out_of_band::receiver::oob_invitation_to_diddoc,
    protocols::did_exchange::{
        state_machine::helpers::{attach_to_ddo_sov, create_our_did_document, to_transition_error},
        states::{completed::Completed, requester::request_sent::RequestSent},
        transition::{transition_error::TransitionError, transition_result::TransitionResult},
    },
};

mod helpers;

impl DidExchangeRequester<RequestSent> {
    pub async fn construct_request_pairwise(
        wallet: &impl BaseWallet,
        invitation: Invitation,
        resolver_registry: Arc<ResolverRegistry>,
        service_endpoint: Url,
        routing_keys: Vec<String>,
    ) -> Result<TransitionResult<Self, Request>, AriesVcxError> {
        verify_handshake_protocol(invitation.clone())?;
        let (our_did_document, _our_verkey) =
            create_our_did_document(wallet, service_endpoint, routing_keys).await?;
        let their_did_document =
            oob_invitation_to_diddoc(&resolver_registry, invitation.clone()).await?;

        let request = construct_request(invitation.id.clone(), our_did_document.id().to_string());

        Ok(TransitionResult {
            state: DidExchangeRequester::from_parts(
                RequestSent {
                    invitation_id: invitation.id.clone(),
                    request_id: request.id.clone(),
                },
                their_did_document,
                our_did_document,
            ),
            output: request,
        })
    }

    pub async fn construct_request_public(
        ledger: &impl IndyLedgerRead,
        their_did: Did,
        our_did: Did,
    ) -> Result<TransitionResult<Self, Request>, AriesVcxError> {
        let (their_did_document, service) = did_doc_from_did(ledger, their_did.clone()).await?;
        let (our_did_document, _) = did_doc_from_did(ledger, our_did.clone()).await?;
        let invitation_id = format!("{}#{}", their_did, service.id());

        let request = construct_request(invitation_id.clone(), our_did.to_string());

        Ok(TransitionResult {
            state: DidExchangeRequester::from_parts(
                RequestSent {
                    request_id: request.id.clone(),
                    invitation_id,
                },
                their_did_document,
                our_did_document,
            ),
            output: request,
        })
    }

    pub async fn receive_response(
        self,
        response: Response,
    ) -> Result<
        TransitionResult<DidExchangeRequester<Completed>, CompleteMessage>,
        TransitionError<Self>,
    > {
        if response.decorators.thread.thid != self.state.request_id {
            return Err(TransitionError {
                error: AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Response thread ID does not match request ID",
                ),
                state: self,
            });
        }
        let did_document = if let Some(ddo) = response.content.did_doc {
            attach_to_ddo_sov(ddo).map_err(to_transition_error(self.clone()))?
        } else {
            PeerDidResolver::new()
                .resolve(
                    &response
                        .content
                        .did
                        .parse()
                        .map_err(to_transition_error(self.clone()))?,
                    &Default::default(),
                )
                .await
                .map_err(to_transition_error(self.clone()))?
                .did_document()
                .to_owned()
                .into()
        };
        let decorators = CompleteDecorators::builder()
            .thread(
                Thread::builder()
                    .thid(self.state.request_id.clone())
                    .pthid(self.state.invitation_id.clone())
                    .build(),
            )
            .timing(Timing::builder().out_time(Utc::now()).build())
            .build();
        let complete_message = Complete::builder()
            .id(Uuid::new_v4().to_string())
            .decorators(decorators)
            .build();

        Ok(TransitionResult {
            state: DidExchangeRequester::from_parts(
                Completed {
                    invitation_id: self.state.invitation_id,
                    request_id: self.state.request_id,
                },
                // TODO: Make sure to make the DDO identifier did:peer:3 for both
                did_document,
                self.our_did_document,
            ),
            output: complete_message,
        })
    }
}
