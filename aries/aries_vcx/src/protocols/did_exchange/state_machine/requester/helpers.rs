use chrono::Utc;
use did_parser::Did;
use messages::{
    decorators::{
        thread::{Thread, ThreadGoalCode},
        timing::Timing,
    },
    msg_fields::protocols::{
        did_exchange::request::{Request, RequestContent, RequestDecorators},
        out_of_band::invitation::{Invitation, OobService},
    },
};
use shared::maybe_known::MaybeKnown;
use uuid::Uuid;

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

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

/// We are going to support only DID service values in did-exchange protocol unless there's explicit
/// good reason to keep support for "embedded" type of service value.
/// This function returns first found DID based service value from invitation.
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
