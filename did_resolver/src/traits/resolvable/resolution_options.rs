use crate::shared_types::media_type::MediaType;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DIDResolutionOptions {
    accept: Option<MediaType>,
}

impl DIDResolutionOptions {
    pub fn new() -> Self {
        Self { accept: None }
    }

    pub fn set_accept(mut self, accept: String) -> Self {
        self.accept = Some(accept);
        self
    }

    pub fn accept(&self) -> Option<&String> {
        self.accept.as_ref()
    }
}
