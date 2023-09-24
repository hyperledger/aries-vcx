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
    send_message: SendClosure,
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

pub mod test_utils {
    use agency_client::agency_client::AgencyClient;
    use messages::{
        msg_fields::protocols::revocation::{ack::AckRevoke, revoke::Revoke, Revocation},
        AriesMessage,
    };

    use crate::{
        errors::error::prelude::*, handlers::connection::mediated_connection::MediatedConnection,
    };

    pub async fn get_revocation_notification_messages(
        agency_client: &AgencyClient,
        connection: &MediatedConnection,
    ) -> VcxResult<Vec<Revoke>> {
        let mut messages = Vec::<Revoke>::new();
        for (uid, message) in connection
            .get_messages_noauth(agency_client)
            .await?
            .into_iter()
        {
            if let AriesMessage::Revocation(Revocation::Revoke(message)) = message {
                connection
                    .update_message_status(&uid, agency_client)
                    .await
                    .ok();
                messages.push(message);
            }
        }
        Ok(messages)
    }

    pub async fn get_revocation_notification_ack_messages(
        agency_client: &AgencyClient,
        connection: &MediatedConnection,
    ) -> VcxResult<Vec<AckRevoke>> {
        let mut messages = Vec::<AckRevoke>::new();
        for (uid, message) in connection
            .get_messages_noauth(agency_client)
            .await?
            .into_iter()
        {
            if let AriesMessage::Revocation(Revocation::Ack(message)) = message {
                connection
                    .update_message_status(&uid, agency_client)
                    .await
                    .ok();
                messages.push(message);
            }
        }
        Ok(messages)
    }
}
