use clap::{arg, command, Parser, ValueHint};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct AppArgs {
    #[arg(long)]
    port: Option<u32>,
    #[arg(long)]
    log_level: Option<String>,
    #[arg(long, value_hint = ValueHint::FilePath, default_value = "config/localhost.toml")]
    config_file: String,
    #[arg(long)]
    agent_name: Option<String>,
}

impl AppArgs {
    pub fn port(&self) -> Option<u32> {
        self.port
    }

    pub fn log_level(&self) -> Option<String> {
        self.log_level.clone()
    }

    pub fn config_file(&self) -> String {
        self.config_file.clone()
    }

    pub fn agent_name(&self) -> Option<String> {
        self.agent_name.clone()
    }
}

pub fn parse_cli_args() -> AppArgs {
    AppArgs::parse()
}
