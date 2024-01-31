use aries_vcx_core::wallet::base_wallet::BaseWallet;
use messages::msg_fields::protocols::connection::Connection;

use super::{unhandled_aries_message, utils::prelude::*, ArcAgent};

pub async fn handle_aries_connection<T: BaseWallet + 'static, P: MediatorPersistence>(
    agent: ArcAgent<T, P>,
    connection: Connection,
) -> Result<EncryptionEnvelope, String> {
    match connection {
        Connection::Invitation(_invite) => {
            Err("Mediator does not handle random invites. Sorry.".to_owned())
        }
        Connection::Request(register_request) => {
            agent.handle_connection_req(register_request).await
        }
        _ => Err(unhandled_aries_message(connection)),
    }
}
