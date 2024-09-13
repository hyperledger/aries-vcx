use chrono::Utc;
use did_parser_nom::Did;
use messages::{
    decorators::{
        thread::{Thread, ThreadGoalCode},
        timing::Timing,
        transport::Transport,
    },
    msg_fields::protocols::{
        did_exchange::v1_x::{
            complete::{AnyComplete, Complete, CompleteDecorators},
            request::{AnyRequest, Request, RequestContent, RequestDecorators},
        },
        out_of_band::invitation::{Invitation, OobService},
    },
    msg_types::{
        protocols::did_exchange::{DidExchangeType, DidExchangeTypeV1},
        Protocol,
    },
};
use shared::maybe_known::MaybeKnown;
use uuid::Uuid;

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

pub fn construct_request(
    invitation_id: Option<String>,
    our_did: String,
    our_label: String,
    version: DidExchangeTypeV1,
    transport_decorator: Option<Transport>,
) -> AnyRequest {
    let msg_id = Uuid::new_v4().to_string();
    let thid = msg_id.clone();
    let thread = match invitation_id {
        Some(invitation_id) => Thread::builder().thid(thid).pthid(invitation_id).build(),
        None => Thread::builder().thid(thid).build(),
    };
    let decorators = RequestDecorators::builder()
        .thread(Some(thread))
        .timing(Timing::builder().out_time(Utc::now()).build())
        .transport(transport_decorator)
        .build();
    let content = RequestContent::builder()
        .label(our_label)
        .did(our_did)
        .did_doc(None)
        .goal(Some("To establish a connection".into())) // Rejected if non-empty by acapy
        .goal_code(Some(MaybeKnown::Known(ThreadGoalCode::AriesRelBuild))) // Rejected if non-empty by acapy
        .build();
    let req = Request::builder()
        .id(msg_id)
        .content(content)
        .decorators(decorators)
        .build();

    match version {
        DidExchangeTypeV1::V1_1(_) => AnyRequest::V1_1(req),
        DidExchangeTypeV1::V1_0(_) => AnyRequest::V1_0(req),
    }
}

pub fn construct_didexchange_complete(
    // pthid inclusion is overkill in practice, but needed. see: https://github.com/hyperledger/aries-rfcs/issues/817
    invitation_id: Option<String>,
    request_id: String,
    version: DidExchangeTypeV1,
) -> AnyComplete {
    let thread = match invitation_id {
        Some(invitation_id) => Thread::builder()
            .thid(request_id)
            .pthid(invitation_id)
            .build(),
        None => Thread::builder().thid(request_id).build(),
    };
    let decorators = CompleteDecorators::builder()
        .thread(thread)
        .timing(Timing::builder().out_time(Utc::now()).build())
        .build();
    let msg = Complete::builder()
        .id(Uuid::new_v4().to_string())
        .decorators(decorators)
        .build();

    match version {
        DidExchangeTypeV1::V1_1(_) => AnyComplete::V1_1(msg),
        DidExchangeTypeV1::V1_0(_) => AnyComplete::V1_0(msg),
    }
}

/// We are going to support only DID service values in did-exchange protocol unless there's explicit
/// good reason to keep support for "embedded" type of service value.
/// This function returns first found DID based service value from invitation.
/// TODO: also used by harness, move this to messages crate
pub fn invitation_get_first_did_service(invitation: &Invitation) -> VcxResult<Did> {
    for service in invitation.content.services.iter() {
        if let OobService::Did(did_string) = service {
            return Did::parse(did_string.clone()).map_err(|err| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    format!("Invalid DID in invitation: {}", err),
                )
            });
        }
    }
    Err(AriesVcxError::from_msg(
        AriesVcxErrorKind::InvalidState,
        "Invitation does not contain did service",
    ))
}

/// Finds the best suitable DIDExchange V1.X version specified in an invitation, or an error if
/// none.
pub fn invitation_get_acceptable_did_exchange_version(
    invitation: &Invitation,
) -> VcxResult<DidExchangeTypeV1> {
    // determine acceptable protocol
    let mut did_exch_v1_1_accepted = false;
    let mut did_exch_v1_0_accepted = false;
    for proto in invitation.content.handshake_protocols.iter().flatten() {
        let MaybeKnown::Known(Protocol::DidExchangeType(DidExchangeType::V1(exch_proto))) = proto
        else {
            continue;
        };
        if matches!(exch_proto, DidExchangeTypeV1::V1_1(_)) {
            did_exch_v1_1_accepted = true;
            continue;
        }
        if matches!(exch_proto, DidExchangeTypeV1::V1_0(_)) {
            did_exch_v1_0_accepted = true;
        }
    }

    let version = match (did_exch_v1_1_accepted, did_exch_v1_0_accepted) {
        (true, _) => DidExchangeTypeV1::new_v1_1(),
        (false, true) => DidExchangeTypeV1::new_v1_0(),
        _ => {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidInput,
                "OOB invitation does not have a suitable handshake protocol for DIDExchange",
            ))
        }
    };

    Ok(version)
}
