use std::fmt::Display;

use thiserror::Error as ThisError;

pub type MediatorClientResult<T> = Result<T, MediatorClientError>;

#[derive(Debug, ThisError)]
pub enum MediatorClientError {
    UrlError(#[from] url::ParseError),
    RequestError(#[from] reqwest::Error),
}

impl Display for MediatorClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UrlError(inner) => write!(f, "{}", inner),
            Self::RequestError(inner) => write!(f, "{}", inner),
        }
    }
}
