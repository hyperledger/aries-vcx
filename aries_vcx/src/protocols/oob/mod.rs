use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};
use chrono::Utc;
use messages::decorators::thread::Thread;
use messages::decorators::timing::Timing;
use messages::msg_fields::protocols::out_of_band::invitation::Invitation;
use messages::msg_fields::protocols::out_of_band::reuse::{HandshakeReuse, HandshakeReuseDecorators};
use messages::msg_fields::protocols::out_of_band::reuse_accepted::{
    HandshakeReuseAccepted, HandshakeReuseAcceptedDecorators,
};

use uuid::Uuid;

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

    HandshakeReuse::builder().id(id).decorators(decorators).build()
}

pub fn build_handshake_reuse_accepted_msg(handshake_reuse: &HandshakeReuse) -> VcxResult<HandshakeReuseAccepted> {
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
