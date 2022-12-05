mod command;
mod connection;

use std::sync::{Arc, RwLock};

use inquire::Select;

use crate::agent::CliAriesAgent;

use self::{
    command::{get_messages, MessagesCommand},
    connection::{
        ack::process_connection_ack_message_command, request::process_connection_request_message_command,
        response::process_connection_response_message_command,
    },
};

use super::LoopStatus;

async fn process_messages_command(agent: Arc<RwLock<CliAriesAgent>>) -> anyhow::Result<LoopStatus> {
    match Select::new("Select message:", get_messages(&agent)).prompt()? {
        MessagesCommand::ConnectionRequest(request) => process_connection_request_message_command(agent, request).await,
        MessagesCommand::ConnectionResponse(response) => {
            process_connection_response_message_command(agent, response).await
        }
        MessagesCommand::Ack(ack) => process_connection_ack_message_command(agent, ack).await,
        MessagesCommand::GoBack => Ok(LoopStatus::GoBack),
        _ => {
            info!("Not implemented yet");
            Ok(LoopStatus::Continue)
        }
    }
}

pub async fn messages_command_loop(agent: Arc<RwLock<CliAriesAgent>>) -> anyhow::Result<LoopStatus> {
    loop {
        match process_messages_command(agent.clone()).await {
            Ok(LoopStatus::Continue) => continue,
            Ok(LoopStatus::Exit) => break,
            Ok(LoopStatus::GoBack) => break,
            Err(err) => {
                error!("Error processing messages command: {}", err);
                break;
            }
        }
    }
    Ok(LoopStatus::Continue)
}
