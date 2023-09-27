pub mod receiver;
pub mod sender;

pub mod test_utils {
    use messages::{
        decorators::please_ack::{AckOn, PleaseAck},
        msg_fields::protocols::revocation::revoke::{
            RevocationFormat, Revoke, RevokeContent, RevokeDecorators,
        },
        AriesMessage,
    };
    use shared_vcx::maybe_known::MaybeKnown;
    use uuid::Uuid;

    use crate::{errors::error::VcxResult, protocols::SendClosure, utils::constants::REV_REG_ID};

    pub fn _send_message() -> SendClosure<'static> {
        Box::new(|_: AriesMessage| Box::pin(async { VcxResult::Ok(()) }))
    }

    pub fn _rev_reg_id() -> String {
        String::from(REV_REG_ID)
    }

    pub fn _cred_rev_id() -> String {
        String::from("12")
    }

    pub fn _comment() -> String {
        "Comment.".to_string()
    }

    pub fn _revocation_notification(ack_on: Vec<AckOn>) -> Revoke {
        let id = Uuid::new_v4().to_string();

        let content = RevokeContent::builder()
            .credential_id(format!("{}::{}", _rev_reg_id(), _cred_rev_id()))
            .revocation_format(MaybeKnown::Known(RevocationFormat::IndyAnoncreds))
            .comment(_comment())
            .build();

        let decorators = RevokeDecorators::builder()
            .please_ack(PleaseAck::builder().on(ack_on).build())
            .build();

        Revoke::builder()
            .id(id)
            .content(content)
            .decorators(decorators)
            .build()
    }
}
