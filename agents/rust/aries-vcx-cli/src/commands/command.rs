use std::fmt::Display;

pub enum Command {
    Ledger,
    Connections,
    Messages,
    Exit,
}

pub enum ConfirmExit {
    Yes,
    No,
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ledger => f.write_str("Ledger"),
            Self::Connections => f.write_str("Connections"),
            Self::Messages => f.write_str("Messages"),
            Self::Exit => f.write_str("Exit"),
        }
    }
}

impl Display for ConfirmExit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Yes => f.write_str("Yes"),
            Self::No => f.write_str("No"),
        }
    }
}

impl Command {
    pub fn iter() -> impl Iterator<Item = &'static Command> {
        [Self::Ledger, Self::Connections, Self::Messages, Self::Exit].iter()
    }
}

pub fn get_options() -> Vec<&'static Command> {
    Command::iter().collect()
}

pub enum LoopStatus {
    Continue,
    Exit,
    GoBack,
}
