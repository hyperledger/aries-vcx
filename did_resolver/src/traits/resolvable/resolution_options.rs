use serde::Deserialize;

use crate::shared_types::media_type::MediaType;

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct DidResolutionOptions<E> {
    accept: Option<MediaType>,
    extra: E,
}

impl<E> DidResolutionOptions<E> {
    pub fn new(extra: E) -> Self {
        Self {
            accept: None,
            extra,
        }
    }

    pub fn set_accept(mut self, accept: MediaType) -> Self {
        self.accept = Some(accept);
        self
    }

    pub fn accept(&self) -> Option<&MediaType> {
        self.accept.as_ref()
    }

    pub fn extra(&self) -> &E {
        &self.extra
    }
}
