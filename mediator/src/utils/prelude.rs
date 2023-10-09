// use std::error::Error;

pub use log::{debug, error, info};

// Generic Wrapper struct for newtype pattern
// for implementing external trait on external types
// pub struct Wrapper<T>(pub T);

// Generic Result
// pub type Result_<T> = Result<T, Box<dyn Error>>;

// pub fn string_from_std_error(err: impl std::error::Error) -> String {
//     err.to_string()
// }

#[derive(thiserror::Error, Debug)]
#[error("{msg}")]
pub struct GenericStringError {
    pub msg: String,
}
