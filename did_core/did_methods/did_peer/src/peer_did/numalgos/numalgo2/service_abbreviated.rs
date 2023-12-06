use did_doc::schema::{
    service::extra_fields::{ServiceAcceptType, ServiceKeyKind},
    utils::OneOrList,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct ServiceAbbreviatedDidPeer2 {
    // https://identity.foundation/peer-did-method-spec/#generating-a-didpeer2
    //   > For use with did:peer:2, service id attributes MUST be relative.
    //   > The service MAY omit the id; however, this is NOT RECOMMEDED (clarified).
    id: Option<String>,
    #[serde(rename = "t")]
    service_type: OneOrList<String>,
    #[serde(rename = "s")]
    service_endpoint: Url,
    #[serde(rename = "r")]
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    routing_keys: Vec<ServiceKeyKind>,
    #[serde(rename = "a")]
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    accept: Vec<ServiceAcceptType>,
}

impl ServiceAbbreviatedDidPeer2 {
    pub fn new(
        id: Option<String>,
        service_type: OneOrList<String>,
        service_endpoint: Url,
        routing_keys: Vec<ServiceKeyKind>,
        accept: Vec<ServiceAcceptType>,
    ) -> Self {
        Self {
            id,
            service_type,
            service_endpoint,
            routing_keys,
            accept,
        }
    }

    pub fn service_type(&self) -> &OneOrList<String> {
        &self.service_type
    }

    pub fn service_endpoint(&self) -> &Url {
        &self.service_endpoint
    }

    pub fn routing_keys(&self) -> &[ServiceKeyKind] {
        &self.routing_keys
    }

    pub fn accept(&self) -> &[ServiceAcceptType] {
        &self.accept
    }
}
