pub mod receiver;
pub mod sender;

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use agency_client::agency_client::AgencyClient;
    use messages::msg_fields::protocols::revocation::ack::AckRevoke;
    use messages::msg_fields::protocols::revocation::revoke::Revoke;
    use messages::msg_fields::protocols::revocation::Revocation;
    use messages::AriesMessage;

    use crate::errors::error::prelude::*;
    use crate::handlers::connection::mediated_connection::MediatedConnection;

    pub async fn get_revocation_notification_messages(
        agency_client: &AgencyClient,
        connection: &MediatedConnection,
    ) -> VcxResult<Vec<Revoke>> {
        let mut messages = Vec::<Revoke>::new();
        for (uid, message) in connection.get_messages_noauth(&agency_client).await?.into_iter() {
            if let AriesMessage::Revocation(Revocation::Revoke(message)) = message {
                connection.update_message_status(&uid, &agency_client).await.ok();
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
        for (uid, message) in connection.get_messages_noauth(&agency_client).await?.into_iter() {
            if let AriesMessage::Revocation(Revocation::Ack(message)) = message {
                connection.update_message_status(&uid, &agency_client).await.ok();
                messages.push(message);
            }
        }
        Ok(messages)
    }
}
