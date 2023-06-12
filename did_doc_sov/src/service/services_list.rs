use did_doc::schema::service::Service;

use crate::{error::DidDocumentSovError, extra_fields::ExtraFieldsSov};

use super::ServiceSov;

pub struct ServicesList<'a> {
    services: &'a [Service<ExtraFieldsSov>],
}

impl<'a> ServicesList<'a> {
    pub fn new(services: &'a [Service<ExtraFieldsSov>]) -> Self {
        Self { services }
    }

    pub fn get(&self, index: usize) -> Result<ServiceSov, DidDocumentSovError> {
        self.services
            .get(index)
            .ok_or(DidDocumentSovError::IndexOutOfBounds(index))
            .and_then(|service| ServiceSov::try_from(service.to_owned()))
    }

    pub fn len(&self) -> usize {
        self.services.len()
    }
}
