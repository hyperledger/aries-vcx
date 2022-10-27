pub mod receiver;
pub mod sender;

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use agency_client::agency_client::AgencyClient;
    use messages::ack::Ack;
    use messages::revocation_notification::revocation_notification::RevocationNotification;

    use crate::error::prelude::*;
    use crate::handlers::connection::connection::Connection;
    use messages::a2a::A2AMessage;

    pub async fn get_revocation_notification_messages(
        agency_client: &AgencyClient,
        connection: &Connection,
    ) -> VcxResult<Vec<RevocationNotification>> {
        let mut messages = Vec::<RevocationNotification>::new();
        for (uid, message) in connection.get_messages_noauth(&agency_client).await?.into_iter() {
            if let A2AMessage::RevocationNotification(message) = message {
                connection.update_message_status(&uid, &agency_client).await.ok();
                messages.push(message);
            }
        }
        Ok(messages)
    }

    pub async fn get_revocation_notification_ack_messages(
        agency_client: &AgencyClient,
        connection: &Connection,
    ) -> VcxResult<Vec<Ack>> {
        let mut messages = Vec::<Ack>::new();
        for (uid, message) in connection.get_messages_noauth(&agency_client).await?.into_iter() {
            if let A2AMessage::RevocationAck(message) = message {
                connection.update_message_status(&uid, &agency_client).await.ok();
                messages.push(message);
            }
        }
        Ok(messages)
    }
}
