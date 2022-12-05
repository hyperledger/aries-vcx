use std::fmt::Display;

use clap::ValueEnum;
use serde::Deserialize;

#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug, Clone, ValueEnum)]
#[clap(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KeyDerivationMethod {
    RAW,
    ARGON2I_MOD,
    ARGON2I_INT,
}

impl Display for KeyDerivationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RAW => f.write_str("RAW"),
            Self::ARGON2I_MOD => f.write_str("ARGON2I_MOD"),
            Self::ARGON2I_INT => f.write_str("ARGON2I_INT"),
        }
    }
}
