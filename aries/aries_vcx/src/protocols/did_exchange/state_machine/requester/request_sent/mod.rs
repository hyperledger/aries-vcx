use std::sync::Arc;

use chrono::Utc;
use did_parser::Did;
use did_peer::{
    peer_did::{
        numalgos::numalgo2::{resolve::resolve_numalgo2, Numalgo2},
        PeerDid,
    },
    resolver::{options::PublicKeyEncoding, PeerDidResolver},
};
use did_resolver::traits::resolvable::DidResolvable;
use did_resolver_registry::ResolverRegistry;
use messages::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::did_exchange::{
        complete::{Complete as CompleteMessage, Complete, CompleteDecorators},
        request::Request,
        response::Response,
    },
};
use uuid::Uuid;

use super::DidExchangeRequester;
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    protocols::did_exchange::{
        state_machine::{
            helpers::{attachment_to_diddoc, to_transition_error},
            requester::helpers::construct_request,
        },
        states::{completed::Completed, requester::request_sent::RequestSent},
        transition::{transition_error::TransitionError, transition_result::TransitionResult},
    },
};

impl DidExchangeRequester<RequestSent> {
    pub async fn construct_request(
        resolver_registry: Arc<ResolverRegistry>,
        their_did: &Did,
        our_peer_did: &PeerDid<Numalgo2>,
    ) -> Result<TransitionResult<Self, Request>, AriesVcxError> {
        let their_did_document = resolver_registry
            .resolve(their_did, &Default::default())
            .await?
            .did_document()
            .clone();
        let our_did_document = resolve_numalgo2(our_peer_did, PublicKeyEncoding::Base58)?.build();
        let invitation_id = Uuid::new_v4().to_string();

        let request = construct_request(invitation_id.clone(), our_peer_did.to_string());

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
            attachment_to_diddoc(ddo).map_err(to_transition_error(self.clone()))?
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
