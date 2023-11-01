use std::sync::Arc;

use aries_vcx_core::wallet::base_wallet::BaseWallet;
use chrono::Utc;
use did_resolver_registry::ResolverRegistry;
use messages::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::did_exchange::{
        complete::Complete,
        request::Request,
        response::{Response, ResponseContent, ResponseDecorators},
    },
};
use public_key::Key;
use url::Url;
use uuid::Uuid;
use did_doc::schema::did_doc::DidDocument;

use super::DidExchangeResponder;
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    protocols::did_exchange::{
        state_machine::helpers::{
            attach_to_ddo_sov, create_our_did_document, ddo_to_attach, jws_sign_attach,
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
        service_endpoint: Url,
        routing_keys: Vec<String>,
        invitation_id: String,
        invitation_key: Key,
    ) -> Result<TransitionResult<DidExchangeResponder<ResponseSent>, Response>, AriesVcxError> {
        let their_ddo = resolve_their_ddo(&resolver_registry, &request).await?;
        let (our_did_document, _enc_key) =
            create_our_did_document(wallet, service_endpoint, routing_keys).await?;

        if request.decorators.thread.and_then(|t| t.pthid) != Some(invitation_id.clone()) {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Parent thread ID of the request does not match the id of the invite",
            ));
        }

        // TODO: Response should sign the new *did* with invitation_key only if key was rotated
        let signed_attach = jws_sign_attach(
            ddo_to_attach(our_did_document.clone())?,
            invitation_key,
            wallet,
        )
        .await?;

        let content = ResponseContent::builder()
            .did(our_did_document.id().to_string())
            .did_doc(Some(signed_attach))
            .build();
        let decorators = ResponseDecorators::builder()
            .thread(
                Thread::builder()
                    .thid(request.id.clone())
                    .pthid(invitation_id.clone()) // todo: do we need to set this in Response?
                    .build(),
            )
            .timing(Timing::builder().out_time(Utc::now()).build())
            .build();
        let response = Response::builder()
            .id(Uuid::new_v4().to_string())
            .content(content)
            .decorators(decorators)
            .build();

        Ok(TransitionResult {
            state: DidExchangeResponder::from_parts(
                ResponseSent {
                    request_id: request.id,
                    invitation_id,
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
        if complete.decorators.thread.pthid != Some(self.state.invitation_id.to_string()) {
            return Err(TransitionError {
                error: AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Parent thread ID of the complete message does not match the id of the invite",
                ),
                state: self,
            });
        }
        Ok(DidExchangeResponder::from_parts(
            Completed {
                invitation_id: self.state.invitation_id,
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
        .map(attach_to_ddo_sov)
        .transpose()?
        .unwrap_or(
            resolver_registry
                .resolve(&request.content.did.parse()?, &Default::default())
                .await?
                .did_document()
                .to_owned()
                .into(),
        ))
}
