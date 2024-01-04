use std::sync::Arc;

use did_parser::Did;
use did_peer::{
    peer_did::{numalgos::numalgo2::Numalgo2, PeerDid},
    resolver::options::PublicKeyEncoding,
};
use did_resolver_registry::ResolverRegistry;
use messages::msg_fields::protocols::did_exchange::{
    complete::Complete as CompleteMessage, request::Request, response::Response,
};
use uuid::Uuid;

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
        their_did: &Did,
        our_peer_did: &PeerDid<Numalgo2>,
    ) -> Result<TransitionResult<Self, Request>, AriesVcxError> {
        let their_did_document = resolver_registry
            .resolve(their_did, &Default::default())
            .await?
            .did_document()
            .clone();
        let our_did_document = our_peer_did.to_did_doc(PublicKeyEncoding::Base58)?;
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
            let peer_did = PeerDid::<Numalgo2>::parse(response.content.did)
                .map_err(to_transition_error(self.clone()))?;
            peer_did
                .to_did_doc(PublicKeyEncoding::Base58)
                .map_err(to_transition_error(self.clone()))?
        };

        let complete_message = construct_didexchange_complete(
            self.state.request_id.clone(),
            self.state.invitation_id.clone(),
        );

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
