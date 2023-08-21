pub mod config;
mod helpers;

use did_parser::ParseError;
use did_peer::peer_did_resolver::resolver::PeerDidResolver;
use did_resolver::{error::GenericError, traits::resolvable::DidResolvable};
use messages::msg_fields::protocols::did_exchange::{
    complete::Complete as CompleteMessage, request::Request, response::Response,
};
use public_key::{Key, KeyType};

use crate::{
    common::{
        keys::get_verkey_from_ledger,
        ledger::transactions::{into_did_doc, resolve_oob_invitation},
    },
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    handlers::util::AnyInvitation,
    protocols::did_exchange::{
        state_machine::helpers::{attach_to_ddo_sov, create_our_did_document, ddo_sov_to_attach, jws_sign_attach},
        states::{completed::Completed, requester::request_sent::RequestSent},
        transition::{transition_error::TransitionError, transition_result::TransitionResult},
    },
    utils::from_legacy_did_doc_to_sov,
};

use helpers::{construct_complete_message, construct_request, did_doc_from_did, verify_handshake_protocol};

use self::config::{ConstructRequestConfig, PairwiseConstructRequestConfig, PublicConstructRequestConfig};

use super::DidExchangeRequester;

impl DidExchangeRequester<RequestSent> {
    async fn construct_request_pairwise(
        PairwiseConstructRequestConfig {
            ledger,
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
        let request = construct_request(
            invitation.id.clone(),
            our_did_document.id().to_string(),
            Some(signed_attach),
        )?;

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

        let key = Key::new(
            our_did_document
                .verification_method()
                .first()
                .unwrap()
                .public_key()
                .key_decoded()?,
            KeyType::Ed25519,
        )?;
        let signed_attach = jws_sign_attach(ddo_sov_to_attach(our_did_document.clone())?, key, &wallet).await?;
        let request = construct_request(invitation_id.clone(), our_did.to_string(), Some(signed_attach))?;

        Ok(TransitionResult {
            state: DidExchangeRequester::from_parts(
                RequestSent {
                    request_id: request.id.clone(),
                    invitation_id,
                },
                their_did_document,
                our_did_document, // Key::from_base58(
                                  //     &wallet.key_for_local_did(&our_did.id().to_string()).await?,
                                  //     KeyType::X25519,
                                  // )?
                                  // .clone(),
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
                did_document,
                self.our_did_document,
            ),
            output: complete_message,
        })
    }
}
