pub mod config;
mod helpers;

use did_doc_sov::DidDocumentSov;
use did_parser::ParseError;
use did_peer::peer_did_resolver::resolver::PeerDidResolver;
use did_resolver::{error::GenericError, traits::resolvable::DidResolvable};
use messages::msg_fields::protocols::did_exchange::{
    complete::Complete as CompleteMessage, request::Request, response::Response,
};
use public_key::{Key, KeyType};

use crate::{
    common::{keys::get_verkey_from_ledger, ledger::transactions::into_did_doc},
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    handlers::util::AnyInvitation,
    protocols::did_exchange::{
        service::helpers::{attach_to_ddo_sov, create_our_did_document, ddo_sov_to_attach, jws_sign_attach},
        states::{completed::Completed, requester::request_sent::RequestSent},
        transition::{transition_error::TransitionError, transition_result::TransitionResult},
    },
    utils::from_legacy_did_doc_to_sov,
};

use helpers::{construct_complete_message, construct_request, did_doc_from_did, verify_handshake_protocol};
use messages::msg_fields::protocols::did_exchange::complete::Complete;

use self::config::{ConstructRequestConfig, PairwiseConstructRequestConfig, PublicConstructRequestConfig};

use super::DidExchangeServiceRequester;

impl DidExchangeServiceRequester<RequestSent> {
    async fn construct_request_pairwise(
        PairwiseConstructRequestConfig {
            ledger,
            wallet,
            service_endpoint,
            routing_keys,
            invitation,
        }: PairwiseConstructRequestConfig,
    ) -> Result<TransitionResult<Self, Request>, AriesVcxError> {
        verify_handshake_protocol(invitation.clone())?;
        let (our_did_document, our_verkey) = create_our_did_document(&wallet, service_endpoint, routing_keys).await?;
        let their_did_document =
            from_legacy_did_doc_to_sov(into_did_doc(&ledger, &AnyInvitation::Oob(invitation.clone())).await?)?;

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
            state: DidExchangeServiceRequester::from_parts(
                RequestSent {
                    invitation_id: invitation.id.clone(),
                    request_id: request.id.clone(),
                },
                their_did_document,
                our_verkey,
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
            state: DidExchangeServiceRequester::from_parts(
                RequestSent {
                    request_id: request.id.clone(),
                    invitation_id,
                },
                their_did_document,
                // TODO: Get it from wallet instead
                Key::from_base58(
                    // &get_verkey_from_ledger(&ledger, &our_did.id().to_string()).await?,
                    &wallet.key_for_local_did(&our_did.id().to_string()).await?,
                    KeyType::X25519,
                )?
                .clone(),
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
    ) -> Result<TransitionResult<DidExchangeServiceRequester<Completed>, CompleteMessage>, TransitionError<Self>> {
        // todo: note, the processor could be perhaps injected from top, and then modified by state machine contextual data if needed
        let mut processor = ConnectionResponseProcessor::default();
        processor.add_preprocessor(MsgThidVerifier { expected_thid: self.state.request_id.clone() } );
        let data = processor.process(
            response,
            self.state.invitation_id.clone(),
            self.state.request_id.clone()
        ).await.unwrap();
        Ok(TransitionResult {
            state: DidExchangeServiceRequester::from_parts(
                Completed {
                    invitation_id: self.state.invitation_id,
                    request_id: self.state.request_id,
                },
                data.their_did_doc,
                self.our_verkey,
            ),
            output: data.message,
        })
    }
}


#[derive(Default)]
pub struct ConnectionResponseProcessor<V> where V: InputMsgVerifier {
    input_msg_verifiers: Vec<V>
}

struct ConnectionResponseProcessingOutput {
    message: Complete,
    their_did_doc: DidDocumentSov
}

impl <V> ConnectionResponseProcessor<V> where V: InputMsgVerifier {

    pub fn add_preprocessor(&mut self, input_verifier: V) {
        self.input_msg_verifiers.append(input_verifier);
    }

    pub fn input_msg_verification(&self, response: &Response) -> Result<(), String> {
        for verifier in &self.input_msg_verifiers {
            if !verifier.verify(response) {
                return Err("Input verification failed".to_string());
            }
        }
        Ok(())
    }

    // note:  these require invitation_id, request_id - if you don't use state machines do guide you
    //        it's up to you to remember data you need and inject them here correctly
    //        In particular, invitation)id, request_id are needed to correct build thid, pthid decorators
    //        which should be most likely responsibility of this function
    // note2: If anyone want's truly custom & curated behaviour, nothing stops them from using
    //        Message crate, and utils such as:
    //                                             attach_to_ddo_sov
    //                                             PeerDidResolver::new()
    //                                             construct_complete_message
    //        themselves.

    pub async fn process(
        &self,
        response: Response,
        invitation_id: String,
        request_id: String
    ) -> Result<ConnectionResponseProcessingOutput, String>  {
        self.input_msg_verification(&response)?;
        let did_document = if let Some(ddo) = response.content.did_doc {
            attach_to_ddo_sov(ddo).map_err(|error| Err("attachment handling err"))?
        } else {
            PeerDidResolver::new()
                .resolve(
                    &response
                        .content
                        .did
                        .parse()
                        .map_err(|error: ParseError| Err("parsing error"))?,
                    &Default::default(),
                )
                .await
                .map_err(|error: GenericError| Err("resolver error"))?
                .did_document()
                .to_owned()
                .into()
        };
        let complete_message =
            construct_complete_message(invitation_id.clone(), request_id.clone());
        Ok(ConnectionResponseProcessingOutput {
            message: complete_message,
            their_did_doc: did_document,

        })
    }
}

trait InputMsgVerifier {
    // todo: input_msg can be typed as decorator of an arbitrary aries message
    fn verify(&self, input_msg: &Response) -> bool;
}

struct MsgThidVerifier {
    expected_thid: String
}

impl InputMsgVerifier for MsgThidVerifier {
    fn verify(&self, input_msg: &Response) -> bool {
        input_msg.decorators.thread.thid == self.expected_thid
    }
}
