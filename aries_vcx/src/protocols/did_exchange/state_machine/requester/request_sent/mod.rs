pub mod config;
mod helpers;

use did_parser::ParseError;
use did_peer::peer_did_resolver::resolver::PeerDidResolver;
use did_resolver::{error::GenericError, traits::resolvable::DidResolvable};
use messages::msg_fields::protocols::did_exchange::{
    complete::Complete as CompleteMessage, request::Request, response::Response,
};

use crate::{
    common::ledger::transactions::resolve_oob_invitation,
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    protocols::did_exchange::{
        state_machine::helpers::{attach_to_ddo_sov, create_our_did_document, ddo_sov_to_attach, jws_sign_attach},
        states::{completed::Completed, requester::request_sent::RequestSent},
        transition::{transition_error::TransitionError, transition_result::TransitionResult},
    },
};

use helpers::{construct_complete_message, construct_request, did_doc_from_did, verify_handshake_protocol};

use self::config::{ConstructRequestConfig, PairwiseConstructRequestConfig, PublicConstructRequestConfig};

use super::DidExchangeRequester;

impl DidExchangeRequester<RequestSent> {
    async fn construct_request_pairwise(
        PairwiseConstructRequestConfig {
            wallet,
            service_endpoint,
            routing_keys,
            invitation,
            resolver_registry,
        }: PairwiseConstructRequestConfig,
    ) -> Result<TransitionResult<Self, Request>, AriesVcxError> {
        verify_handshake_protocol(invitation.clone())?;
        let (our_did_document, our_verkey) = create_our_did_document(&wallet, service_endpoint, routing_keys).await?;
        let their_did_document = resolve_oob_invitation(&resolver_registry, invitation.clone()).await?;

        let signed_attach = jws_sign_attach(
            ddo_sov_to_attach(our_did_document.clone())?,
            our_verkey.clone(),
            &wallet,
        )
        .await?;
        let request = construct_request(invitation.id.clone(), our_did_document.id().to_string())?;

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

    async fn construct_request_public(
        PublicConstructRequestConfig {
            wallet,
            ledger,
            their_did,
            our_did,
        }: PublicConstructRequestConfig,
    ) -> Result<TransitionResult<Self, Request>, AriesVcxError> {
        let (their_did_document, service) = did_doc_from_did(&ledger, their_did.clone()).await?;
        let (our_did_document, _) = did_doc_from_did(&ledger, our_did.clone()).await?;
        let invitation_id = format!("{}#{}", their_did, service.id().to_string());

        let key = our_did_document
            .verification_method()
            .first()
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "No verification method in requester's did document",
            ))?
            .public_key()?;
        let signed_attach = jws_sign_attach(ddo_sov_to_attach(our_did_document.clone())?, key, &wallet).await?;
        let request = construct_request(invitation_id.clone(), our_did.to_string())?;

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

    pub async fn construct_request(
        config: ConstructRequestConfig,
    ) -> Result<TransitionResult<Self, Request>, AriesVcxError> {
        match config {
            ConstructRequestConfig::Pairwise(config) => Self::construct_request_pairwise(config).await,
            ConstructRequestConfig::Public(config) => Self::construct_request_public(config).await,
        }
    }

    pub async fn receive_response(
        self,
        response: Response,
    ) -> Result<TransitionResult<DidExchangeRequester<Completed>, CompleteMessage>, TransitionError<Self>> {
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
            attach_to_ddo_sov(ddo).map_err(|error| TransitionError {
                error,
                state: self.clone(),
            })?
        } else {
            PeerDidResolver::new()
                .resolve(
                    &response
                        .content
                        .did
                        .parse()
                        .map_err(|error: ParseError| TransitionError {
                            error: error.into(),
                            state: self.clone(),
                        })?,
                    &Default::default(),
                )
                .await
                .map_err(|error: GenericError| TransitionError {
                    error: error.into(),
                    state: self.clone(),
                })?
                .did_document()
                .to_owned()
                .into()
        };
        let complete_message =
            construct_complete_message(self.state.invitation_id.clone(), self.state.request_id.clone());
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
