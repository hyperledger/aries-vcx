use messages::decorators::please_ack::AckOn;

use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    handlers::{
        issuance::issuer::Issuer, revocation_notification::sender::RevocationNotificationSender,
    },
    protocols::{revocation_notification::sender::state_machine::SenderConfigBuilder, SendClosure},
};

pub mod receiver;
pub mod sender;

pub async fn send_revocation_notification(
    issuer: &Issuer,
    ack_on: Vec<AckOn>,
    comment: Option<String>,
    send_message: SendClosure<'_>,
) -> VcxResult<()> {
    // TODO: Check if actually revoked
    if issuer.is_revokable() {
        // TODO: Store to allow checking not. status (sent, acked)
        let config = SenderConfigBuilder::default()
            .rev_reg_id(issuer.get_rev_reg_id()?)
            .cred_rev_id(issuer.get_rev_id()?)
            .comment(comment)
            .ack_on(ack_on)
            .build()?;
        RevocationNotificationSender::build()
            .send_revocation_notification(config, send_message)
            .await?;
        Ok(())
    } else {
        Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidState,
            format!(
                "Can't send revocation notification in state {:?}, credential is not revokable",
                issuer.get_state()
            ),
        ))
    }
}
