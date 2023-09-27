use std::error::Error;

pub use log::{debug, error, info};

// Generic Wrapper struct for newtype pattern
// for implementing external trait on external types
pub struct Wrapper<T>(pub T);

// Generic Result
pub type Result_<T> = Result<T, Box<dyn Error>>;
