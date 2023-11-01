use std::sync::Arc;

use aries_vcx_core::ledger::base_ledger::IndyLedgerRead;
use chrono::Utc;
use did_doc::schema::verification_method::{VerificationMethod, VerificationMethodType};
use did_parser::{Did, DidUrl};
use did_resolver::traits::resolvable::resolution_output::DidResolutionOutput;
use did_resolver_registry::ResolverRegistry;
use messages::{
    decorators::{
        thread::{Thread, ThreadGoalCode},
        timing::Timing,
    },
    msg_fields::protocols::{
        did_exchange::request::{Request, RequestContent, RequestDecorators},
        out_of_band::invitation::{Invitation as OobInvitation, Invitation, OobService},
    },
};
use shared::maybe_known::MaybeKnown;
use uuid::Uuid;
use did_doc::schema::did_doc::DidDocument;

use crate::{
    common::ledger::transactions::resolve_service,
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    utils::from_legacy_service,
};


// pub async fn did_doc_from_did(
//     ledger: &impl IndyLedgerRead,
//     did: Did,
// ) -> Result<(DidDocument, Service), AriesVcxError> {
//     let service = resolve_service(ledger, &OobService::Did(did.id().to_string())).await?;
//     let did_url: DidUrl = format!("{}#vm-0", did).try_into()?;
//     let vm = VerificationMethod::builder(
//         did_url,
//         did.clone(),
//         VerificationMethodType::Ed25519VerificationKey2020,
//     )
//     .add_public_key_base58(
//         service
//             .recipient_keys
//             .first()
//             .ok_or_else(|| {
//                 AriesVcxError::from_msg(
//                     AriesVcxErrorKind::InvalidState,
//                     "No recipient keys found in resolved service",
//                 )
//             })?
//             .clone(),
//     )
//     .build();
//     let sov_service = from_legacy_service_to_service_sov(service)?;
//     let did_document = DidDocumentSov::builder(did.clone())
//         .add_service(sov_service.clone())
//         .add_controller(did)
//         .add_verification_method(vm)
//         .build();
//     Ok((did_document, sov_service))
// }

pub fn construct_request(invitation_id: String, our_did: String) -> Request {
    let request_id = Uuid::new_v4().to_string();
    let decorators = RequestDecorators::builder()
        .thread(Some(
            Thread::builder()
                .thid(request_id.clone())
                .pthid(invitation_id)
                .build(),
        ))
        .timing(Timing::builder().out_time(Utc::now()).build())
        .build();
    let content = RequestContent::builder()
        .label("".into())
        .did(our_did)
        .did_doc(None)
        .goal(Some("To establish a connection".into())) // Rejected if non-empty by acapy
        .goal_code(Some(MaybeKnown::Known(ThreadGoalCode::AriesRelBuild))) // Rejected if non-empty by acapy
        .build();
    Request::builder()
        .id(request_id)
        .content(content)
        .decorators(decorators)
        .build()
}

//
// pub async fn oob_invitation_to_diddoc(
//     resolver_registry: &Arc<ResolverRegistry>,
//     invitation: Invitation,
// ) -> VcxResult<DidDocument> {
//     let mut builder = DidDocument::builder(Default::default());
//
//     let mut resolved_services = vec![];
//     let mut resolved_vms = vec![];
//     let mut resolved_kas = vec![];
//     let mut resolved_dids = vec![];
//
//     for service in invitation.content.services {
//         match service {
//             OobService::SovService(service) => {
//                 builder = builder.add_service(service.clone());
//             }
//             OobService::Did(did) => {
//                 let parsed_did = Did::parse(did)?;
//                 let DidResolutionOutput { did_document, .. } = resolver_registry
//                     .resolve(&parsed_did, &Default::default())
//                     .await?;
//                 resolved_services.extend(
//                     did_document
//                         .service()
//                         .iter()
//                         .map(|s| ServiceSov::try_from(s.clone()))
//                         .collect::<Result<Vec<_>, _>>()?,
//                 );
//                 resolved_vms.extend_from_slice(did_document.verification_method());
//                 resolved_kas.extend_from_slice(did_document.key_agreement());
//                 resolved_dids.push(parsed_did);
//             }
//             OobService::AriesService(service) => {
//                 resolved_services.push(from_legacy_service(service.clone())?)
//             }
//         }
//     }
//
//     for service in resolved_services {
//         builder = builder.add_service(service);
//     }
//
//     for vm in resolved_vms {
//         builder = builder.add_verification_method(vm.clone());
//     }
//
//     for ka in resolved_kas {
//         builder = builder.add_key_agreement(ka.clone());
//     }
//
//     for did in resolved_dids {
//         builder = builder.add_controller(did);
//     }
//
//     Ok(builder.build())
// }
