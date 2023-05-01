use crate::shared_types::media_type::MediaType;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DIDDereferencingOptions {
    accept: Option<MediaType>,
}

impl DIDDereferencingOptions {
    pub fn new() -> Self {
        Self { accept: None }
    }

    pub fn set_accept(mut self, accept: MediaType) -> Self {
        self.accept = Some(accept);
        self
    }

    pub fn accept(&self) -> Option<&MediaType> {
        self.accept.as_ref()
    }
}
