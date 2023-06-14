use crate::shared_types::media_type::MediaType;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DidResolutionOptions<E: Default> {
    accept: Option<MediaType>,
    extra: E,
}

impl<E: Default> DidResolutionOptions<E> {
    pub fn new() -> Self {
        Self {
            accept: None,
            extra: E::default(),
        }
    }

    pub fn set_accept(mut self, accept: MediaType) -> Self {
        self.accept = Some(accept);
        self
    }

    pub fn set_extra(mut self, extra: E) -> Self {
        self.extra = extra;
        self
    }

    pub fn accept(&self) -> Option<&MediaType> {
        self.accept.as_ref()
    }

    pub fn extra(&self) -> &E {
        &self.extra
    }
}
