use std::sync::Arc;

use aries_vcx_wallet::wallet::base_wallet::BaseWallet;
use base64::Engine;
use did_doc::schema::did_doc::DidDocument;
use did_parser_nom::Did;
use did_peer::peer_did::{numalgos::numalgo4::Numalgo4, PeerDid};
use did_resolver::traits::resolvable::resolution_output::DidResolutionOutput;
use did_resolver_registry::ResolverRegistry;
use messages::{
    decorators::attachment::AttachmentType,
    msg_fields::protocols::did_exchange::{
        v1_1::{request::Request, response::Response},
        v1_x::{complete::Complete, response::AnyResponse},
    },
    msg_types::protocols::did_exchange::DidExchangeTypeV1,
};
use public_key::Key;

use super::DidExchangeRequester;
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    protocols::did_exchange::{
        state_machine::{
            helpers::{attachment_to_diddoc, jws_verify_attachment, to_transition_error},
            requester::helpers::{construct_didexchange_complete, construct_request},
        },
        states::{completed::Completed, requester::request_sent::RequestSent},
        transition::{transition_error::TransitionError, transition_result::TransitionResult},
    },
    utils::base64::URL_SAFE_LENIENT,
};

impl DidExchangeRequester<RequestSent> {
    pub async fn construct_request(
        resolver_registry: Arc<ResolverRegistry>,
        invitation_id: Option<String>,
        their_did: &Did,
        our_peer_did: &PeerDid<Numalgo4>,
        our_label: String,
    ) -> Result<TransitionResult<Self, Request>, AriesVcxError> {
        debug!(
            "DidExchangeRequester<RequestSent>::construct_request >> their_did: {}, our_peer_did: \
             {}",
            their_did, our_peer_did
        );
        let their_did_document = resolver_registry
            .resolve(their_did, &Default::default())
            .await?
            .did_document;
        let our_did_document = our_peer_did.resolve_did_doc()?;
        let request = construct_request(
            invitation_id.clone(),
            our_peer_did.to_string(),
            our_label,
            DidExchangeTypeV1::new_v1_1(),
        );

        debug!(
            "DidExchangeRequester<RequestSent>::construct_request << prepared request: {}",
            request
        );
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
        wallet: &impl BaseWallet,
        invitation_key: &Key,
        response: AnyResponse,
        resolver_registry: &Arc<ResolverRegistry>,
    ) -> Result<TransitionResult<DidExchangeRequester<Completed>, Complete>, TransitionError<Self>>
    {
        debug!(
            "DidExchangeRequester<RequestSent>::receive_response >> response: {:?}",
            response
        );
        let version = response.get_version();
        let response = response.into_v1_1();

        if response.decorators.thread.thid != self.state.request_id {
            return Err(TransitionError {
                error: AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Response thread ID does not match request ID",
                ),
                state: self,
            });
        }

        let did_document = extract_and_verify_responder_did_doc(
            wallet,
            invitation_key,
            response,
            resolver_registry,
        )
        .await
        .map_err(to_transition_error(self.clone()))?;

        let complete_message = construct_didexchange_complete(
            self.state.invitation_id,
            self.state.request_id.clone(),
            version,
        );
        debug!(
            "DidExchangeRequester<RequestSent>::receive_response << complete_message: {:?}",
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

async fn extract_and_verify_responder_did_doc(
    wallet: &impl BaseWallet,
    invitation_key: &Key,
    response: Response,
    resolver_registry: &Arc<ResolverRegistry>,
) -> Result<DidDocument, AriesVcxError> {
    let their_did = response.content.did;

    if let Some(did_doc_attach) = response.content.did_doc {
        let verified_signature =
            jws_verify_attachment(&did_doc_attach, invitation_key, wallet).await?;
        if !verified_signature {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidInput,
                "DIDExchange response did not have a valid DIDDoc signature from the expected \
                 inviter",
            ));
        }

        let did_doc = attachment_to_diddoc(did_doc_attach)?;
        if did_doc.id().to_string() != their_did {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidInput,
                "DIDExchange response had a DIDDoc which did not match the response DID",
            ));
        }
        return Ok(did_doc);
    }

    if let Some(did_rotate_attach) = response.content.did_rotate {
        let verified_signature =
            jws_verify_attachment(&did_rotate_attach, invitation_key, wallet).await?;
        if !verified_signature {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidInput,
                "DIDExchange response did not have a valid DID rotate signature from the expected \
                 inviter",
            ));
        }

        let AttachmentType::Base64(signed_did_b64) = did_rotate_attach.data.content else {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::EncodeError,
                "DIDExchange response did not have a valid DID rotate attachment",
            ));
        };

        let did_bytes = URL_SAFE_LENIENT.decode(signed_did_b64).map_err(|_| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::EncodeError,
                "DIDExchange response did not have a valid base64 did rotate attachment",
            )
        })?;
        let signed_did = String::from_utf8(did_bytes).map_err(|_| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::EncodeError,
                "DIDExchange response did not have a valid UTF8 did rotate attachment",
            )
        })?;

        if signed_did != their_did {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidInput,
                format!(
                    "DIDExchange response had a DID rotate which did not match the response DID. \
                     Wanted {their_did}, found {signed_did}"
                ),
            ));
        }

        let did = &Did::parse(their_did)?;
        let DidResolutionOutput {
            did_document: did_doc,
            ..
        } = resolver_registry.resolve(did, &Default::default()).await?;
        return Ok(did_doc);
    }

    // default to error
    Err(AriesVcxError::from_msg(
        AriesVcxErrorKind::InvalidInput,
        "DIDExchange response could not be verified. No DIDDoc nor DIDRotate was attached.",
    ))
}
