mod command;
mod list_connections;

use std::sync::{Arc, RwLock};

use anyhow::Context;
use aries_vcx_agent::aries_vcx::messages::connection::invite::Invitation;
use inquire::{Select, Text};

use crate::agent::CliAriesAgent;

use self::{command::{get_options, ConnectionCommand}, list_connections::list_connections_command_loop};

use super::LoopStatus;

async fn process_connection_command(agent: Arc<RwLock<CliAriesAgent>>) -> anyhow::Result<LoopStatus> {
    match Select::new("Select command:", get_options()).prompt()?.clone() {
        ConnectionCommand::CreateInvite => {
            let invite = agent
                .read()
                .unwrap()
                .agent()
                .connections()
                .create_invitation()
                .await
                .map_err(|err| anyhow!("Error creating invitation: {}", err))?;
            println!("{}", json!(invite).to_string());
            Ok(LoopStatus::Continue)
        }
        ConnectionCommand::ReceiveInvite => {
            let s = Text::new("Enter invite:\n").prompt()?;
            let invite: Invitation = serde_json::from_str(&s).context("Failed to deserialize invite")?;
            let tid = agent
                .read()
                .unwrap()
                .agent()
                .connections()
                .receive_invitation(invite)
                .await
                .map_err(|err| anyhow!("Error receiving invitation: {}", err))?;
            agent
                .read()
                .unwrap()
                .agent()
                .connections()
                .send_request(&tid)
                .await
                .map_err(|err| anyhow!("Error sending request: {}", err))?;
            Ok(LoopStatus::Continue)
        }
        ConnectionCommand::ListConnections => list_connections_command_loop(agent).await,
        ConnectionCommand::GoBack => Ok(LoopStatus::GoBack),
    }
}

pub async fn connection_command_loop(agent: Arc<RwLock<CliAriesAgent>>) -> anyhow::Result<LoopStatus> {
    loop {
        match process_connection_command(agent.clone()).await {
            Ok(LoopStatus::Continue) => continue,
            Ok(LoopStatus::GoBack) => break,
            Ok(LoopStatus::Exit) => break,
            Err(err) => {
                error!("Error processing connection command: {}", err);
                break;
            }
        }
    }
    Ok(LoopStatus::Continue)
}
