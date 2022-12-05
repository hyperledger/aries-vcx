mod command;
mod connection;
mod ledger;
mod messages;

use std::sync::{Arc, RwLock};

use inquire::Select;

use crate::agent::CliAriesAgent;

pub use self::command::LoopStatus;
use self::{
    command::{get_options, Command, ConfirmExit},
    connection::connection_command_loop,
    ledger::ledger_command_loop,
    messages::messages_command_loop,
};

pub async fn process_root_command(agent: Arc<RwLock<CliAriesAgent>>) -> anyhow::Result<LoopStatus> {
    match Select::new("Select command:", get_options()).prompt()?.clone() {
        Command::Ledger => ledger_command_loop(agent).await,
        Command::Connections => connection_command_loop(agent).await,
        Command::Messages => messages_command_loop(agent).await,
        Command::Exit => Ok(LoopStatus::Exit),
    }
}

pub async fn process_exit_command() -> anyhow::Result<LoopStatus> {
    match Select::new(
        "Are you sure you want to exit?",
        vec![ConfirmExit::Yes, ConfirmExit::No],
    )
    .prompt()?
    {
        ConfirmExit::Yes => Ok(LoopStatus::Exit),
        ConfirmExit::No => Ok(LoopStatus::Continue),
    }
}

pub async fn root_command_loop(agent: Arc<RwLock<CliAriesAgent>>) -> Result<(), std::io::Error> {
    loop {
        match process_root_command(agent.clone()).await {
            Ok(LoopStatus::Continue) => continue,
            Ok(LoopStatus::Exit) | Ok(LoopStatus::GoBack) => match process_exit_command().await {
                Ok(LoopStatus::Continue) => continue,
                Ok(LoopStatus::Exit) | Ok(LoopStatus::GoBack) => break Ok(()),
                Err(err) => {
                    error!("Error processing exit command: {}", err);
                    break Ok(());
                }
            },
            Err(err) => {
                error!("An error occurred inside user input loop: {}", err);
                continue;
            }
        }
    }
}
