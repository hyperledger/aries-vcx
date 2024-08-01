use std::sync::Arc;

use aries_vcx_wallet::wallet::base_wallet::BaseWallet;
use did_doc::schema::did_doc::DidDocument;
use did_peer::peer_did::{numalgos::numalgo4::Numalgo4, PeerDid};
use did_resolver_registry::ResolverRegistry;
use messages::{
    msg_fields::protocols::did_exchange::v1_x::{
        complete::Complete,
        request::{AnyRequest, Request},
        response::AnyResponse,
    },
    msg_types::protocols::did_exchange::DidExchangeTypeV1,
};
use public_key::Key;

use super::DidExchangeResponder;
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    protocols::did_exchange::{
        state_machine::helpers::{
            assemble_did_rotate_attachment, attachment_to_diddoc, construct_response_v1_0,
            construct_response_v1_1, ddo_to_attach, jws_sign_attach,
        },
        states::{completed::Completed, responder::response_sent::ResponseSent},
        transition::{transition_error::TransitionError, transition_result::TransitionResult},
    },
};

impl DidExchangeResponder<ResponseSent> {
    pub async fn receive_request(
        wallet: &impl BaseWallet,
        resolver_registry: &Arc<ResolverRegistry>,
        request: AnyRequest,
        our_peer_did: &PeerDid<Numalgo4>,
        invitation_key: Key,
    ) -> Result<TransitionResult<DidExchangeResponder<ResponseSent>, AnyResponse>, AriesVcxError>
    {
        debug!(
            "DidExchangeResponder<ResponseSent>::receive_request >> request: {:?}, our_peer_did: \
             {}, invitation_key: {:?}",
            request, our_peer_did, invitation_key
        );
        let version = request.get_version();
        let request = request.into_inner();

        let their_ddo = resolve_ddo_from_request(resolver_registry, &request).await?;
        let our_did_document = our_peer_did.resolve_did_doc()?;

        let unsigned_attachment = match version {
            DidExchangeTypeV1::V1_1(_) => assemble_did_rotate_attachment(our_peer_did.did()),
            DidExchangeTypeV1::V1_0(_) => ddo_to_attach(our_did_document.clone())?,
        };
        let attachment = jws_sign_attach(unsigned_attachment, invitation_key, wallet).await?;

        let request_id = request.id.clone();
        let request_pthid = request.decorators.thread.and_then(|thid| thid.pthid);

        let response = match version {
            DidExchangeTypeV1::V1_1(_) => AnyResponse::V1_1(construct_response_v1_1(
                request_pthid,
                request_id,
                our_peer_did.did(),
                attachment,
            )),
            DidExchangeTypeV1::V1_0(_) => AnyResponse::V1_0(construct_response_v1_0(
                request_pthid,
                request_id,
                our_peer_did.did(),
                attachment,
            )),
        };
        debug!(
            "DidExchangeResponder<ResponseSent>::receive_request << prepared response: {:?}",
            response
        );

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

async fn resolve_ddo_from_request(
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
                .did_document,
        ))
}
