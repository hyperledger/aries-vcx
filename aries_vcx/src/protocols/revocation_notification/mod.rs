pub mod receiver;
pub mod sender;

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use messages::decorators::please_ack::{AckOn, PleaseAck};
    use messages::maybe_known::MaybeKnown;
    use messages::msg_fields::protocols::revocation::revoke::{
        RevocationFormat, Revoke, RevokeContent, RevokeDecorators,
    };
    use messages::AriesMessage;
    use uuid::Uuid;

    use crate::errors::error::VcxResult;
    use crate::{protocols::SendClosure, utils::constants::REV_REG_ID};

    pub fn _send_message() -> SendClosure {
        Box::new(|_: AriesMessage| Box::pin(async { VcxResult::Ok(()) }))
    }

    pub fn _rev_reg_id() -> String {
        String::from(REV_REG_ID)
    }

    pub fn _cred_rev_id() -> String {
        String::from("12")
    }

    pub fn _comment() -> Option<String> {
        Some("Comment.".to_string())
    }

    pub fn _revocation_notification(ack_on: Vec<AckOn>) -> Revoke {
        let id = Uuid::new_v4().to_string();

        let mut content = RevokeContent::new(
            format!("{}::{}", _rev_reg_id(), _cred_rev_id()),
            MaybeKnown::Known(RevocationFormat::IndyAnoncreds),
        );
        content.comment = _comment();

        let mut decorators = RevokeDecorators::default();
        let please_ack = PleaseAck::new(ack_on);
        decorators.please_ack = Some(please_ack);

        Revoke::with_decorators(id, content, decorators)
    }

    pub fn _revocation_notification_invalid_format() -> Revoke {
        let id = Uuid::new_v4().to_string();

        let mut content = RevokeContent::new(
            format!("{}::{}", _rev_reg_id(), _cred_rev_id()),
            MaybeKnown::Known(RevocationFormat::IndyAnoncreds),
        );
        content.comment = _comment();

        let mut decorators = RevokeDecorators::default();
        let please_ack = PleaseAck::new(vec![AckOn::Receipt]);
        decorators.please_ack = Some(please_ack);

        Revoke::with_decorators(id, content, decorators)
    }
}
