use std::sync::Arc;

use aries_vcx_core::{ledger::base_ledger::IndyLedgerRead, wallet::base_wallet::BaseWallet};
use chrono::Utc;
use did_parser::Did;
use did_peer::resolver::PeerDidResolver;
use did_resolver::traits::resolvable::DidResolvable;
use did_resolver_registry::ResolverRegistry;
use helpers::{
    construct_request, verify_handshake_protocol,
};
use messages::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::{
        did_exchange::{
            complete::{Complete as CompleteMessage, Complete, CompleteDecorators},
            request::Request,
            response::Response,
        },
        out_of_band::invitation::Invitation,
    },
};
use url::Url;
use uuid::Uuid;
use did_peer::peer_did::numalgos::numalgo2::Numalgo2;
use did_peer::peer_did::numalgos::numalgo2::resolve::resolve_numalgo2;
use did_peer::peer_did::PeerDid;
use messages::msg_fields::protocols::out_of_band::invitation::{invitation_get_first_did_service, OobService};

use super::DidExchangeRequester;
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    protocols::did_exchange::{
        state_machine::helpers::{attach_to_ddo_sov, create_our_did_document, to_transition_error},
        states::{completed::Completed, requester::request_sent::RequestSent},
        transition::{transition_error::TransitionError, transition_result::TransitionResult},
    },
};
use crate::errors::error::VcxResult;

mod helpers;

/// We are going to support only DID service values in did-exchange protocol unless there's explicit
/// good reason to keep support for "embedded" type of service value.
/// This function returns first found DID based service value from invitation.
pub fn invitation_get_first_did_service(invitation: &Invitation) -> VcxResult<Did> {
    let did_string = invitation
        .content
        .services
        .iter()
        .map(|service| match service {
            OobService::Did(did) => Some(did.clone()),
            _ => None,
        })
        .filter(|did| did.is_some())
        .first()
        .ok_or(|| {
            Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Invitation does not contain did service",
            ))
        })?;
    Did::parse(did_string)
        .map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                format!("Invalid DID in invitation: {}", err),
            )
        })
}

impl DidExchangeRequester<RequestSent> {
    pub async fn construct_request(
        resolver_registry: Arc<ResolverRegistry>,
        their_did: Did,
        our_peer_did: PeerDid<Numalgo2>
    ) -> Result<TransitionResult<Self, Request>, AriesVcxError> {
        let their_did_document = resolver_registry.resolve(&their_did, &Default::default()).await?.did_document().clone();
        // todo: resolving our did is strange, we should require caller to pass diddoc/peer:did/or data to construct these
        let our_did_document = resolve_numalgo2(our_peer_did)?.build();
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
            attach_to_ddo_sov(ddo).map_err(to_transition_error(self.clone()))?
        } else {
            PeerDidResolver::new()
                .resolve(
                    &response
                        .content
                        .did
                        .parse()
                        .map_err(to_transition_error(self.clone()))?,
                    &Default::default(),
                )
                .await
                .map_err(to_transition_error(self.clone()))?
                .did_document()
                .to_owned()
                .into()
        };
        let decorators = CompleteDecorators::builder()
            .thread(
                Thread::builder()
                    .thid(self.state.request_id.clone())
                    .pthid(self.state.invitation_id.clone())
                    .build(),
            )
            .timing(Timing::builder().out_time(Utc::now()).build())
            .build();
        let complete_message = Complete::builder()
            .id(Uuid::new_v4().to_string())
            .decorators(decorators)
            .build();

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
