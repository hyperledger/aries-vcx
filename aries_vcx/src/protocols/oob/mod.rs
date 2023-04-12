use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};
use chrono::Utc;
use messages::decorators::thread::Thread;
use messages::decorators::timing::Timing;
use messages::msg_fields::protocols::out_of_band::invitation::Invitation;
use messages::msg_fields::protocols::out_of_band::reuse::{
    HandshakeReuse, HandshakeReuseContent, HandshakeReuseDecorators,
};
use messages::msg_fields::protocols::out_of_band::reuse_accepted::{
    HandshakeReuseAccepted, HandshakeReuseAcceptedContent, HandshakeReuseAcceptedDecorators,
};

use uuid::Uuid;

pub fn build_handshake_reuse_msg(oob_invitation: &Invitation) -> HandshakeReuse {
    let id = Uuid::new_v4().to_string();
    let content = HandshakeReuseContent::default();

    let mut thread = Thread::new(id.clone());
    thread.pthid = Some(oob_invitation.id.clone());

    let mut decorators = HandshakeReuseDecorators::new(thread);
    let mut timing = Timing::default();
    timing.out_time = Some(Utc::now());
    decorators.timing = Some(timing);

    HandshakeReuse::with_decorators(id, content, decorators)
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

    let content = HandshakeReuseAcceptedContent::default();
    let mut thread = Thread::new(thread_id);
    thread.pthid = Some(pthread_id);

    let decorators = HandshakeReuseAcceptedDecorators::new(thread);
    let mut timing = Timing::default();
    timing.out_time = Some(Utc::now());

    Ok(HandshakeReuseAccepted::with_decorators(
        Uuid::new_v4().to_string(),
        content,
        decorators,
    ))
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {
    use crate::protocols::oob::{build_handshake_reuse_accepted_msg, build_handshake_reuse_msg};
    use crate::utils::devsetup::{was_in_past, SetupMocks};
    use messages::a2a::MessageId;
    use messages::protocols::out_of_band::invitation::OutOfBandInvitation;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_build_handshake_reuse_msg() {
        let _setup = SetupMocks::init();
        let msg_invitation = OutOfBandInvitation::default();
        let msg_reuse = build_handshake_reuse_msg(&msg_invitation);

        assert_eq!(msg_reuse.id, MessageId("testid".into()));
        assert_eq!(msg_reuse.id, MessageId(msg_reuse.thread.thid.unwrap()));
        assert_eq!(msg_reuse.thread.pthid.unwrap(), msg_invitation.id.0);
        assert!(was_in_past(
            &msg_reuse.timing.unwrap().out_time.unwrap(),
            chrono::Duration::milliseconds(100)
        )
        .unwrap());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_build_handshake_reuse_accepted_msg() {
        let _setup = SetupMocks::init();
        let mut msg_invitation = OutOfBandInvitation::default();
        msg_invitation.id = MessageId("invitation-id".to_string());
        let msg_reuse = build_handshake_reuse_msg(&msg_invitation);
        let msg_reuse_accepted = build_handshake_reuse_accepted_msg(&msg_reuse).unwrap();

        assert_eq!(msg_reuse_accepted.id, MessageId("testid".into()));
        assert_eq!(msg_reuse_accepted.thread.thid.unwrap(), msg_reuse.id.0);
        assert_eq!(msg_reuse_accepted.thread.pthid.unwrap(), msg_invitation.id.0);
        assert!(was_in_past(
            &msg_reuse_accepted.timing.unwrap().out_time.unwrap(),
            chrono::Duration::milliseconds(100)
        )
        .unwrap());
    }
}
