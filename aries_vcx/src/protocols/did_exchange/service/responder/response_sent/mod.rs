use messages::msg_fields::protocols::did_exchange::{complete::Complete, response::Response};

use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    protocols::did_exchange::{
        service::helpers::create_our_did_document,
        states::{completed::Completed, responder::response_sent::ResponseSent},
        transition::{transition_error::TransitionError, transition_result::TransitionResult},
    },
};

use self::{
    config::ReceiveRequestConfig,
    helpers::{construct_response, resolve_their_ddo},
};

use super::DidExchangeServiceResponder;

pub mod config;
mod helpers;

impl DidExchangeServiceResponder<ResponseSent> {
    pub async fn receive_request(
        ReceiveRequestConfig {
            wallet,
            resolver_registry,
            request,
            service_endpoint,
            routing_keys,
            invitation_id,
        }: ReceiveRequestConfig,
    ) -> Result<TransitionResult<DidExchangeServiceResponder<ResponseSent>, Response>, AriesVcxError> {
        let their_ddo = resolve_their_ddo(&resolver_registry, &request).await?;
        let (our_ddo, enc_key) = create_our_did_document(&wallet, service_endpoint, routing_keys).await?;

        if request.decorators.thread.and_then(|t| t.pthid) != Some(invitation_id.clone()) {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Parent thread ID of the request does not match the id of the invite",
            ));
        }

        let response = construct_response(our_ddo, invitation_id.clone(), request.id.clone())?;

        Ok(TransitionResult {
            state: DidExchangeServiceResponder::from_parts(
                ResponseSent {
                    request_id: request.id,
                    invitation_id,
                },
                their_ddo,
                enc_key,
            ),
            output: response,
        })
    }

    pub fn receive_complete(
        self,
        complete: Complete,
    ) -> Result<DidExchangeServiceResponder<Completed>, TransitionError<Self>> {
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
        Ok(DidExchangeServiceResponder::from_parts(
            Completed {
                invitation_id: self.state.invitation_id,
                request_id: self.state.request_id,
            },
            self.their_did_document,
            self.our_verkey,
        ))
    }
}
