use std::collections::HashMap;
use serde::Deserialize;
use serde_json::Value;

use crate::shared_types::media_type::MediaType;

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct DidResolutionOptions {
    accept: Option<MediaType>,
    extra: HashMap<String, Value>
}

impl DidResolutionOptions {
    pub fn new(extra: HashMap<String, Value>) -> Self {
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

    pub fn extra(&self) -> &HashMap<String, Value> {
        &self.extra
    }
}
