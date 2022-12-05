use std::fmt::Display;

#[derive(Clone)]
pub enum LedgerCommand {
    CreateSchema,
    CreateCredDef,
    GoBack,
}

impl Display for LedgerCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreateSchema => f.write_str("Create Schema"),
            Self::CreateCredDef => f.write_str("Create Credential Definition"),
            Self::GoBack => f.write_str("Back"),
        }
    }
}

impl LedgerCommand {
    pub fn iter() -> impl Iterator<Item = &'static LedgerCommand> {
        [Self::CreateSchema, Self::CreateCredDef, Self::GoBack].iter()
    }
}

pub fn get_options() -> Vec<&'static LedgerCommand> {
    LedgerCommand::iter().collect()
}
