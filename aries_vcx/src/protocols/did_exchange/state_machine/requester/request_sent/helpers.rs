use std::sync::Arc;

use aries_vcx_core::ledger::base_ledger::IndyLedgerRead;
use did_doc::schema::verification_method::{VerificationMethod, VerificationMethodType};
use did_doc_sov::{service::ServiceSov, DidDocumentSov};
use did_parser::{Did, DidUrl};
use messages::{
    decorators::{
        attachment::Attachment,
        thread::{Thread, ThreadGoalCode},
    },
    msg_fields::protocols::{
        did_exchange::{
            complete::{Complete as CompleteMessage, CompleteDecorators},
            request::{Request, RequestContent, RequestDecorators},
        },
        out_of_band::invitation::{Invitation as OobInvitation, OobService},
    },
};
use shared_vcx::{maybe_known::MaybeKnown, misc::serde_ignored::SerdeIgnored as NoContent};
use uuid::Uuid;

use crate::{
    common::ledger::transactions::resolve_service,
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    utils::from_legacy_service_to_service_sov,
};

pub fn verify_handshake_protocol(invitation: OobInvitation) -> Result<(), AriesVcxError> {
    invitation
        .content
        .handshake_protocols
        .ok_or_else(|| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Invitation does not contain handshake protpcols",
            )
        })?
        .iter()
        .find(|protocol| match protocol {
            // TODO: Improve this check
            MaybeKnown::Known(protocol) if protocol.to_string().contains("didexchange") => true,
            _ => false,
        })
        .ok_or_else(|| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Invitation does not contain didexchange handshake protocol",
            )
        })?;
    Ok(())
}

pub async fn did_doc_from_did(
    ledger: &Arc<dyn IndyLedgerRead>,
    did: Did,
) -> Result<(DidDocumentSov, ServiceSov), AriesVcxError> {
    let service = resolve_service(ledger, &OobService::Did(did.id().to_string())).await?;
    let did_url: DidUrl = format!("{}#vm-0", did.to_string()).try_into()?;
    let vm = VerificationMethod::builder(did_url, did.clone(), VerificationMethodType::Ed25519VerificationKey2020)
        .add_public_key_base58(
            service
                .recipient_keys
                .first()
                .ok_or_else(|| {
                    AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidState,
                        "No recipient keys found in resolved service",
                    )
                })?
                .clone(),
        )
        .build();
    let sov_service = from_legacy_service_to_service_sov(service.clone())?;
    let did_document = DidDocumentSov::builder(did.clone())
        .add_service(sov_service.clone())
        .add_controller(did)
        .add_verification_method(vm)
        .build();
    Ok((did_document, sov_service))
}

// TODO: Replace by a builder
pub fn construct_request(invitation_id: String, our_did: String) -> Result<Request, AriesVcxError> {
    let request_id = Uuid::new_v4().to_string();
    let thread = {
        let mut thread = Thread::new(request_id.clone());
        thread.pthid = Some(invitation_id.clone());
        thread
    };
    let decorators = {
        let mut decorators = RequestDecorators::default();
        decorators.thread = Some(thread);
        decorators
    };
    let content = RequestContent {
        // TODO: Obviously, these fields should be more configurable
        label: "".to_string(),
        // Interop note: Rejected if non-empty by acapy (regardless of invite contents)
        goal: Some("To establish a connection".to_string()),
        // Interop note: Rejected if non-empty by acapy (regardless of invite contents)
        goal_code: Some(MaybeKnown::Known(ThreadGoalCode::AriesRelBuild)),
        // Interop note: Should not have to send both DID and DDO if did resolvable
        did: our_did,
        did_doc: None,
    };
    Ok(Request::with_decorators(request_id.clone(), content, decorators))
}

// TODO: Replace by a builder
pub fn construct_complete_message(invitation_id: String, request_id: String) -> CompleteMessage {
    let complete_id = Uuid::new_v4().to_string();
    let decorators = {
        let mut thread = Thread::new(request_id);
        thread.pthid = Some(invitation_id);
        CompleteDecorators { thread, timing: None }
    };
    CompleteMessage::with_decorators(complete_id, NoContent::default(), decorators)
}
