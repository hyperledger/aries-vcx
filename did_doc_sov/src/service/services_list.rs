use crate::error::DidDocumentSovError;

use super::ServiceSov;

pub struct ServicesList {
    services: Vec<ServiceSov>,
}

impl ServicesList {
    pub fn new(services: Vec<ServiceSov>) -> Self {
        Self { services }
    }

    pub fn get(&self, index: usize) -> Result<&ServiceSov, DidDocumentSovError> {
        self.services
            .get(index)
            .ok_or(DidDocumentSovError::IndexOutOfBounds(index))
    }

    pub fn len(&self) -> usize {
        self.services.len()
    }
}
