use std::{fmt::Display, sync::{RwLock, Arc}};

use aries_vcx_agent::aries_vcx::messages::connection::request::Request as ConnectionRequest;
use inquire::Select;

use crate::{commands::LoopStatus, agent::CliAriesAgent};

pub enum ConnectionRequestMessageCommand {
    Accept,
    Decline,
    GoBack
}

impl Display for ConnectionRequestMessageCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Accept => f.write_str("Accept"),
            Self::Decline => f.write_str("Decline"),
            Self::GoBack => f.write_str("Back"),
        }
    }
}

impl ConnectionRequestMessageCommand {
    pub fn iter() -> impl Iterator<Item = &'static ConnectionRequestMessageCommand> {
        [
            Self::Accept,
            Self::Decline,
            Self::GoBack,
        ]
        .iter()
    }
}

pub async fn process_connection_request_message_command(agent: Arc<RwLock<CliAriesAgent>>, request: ConnectionRequest) -> anyhow::Result<LoopStatus> {
    match Select::new("Select command:", ConnectionRequestMessageCommand::iter().collect()).prompt()? {
        ConnectionRequestMessageCommand::Accept => {
            let tid = request.get_thread_id();
            agent
                .read().unwrap()
                .agent()
                .connections()
                .accept_request(&tid, request)
                .await
                .map_err(|err| anyhow!("Error accepting request: {}", err))?;
            agent
                .read().unwrap()
                .agent()
                .connections()
                .send_response(&tid)
                .await
                .map_err(|err| anyhow!("Error sending response: {}", err))?;
            Ok(LoopStatus::Continue)
        }
        ConnectionRequestMessageCommand::Decline => {
            // TODO: Implement
            Ok(LoopStatus::Continue)
        }
        ConnectionRequestMessageCommand::GoBack => {
            Ok(LoopStatus::GoBack)
        }
    }
}
