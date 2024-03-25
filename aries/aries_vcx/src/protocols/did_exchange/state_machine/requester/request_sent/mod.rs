use std::sync::Arc;

use did_parser_nom::Did;
use did_peer::peer_did::{numalgos::numalgo4::Numalgo4, PeerDid};
use did_resolver::traits::resolvable::resolution_output::DidResolutionOutput;
use did_resolver_registry::ResolverRegistry;
use messages::msg_fields::protocols::did_exchange::{
    complete::Complete as CompleteMessage, request::Request, response::Response,
};

use super::DidExchangeRequester;
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    protocols::did_exchange::{
        state_machine::{
            helpers::{attachment_to_diddoc, to_transition_error},
            requester::helpers::{construct_didexchange_complete, construct_request},
        },
        states::{completed::Completed, requester::request_sent::RequestSent},
        transition::{transition_error::TransitionError, transition_result::TransitionResult},
    },
};

impl DidExchangeRequester<RequestSent> {
    pub async fn construct_request(
        resolver_registry: Arc<ResolverRegistry>,
        invitation_id: Option<String>,
        their_did: &Did,
        our_peer_did: &PeerDid<Numalgo4>,
    ) -> Result<TransitionResult<Self, Request>, AriesVcxError> {
        info!(
            "DidExchangeRequester<RequestSent>::construct_request >> their_did: {}, our_peer_did: \
             {}",
            their_did, our_peer_did
        );
        let their_did_document = resolver_registry
            .resolve(their_did, &Default::default())
            .await?
            .did_document;
        let our_did_document = our_peer_did.resolve_did_doc()?;
        let request = construct_request(invitation_id.clone(), our_peer_did.to_string());

        info!(
            "DidExchangeRequester<RequestSent>::construct_request << prepared request: {}",
            request
        );
        Ok(TransitionResult {
            state: DidExchangeRequester::from_parts(
                RequestSent {
                    request_id: request.id.clone(),
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
        resolver_registry: Arc<ResolverRegistry>,
    ) -> Result<
        TransitionResult<DidExchangeRequester<Completed>, CompleteMessage>,
        TransitionError<Self>,
    > {
        info!(
            "DidExchangeRequester<RequestSent>::receive_response >> response: {:?}",
            response
        );
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
            info!(
                "DidExchangeRequester<RequestSent>::receive_response >> the Response message \
                 contained attached ddo"
            );
            attachment_to_diddoc(ddo).map_err(to_transition_error(self.clone()))?
        } else {
            info!(
                "DidExchangeRequester<RequestSent>::receive_response >> the Response message \
                 contains pairwise DID, resolving to DID Document"
            );
            let did =
                &Did::parse(response.content.did).map_err(to_transition_error(self.clone()))?;
            let DidResolutionOutput { did_document, .. } = resolver_registry
                .resolve(did, &Default::default())
                .await
                .map_err(to_transition_error(self.clone()))?;
            did_document
        };

        let complete_message = construct_didexchange_complete(self.state.request_id.clone());
        info!(
            "DidExchangeRequester<RequestSent>::receive_response << complete_message: {}",
            complete_message
        );

        Ok(TransitionResult {
            state: DidExchangeRequester::from_parts(
                Completed {
                    request_id: self.state.request_id,
                },
                did_document,
                self.our_did_document,
            ),
            output: complete_message,
        })
    }
}
