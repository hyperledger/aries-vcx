extern crate display_as_json;

pub mod error;
pub mod extra_fields;
pub mod service;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::schema::{types::uri::Uri, utils::OneOrList};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TypedService<E> {
    id: Uri,
    #[serde(rename = "type")]
    service_type: OneOrList<String>,
    service_endpoint: Url,
    #[serde(flatten)]
    extra: E,
}

impl<E> TypedService<E> {
    pub fn id(&self) -> &Uri {
        &self.id
    }

    pub fn service_type(&self) -> &OneOrList<String> {
        &self.service_type
    }

    pub fn service_endpoint(&self) -> &Url {
        &self.service_endpoint
    }

    pub fn extra(&self) -> &E {
        &self.extra
    }
}
