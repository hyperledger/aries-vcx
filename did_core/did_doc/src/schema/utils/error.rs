use std::fmt::{self, Display, Formatter};

use thiserror::Error;

#[derive(Debug, Error)]
pub struct DidDocumentLookupError {
    reason: String,
}

impl DidDocumentLookupError {
    pub fn new(reason: String) -> Self {
        Self { reason }
    }
}

impl Display for DidDocumentLookupError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DiddocLookupError: {}", self.reason)
    }
}
