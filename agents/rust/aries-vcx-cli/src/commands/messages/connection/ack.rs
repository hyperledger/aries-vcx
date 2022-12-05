use std::{fmt::Display, sync::{RwLock, Arc}};

use aries_vcx_agent::aries_vcx::messages::ack::Ack;
use inquire::Select;

use crate::{commands::LoopStatus, agent::CliAriesAgent};

pub enum ConnectionAckMessageCommand {
    Process,
    GoBack
}

impl Display for ConnectionAckMessageCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Process => f.write_str("Process"),
            Self::GoBack => f.write_str("Back"),
        }
    }
}

impl ConnectionAckMessageCommand {
    pub fn iter() -> impl Iterator<Item = &'static ConnectionAckMessageCommand> {
        [
            Self::Process,
            Self::GoBack,
        ]
        .iter()
    }
}

pub async fn process_connection_ack_message_command(agent: Arc<RwLock<CliAriesAgent>>, ack: Ack) -> anyhow::Result<LoopStatus> {
    match Select::new("Select command:", ConnectionAckMessageCommand::iter().collect()).prompt()? {
        ConnectionAckMessageCommand::Process => {
            let tid = ack.get_thread_id();
            agent
                .read().unwrap()
                .agent()
                .connections()
                .process_ack(&tid, ack)
                .await
                .map_err(|err| anyhow!("Error processing ack: {}", err))?;
            Ok(LoopStatus::Continue)
        }
        ConnectionAckMessageCommand::GoBack => {
            Ok(LoopStatus::GoBack)
        }
    }
}

