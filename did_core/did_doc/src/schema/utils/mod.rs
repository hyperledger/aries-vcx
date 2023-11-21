use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum OneOrList<T> {
    One(T),
    List(Vec<T>),
}

impl<T: Display + Debug> Display for OneOrList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OneOrList::One(t) => write!(f, "{}", t),
            OneOrList::List(t) => write!(f, "{:?}", t),
        }
    }
}
