use chrono::Utc;
use did_parser_nom::Did;
use messages::{
    decorators::{
        thread::{Thread, ThreadGoalCode},
        timing::Timing,
    },
    msg_fields::protocols::{
        did_exchange::v1_0::{
            complete::{Complete, CompleteDecorators},
            request::{Request, RequestContent, RequestDecorators},
        },
        out_of_band::invitation::{Invitation, OobService},
    },
};
use shared::maybe_known::MaybeKnown;
use uuid::Uuid;

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

pub fn construct_request(invitation_id: Option<String>, our_did: String) -> Request {
    let msg_id = Uuid::new_v4().to_string();
    let thid = msg_id.clone();
    let thread = match invitation_id {
        Some(invitation_id) => Thread::builder().thid(thid).pthid(invitation_id).build(),
        None => Thread::builder().thid(thid).build(),
    };
    let decorators = RequestDecorators::builder()
        .thread(Some(thread))
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
        .id(msg_id)
        .content(content)
        .decorators(decorators)
        .build()
}

pub fn construct_didexchange_complete(request_id: String) -> Complete {
    // assuming we'd want to support RFC 100% and include pthread in complete message, we can add
    // new function argument: `invitation_id: Option<String>`
    // We choose not to do this, as it's rather historic artifact and doesn't have justification in
    // practice see https://github.com/hyperledger/aries-rfcs/issues/817
    // We can then build thread decorator conditionally:
    // let thread = match invitation_id {
    //     Some(invitation_id) => Thread::builder()
    //         .thid(request_id)
    //         .pthid(invitation_id)
    //         .build(),
    //     None => Thread::builder()
    //         .thid(request_id)
    //         .build()
    // };
    let thread = Thread::builder().thid(request_id).build();
    let decorators = CompleteDecorators::builder()
        .thread(thread)
        .timing(Timing::builder().out_time(Utc::now()).build())
        .build();
    Complete::builder()
        .id(Uuid::new_v4().to_string())
        .decorators(decorators)
        .build()
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
