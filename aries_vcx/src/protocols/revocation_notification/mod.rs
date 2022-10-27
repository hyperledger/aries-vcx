pub mod receiver;
pub mod sender;

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use messages::ack::please_ack::AckOn;
    use messages::revocation_notification::revocation_notification::RevocationFormat;
    use messages::{a2a::A2AMessage, revocation_notification::revocation_notification::RevocationNotification};

    use crate::{error::VcxResult, protocols::SendClosure, utils::constants::REV_REG_ID};

    pub fn _send_message() -> SendClosure {
        Box::new(|_: A2AMessage| Box::pin(async { VcxResult::Ok(()) }))
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

    pub fn _revocation_notification(ack_on: Vec<AckOn>) -> RevocationNotification {
        RevocationNotification::create()
            .set_credential_id(_rev_reg_id(), _cred_rev_id())
            .set_ack_on(ack_on)
            .set_comment(_comment())
            .set_revocation_format(RevocationFormat::IndyAnoncreds)
    }

    pub fn _revocation_notification_invalid_format() -> RevocationNotification {
        RevocationNotification::create()
            .set_credential_id(_rev_reg_id(), _cred_rev_id())
            .set_ack_on(vec![AckOn::Receipt])
            .set_comment(_comment())
            .set_revocation_format(RevocationFormat::IndyAnoncreds)
    }
}
