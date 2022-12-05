mod command;

use std::sync::{Arc, RwLock};

use inquire::Select;

use crate::{agent::CliAriesAgent, commands::LoopStatus};

use self::command::{ListConnectionsCommand, get_options};

async fn process_list_connections_command(agent: Arc<RwLock<CliAriesAgent>>) -> anyhow::Result<LoopStatus> {
    let thread_ids = agent.read().unwrap().agent().connections().get_all_thread_ids()
        .map_err(|err| anyhow!("Error getting thread ids: {}", err))?;
    match Select::new("Select connection:", get_options(thread_ids)).prompt()? {
        ListConnectionsCommand::ThreadId(tid) => {
            let state = agent
                .read()
                .unwrap()
                .agent()
                .connections()
                .get_state(&tid)
                .map_err(|err| anyhow!("Error getting connection: {}", err))?;
            println!("{:?}", state);
            Ok(LoopStatus::Continue)
        }
        ListConnectionsCommand::GoBack => Ok(LoopStatus::GoBack),
    }
}

pub async fn list_connections_command_loop(agent: Arc<RwLock<CliAriesAgent>>) -> anyhow::Result<LoopStatus> {
    loop {
        match process_list_connections_command(agent.clone()).await {
            Ok(LoopStatus::Continue) => continue,
            Ok(LoopStatus::GoBack) => break,
            Ok(LoopStatus::Exit) => break,
            Err(err) => {
                error!("Error processing lit connections command: {}", err);
                continue;
            }
        }
    }
    Ok(LoopStatus::Continue)
}
