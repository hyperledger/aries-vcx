use aries_askar::{
    storage::{Argon2Level, KdfMethod},
    StoreKeyMethod,
};
use serde::Deserialize;

#[derive(Debug, Copy, Clone, Deserialize)]
pub enum KeyMethod {
    DeriveKey { inner: AskarKdfMethod },
    RawKey,
    Unprotected,
}

impl From<KeyMethod> for StoreKeyMethod {
    fn from(value: KeyMethod) -> Self {
        match value {
            KeyMethod::DeriveKey { inner: payload } => StoreKeyMethod::DeriveKey(payload.into()),
            KeyMethod::RawKey => StoreKeyMethod::RawKey,
            KeyMethod::Unprotected => StoreKeyMethod::Unprotected,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum AskarKdfMethod {
    Argon2i { inner: ArgonLevel },
}

impl From<AskarKdfMethod> for KdfMethod {
    fn from(value: AskarKdfMethod) -> Self {
        match value {
            AskarKdfMethod::Argon2i { inner: payload } => KdfMethod::Argon2i(payload.into()),
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum ArgonLevel {
    Interactive,
    Moderate,
}

impl From<ArgonLevel> for Argon2Level {
    fn from(value: ArgonLevel) -> Self {
        match value {
            ArgonLevel::Interactive => Argon2Level::Interactive,
            ArgonLevel::Moderate => Argon2Level::Moderate,
        }
    }
}
