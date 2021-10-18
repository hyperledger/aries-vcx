use crate::error::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct InitialProverState {}

impl InitialProverState {
    pub fn new() -> Self {
        Self {}
    }
}

