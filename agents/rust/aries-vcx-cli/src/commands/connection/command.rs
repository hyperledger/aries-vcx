use std::fmt::Display;

#[derive(Clone)]
pub enum ConnectionCommand {
    CreateInvite,
    ReceiveInvite,
    ListConnections,
    GoBack,
}

impl Display for ConnectionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreateInvite => f.write_str("Create Invite"),
            Self::ReceiveInvite => f.write_str("Receive Invite"),
            Self::ListConnections => f.write_str("List Connections"),
            Self::GoBack => f.write_str("Back"),
        }
    }
}

impl ConnectionCommand {
    pub fn iter() -> impl Iterator<Item = &'static ConnectionCommand> {
        [
            Self::CreateInvite,
            Self::ReceiveInvite,
            Self::ListConnections,
            Self::GoBack,
        ]
        .iter()
    }
}

pub fn get_options() -> Vec<&'static ConnectionCommand> {
    ConnectionCommand::iter().collect()
}
