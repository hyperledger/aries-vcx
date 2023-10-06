use std::collections::HashMap;

use ursa::cl::{RevocationRegistry, Witness};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RevocationState {
    pub witness: Witness,
    pub rev_reg: RevocationRegistry,
    pub timestamp: u64,
}

pub type RevocationStates = HashMap<String, HashMap<u64, RevocationState>>;
