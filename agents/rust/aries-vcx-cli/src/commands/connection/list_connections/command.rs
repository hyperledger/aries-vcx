use std::fmt::Display;

#[derive(Clone)]
pub enum ListConnectionsCommand {
    ThreadId(String),
    GoBack,
}

impl Display for ListConnectionsCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ThreadId(tid) => f.write_str(&tid),
            Self::GoBack => f.write_str("Back"),
        }
    }
}

pub fn get_options(tids: Vec<String>) -> Vec<ListConnectionsCommand> {
    tids.iter()
        .map(|tid| ListConnectionsCommand::ThreadId(tid.to_string()))
        .chain(std::iter::once(ListConnectionsCommand::GoBack))
        .collect()
}
