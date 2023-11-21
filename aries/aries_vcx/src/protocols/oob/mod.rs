use aries_vcx_core::ledger::base_ledger::IndyLedgerRead;
use chrono::Utc;
use diddoc_legacy::aries::{diddoc::AriesDidDoc, service::AriesService};
use messages::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::out_of_band::{
        invitation::Invitation,
        reuse::{HandshakeReuse, HandshakeReuseDecorators},
        reuse_accepted::{HandshakeReuseAccepted, HandshakeReuseAcceptedDecorators},
    },
};
use uuid::Uuid;

use crate::{
    common::ledger::transactions::resolve_service,
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
};

const DID_KEY_PREFIX: &str = "did:key:";
const ED25519_MULTIBASE_CODEC: [u8; 2] = [0xed, 0x01];

pub fn build_handshake_reuse_msg(oob_invitation: &Invitation) -> HandshakeReuse {
    let id = Uuid::new_v4().to_string();

    let decorators = HandshakeReuseDecorators::builder()
        .thread(
            Thread::builder()
                .thid(id.clone())
                .pthid(oob_invitation.id.clone())
                .build(),
        )
        .timing(Timing::builder().out_time(Utc::now()).build())
        .build();

    HandshakeReuse::builder()
        .id(id)
        .decorators(decorators)
        .build()
}

pub fn build_handshake_reuse_accepted_msg(
    handshake_reuse: &HandshakeReuse,
) -> VcxResult<HandshakeReuseAccepted> {
    let thread = &handshake_reuse.decorators.thread;

    let thread_id = thread.thid.clone();
    let pthread_id = thread
        .pthid
        .as_deref()
        .ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidOption,
            "Parent thread id missing",
        ))?
        .to_owned();

    let decorators = HandshakeReuseAcceptedDecorators::builder()
        .thread(Thread::builder().thid(thread_id).pthid(pthread_id).build())
        .timing(Timing::builder().out_time(Utc::now()).build())
        .build();

    Ok(HandshakeReuseAccepted::builder()
        .id(Uuid::new_v4().to_string())
        .decorators(decorators)
        .build())
}

pub async fn oob_invitation_to_legacy_did_doc(
    indy_ledger: &impl IndyLedgerRead,
    invitation: &Invitation,
) -> VcxResult<AriesDidDoc> {
    let mut did_doc: AriesDidDoc = AriesDidDoc::default();
    let (service_endpoint, recipient_keys, routing_keys) = {
        did_doc.set_id(invitation.id.clone());
        let service = resolve_service(indy_ledger, &invitation.content.services[0])
            .await
            .unwrap_or_else(|err| {
                error!("Failed to obtain service definition from the ledger: {err}");
                AriesService::default()
            });
        let recipient_keys =
            normalize_keys_as_naked(&service.recipient_keys).unwrap_or_else(|err| {
                error!(
                    "Failed to normalize keys of service {} as naked keys: {err}",
                    &service
                );
                Vec::new()
            });
        (
            service.service_endpoint,
            recipient_keys,
            service.routing_keys,
        )
    };
    did_doc.set_service_endpoint(service_endpoint);
    did_doc.set_recipient_keys(recipient_keys);
    did_doc.set_routing_keys(routing_keys);
    Ok(did_doc)
}

fn normalize_keys_as_naked(keys_list: &Vec<String>) -> VcxResult<Vec<String>> {
    let mut result = Vec::new();
    for key in keys_list {
        if let Some(stripped_didkey) = key.strip_prefix(DID_KEY_PREFIX) {
            let stripped = if let Some(stripped) = stripped_didkey.strip_prefix('z') {
                stripped
            } else {
                Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidDid,
                    format!("z prefix is missing: {}", key),
                ))?
            };
            let decoded_value = bs58::decode(stripped).into_vec().map_err(|_| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidDid,
                    format!(
                        "Could not decode base58: {} as portion of {}",
                        stripped, key
                    ),
                )
            })?;
            let verkey = if let Some(public_key_bytes) =
                decoded_value.strip_prefix(&ED25519_MULTIBASE_CODEC)
            {
                Ok(bs58::encode(public_key_bytes).into_string())
            } else {
                Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidDid,
                    format!(
                        "Only Ed25519-based did:keys are currently supported, got key: {}",
                        key
                    ),
                ))
            }?;
            result.push(verkey);
        } else {
            result.push(key.clone());
        }
    }
    Ok(result)
}

// #[cfg(test)]
// mod unit_tests {
//     use crate::protocols::oob::{build_handshake_reuse_accepted_msg, build_handshake_reuse_msg};
//     use crate::utils::devsetup::{was_in_past, SetupMocks};
//     use messages::a2a::MessageId;
//     use messages::protocols::out_of_band::invitation::OutOfBandInvitation;

//     #[test]
//     fn test_build_handshake_reuse_msg() {
//         let _setup = SetupMocks::init();
//         let msg_invitation = OutOfBandInvitation::default();
//         let msg_reuse = build_handshake_reuse_msg(&msg_invitation);

//         assert_eq!(msg_reuse.id, MessageId("testid".into()));
//         assert_eq!(msg_reuse.id, MessageId(msg_reuse.thread.thid.unwrap()));
//         assert_eq!(msg_reuse.thread.pthid.unwrap(), msg_invitation.id.0);
//         assert!(was_in_past(
//             &msg_reuse.timing.unwrap().out_time.unwrap(),
//             chrono::Duration::milliseconds(100)
//         )
//         .unwrap());
//     }

//     #[test]
//     fn test_build_handshake_reuse_accepted_msg() {
//         let _setup = SetupMocks::init();
//         let mut msg_invitation = OutOfBandInvitation::default();
//         msg_invitation.id = MessageId("invitation-id".to_string());
//         let msg_reuse = build_handshake_reuse_msg(&msg_invitation);
//         let msg_reuse_accepted = build_handshake_reuse_accepted_msg(&msg_reuse).unwrap();

//         assert_eq!(msg_reuse_accepted.id, MessageId("testid".into()));
//         assert_eq!(msg_reuse_accepted.thread.thid.unwrap(), msg_reuse.id.0);
//         assert_eq!(msg_reuse_accepted.thread.pthid.unwrap(), msg_invitation.id.0);
//         assert!(was_in_past(
//             &msg_reuse_accepted.timing.unwrap().out_time.unwrap(),
//             chrono::Duration::milliseconds(100)
//         )
//         .unwrap());
//     }
// }
