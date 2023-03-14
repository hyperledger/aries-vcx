use messages::protocols::out_of_band::{
    handshake_reuse::OutOfBandHandshakeReuse, handshake_reuse_accepted::OutOfBandHandshakeReuseAccepted,
    invitation::OutOfBandInvitation,
};

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

pub fn build_handshake_reuse_msg(oob_invitation: &OutOfBandInvitation) -> OutOfBandHandshakeReuse {
    OutOfBandHandshakeReuse::default()
        .set_thread_id_matching_id()
        .set_parent_thread_id(&oob_invitation.id.0)
        .set_out_time()
}

pub fn build_handshake_reuse_accepted_msg(
    handshake_reuse: &OutOfBandHandshakeReuse,
) -> VcxResult<OutOfBandHandshakeReuseAccepted> {
    let thread_id = handshake_reuse.get_thread_id();
    let pthread_id = handshake_reuse.thread.pthid.as_deref().ok_or(AriesVcxError::from_msg(
        AriesVcxErrorKind::InvalidOption,
        "Parent thread id missing",
    ))?;
    Ok(OutOfBandHandshakeReuseAccepted::default()
        .set_thread_id(&thread_id)
        .set_parent_thread_id(pthread_id)
        .set_out_time())
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {
    use messages::{a2a::MessageId, protocols::out_of_band::invitation::OutOfBandInvitation};

    use crate::{
        protocols::oob::{build_handshake_reuse_accepted_msg, build_handshake_reuse_msg},
        utils::devsetup::{was_in_past, SetupMocks},
    };

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
