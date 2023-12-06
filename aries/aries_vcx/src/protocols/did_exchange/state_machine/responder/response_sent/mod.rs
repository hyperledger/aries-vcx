use std::sync::Arc;

use aries_vcx_core::wallet::base_wallet::BaseWallet;
use did_doc::schema::did_doc::DidDocument;
use did_peer::{
    peer_did::{
        numalgos::numalgo2::{resolve::resolve_numalgo2, Numalgo2},
        PeerDid,
    },
    resolver::options::PublicKeyEncoding,
};
use did_resolver_registry::ResolverRegistry;
use messages::msg_fields::protocols::did_exchange::{
    complete::Complete, request::Request, response::Response,
};
use public_key::Key;

use super::DidExchangeResponder;
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    protocols::did_exchange::{
        state_machine::helpers::{
            attachment_to_diddoc, construct_response, ddo_to_attach, jws_sign_attach,
        },
        states::{completed::Completed, responder::response_sent::ResponseSent},
        transition::{transition_error::TransitionError, transition_result::TransitionResult},
    },
};

impl DidExchangeResponder<ResponseSent> {
    pub async fn receive_request(
        wallet: &impl BaseWallet,
        resolver_registry: Arc<ResolverRegistry>,
        request: Request,
        our_peer_did: &PeerDid<Numalgo2>,
        invitation_key: Key,
    ) -> Result<TransitionResult<DidExchangeResponder<ResponseSent>, Response>, AriesVcxError> {
        let their_ddo = resolve_their_ddo(&resolver_registry, &request).await?;
        let our_did_document = resolve_numalgo2(our_peer_did, PublicKeyEncoding::Base58)?.build();

        // TODO: Response should sign the new *did* with invitation_key only if key was rotated
        //       In practice if the invitation was public, we definitely will be rotating to
        //       peer:did if the invitation was peer2peer, we probably keep our original invitation
        // keys       The outstanding question is how, and on what level, we gonna do this
        // detection TODO: Check amendment made to did-exchange protocol in terms of
        // rotating keys. When keys       are rotated, there's a new decorator which conveys
        // that
        let signed_attach = jws_sign_attach(
            ddo_to_attach(our_did_document.clone())?,
            invitation_key,
            wallet,
        )
        .await?;

        let response = construct_response(request.id.clone(), &our_did_document, signed_attach);

        Ok(TransitionResult {
            state: DidExchangeResponder::from_parts(
                ResponseSent {
                    request_id: request.id,
                },
                their_ddo,
                our_did_document,
            ),
            output: response,
        })
    }

    pub fn receive_complete(
        self,
        complete: Complete,
    ) -> Result<DidExchangeResponder<Completed>, TransitionError<Self>> {
        if complete.decorators.thread.thid != self.state.request_id {
            return Err(TransitionError {
                error: AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Thread ID of the complete message does not match the id of the request",
                ),
                state: self,
            });
        }
        Ok(DidExchangeResponder::from_parts(
            Completed {
                request_id: self.state.request_id,
            },
            self.their_did_document,
            self.our_did_document,
        ))
    }
}

async fn resolve_their_ddo(
    resolver_registry: &Arc<ResolverRegistry>,
    request: &Request,
) -> Result<DidDocument, AriesVcxError> {
    Ok(request
        .content
        .did_doc
        .clone()
        .map(attachment_to_diddoc)
        .transpose()?
        .unwrap_or(
            resolver_registry
                .resolve(&request.content.did.parse()?, &Default::default())
                .await?
                .did_document()
                .to_owned(),
        ))
}
