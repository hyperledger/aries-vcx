mod command;

use std::sync::{Arc, RwLock};

use aries_vcx_agent::aries_vcx::common::primitives::credential_definition::CredentialDefConfigBuilder;
use inquire::{Select, Text};

use crate::agent::CliAriesAgent;

use self::command::{get_options, LedgerCommand};

use super::LoopStatus;

async fn process_ledger_command(agent: Arc<RwLock<CliAriesAgent>>) -> anyhow::Result<LoopStatus> {
    match Select::new("Select command:", get_options()).prompt()?.clone() {
        LedgerCommand::CreateSchema => {
            let schema_name = Text::new("Schema name:").prompt()?;
            let schema_version = Text::new("Schema version:").prompt()?;
            let schema_attrs = Text::new("Schema attributes (comma separated):").prompt()?;
            let schema_attrs = schema_attrs.split(",").map(str::to_string).collect();
            let schema_id = agent
                .read()
                .unwrap()
                .agent()
                .schemas()
                .create_schema(&schema_name, &schema_version, &schema_attrs)
                .await
                .map_err(|err| anyhow!("Failed to create schema: {}", err))?;
            println!("Schema created with id: {}", schema_id);
            agent
                .read()
                .unwrap()
                .agent()
                .schemas()
                .publish_schema(&schema_id)
                .await
                .map_err(|err| anyhow!("Failed to publish schema: {}", err))?;
            println!("Schema published with id: {}", schema_id);
            Ok(LoopStatus::Continue)
        }
        LedgerCommand::CreateCredDef => {
            let schema_id = Text::new("Schema id:").prompt()?;
            let tag = Text::new("Tag:").prompt()?;
            let config = CredentialDefConfigBuilder::default()
                .issuer_did(
                    agent
                        .read()
                        .unwrap()
                        .agent()
                        .agent_config()
                        .config_issuer
                        .institution_did
                        .clone(),
                )
                .schema_id(&schema_id)
                .tag(tag)
                .build()
                .map_err(|err| anyhow!("Failed to build credential def config: {}", err))?;
            let cred_def_id = agent
                .read()
                .unwrap()
                .agent()
                .cred_defs()
                .create_cred_def(config)
                .await
                .map_err(|err| anyhow!("Failed to create credential definition: {}", err))?;
            println!("Credential definition created with id: {}", cred_def_id);
            agent
                .read()
                .unwrap()
                .agent()
                .cred_defs()
                .publish_cred_def(&cred_def_id)
                .await
                .map_err(|err| anyhow!("Failed to publish credential definition: {}", err))?;
            println!("Credential definition with id published: {}", cred_def_id);
            Ok(LoopStatus::Continue)
        }
        LedgerCommand::GoBack => Ok(LoopStatus::GoBack),
    }
}

pub async fn ledger_command_loop(agent: Arc<RwLock<CliAriesAgent>>) -> anyhow::Result<LoopStatus> {
    loop {
        match process_ledger_command(agent.clone()).await {
            Ok(LoopStatus::Continue) => continue,
            Ok(LoopStatus::GoBack) => break,
            Ok(LoopStatus::Exit) => break,
            Err(err) => {
                error!("Error processing ledger command: {}", err);
                break;
            }
        }
    }
    Ok(LoopStatus::Continue)
}
