pub mod error;
pub mod extra_fields;
pub mod service;

use did_doc::{
    did_parser::{Did, DidUrl},
    schema::{
        did_doc::{ControllerAlias, DidDocument, DidDocumentBuilder},
        types::uri::Uri,
        verification_method::{VerificationMethod, VerificationMethodKind},
    },
};
use extra_fields::ExtraFieldsSov;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use service::ServiceSov;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct DidDocumentSov {
    did_doc: DidDocument<ExtraFieldsSov>,
    services: Vec<ServiceSov>,
}

impl DidDocumentSov {
    pub fn builder() -> DidDocumentSovBuilder {
        DidDocumentSovBuilder::default()
    }

    pub fn id(&self) -> &Did {
        self.did_doc.id()
    }

    pub fn also_known_as(&self) -> &[Uri] {
        self.did_doc.also_known_as()
    }

    pub fn controller(&self) -> Option<&ControllerAlias> {
        self.did_doc.controller()
    }

    pub fn verification_method(&self) -> &[VerificationMethod] {
        self.did_doc.verification_method()
    }

    pub fn authentication(&self) -> &[VerificationMethodKind] {
        self.did_doc.authentication()
    }

    pub fn service(&self) -> &[ServiceSov] {
        self.services.as_ref()
    }

    pub fn assertion_method(&self) -> &[VerificationMethodKind] {
        self.did_doc.assertion_method()
    }

    pub fn key_agreement(&self) -> &[VerificationMethodKind] {
        self.did_doc.key_agreement()
    }

    pub fn capability_invocation(&self) -> &[VerificationMethodKind] {
        self.did_doc.capability_invocation()
    }

    pub fn capability_delegation(&self) -> &[VerificationMethodKind] {
        self.did_doc.capability_delegation()
    }

    pub fn extra_field(&self, key: &str) -> Option<&Value> {
        self.did_doc.extra_field(key)
    }

    pub fn dereference_key(&self, reference: &DidUrl) -> Option<&VerificationMethod> {
        self.did_doc.dereference_key(reference)
    }
}

#[derive(Default)]
pub struct DidDocumentSovBuilder {
    ddo_builder: DidDocumentBuilder<ExtraFieldsSov>,
    services: Vec<ServiceSov>,
}

impl DidDocumentSovBuilder {
    pub fn new(id: Did) -> Self {
        Self {
            ddo_builder: DidDocumentBuilder::new(id),
            services: Vec::new(),
        }
    }

    pub fn add_controller(mut self, controller: Did) -> Self {
        self.ddo_builder = self.ddo_builder.add_controller(controller);
        self
    }

    pub fn add_verification_method(mut self, verification_method: VerificationMethod) -> Self {
        self.ddo_builder = self.ddo_builder.add_verification_method(verification_method);
        self
    }

    pub fn add_service(mut self, service: ServiceSov) -> Self {
        self.services.push(service);
        self
    }

    pub fn build(self) -> DidDocumentSov {
        DidDocumentSov {
            did_doc: self.ddo_builder.build(),
            services: self.services,
        }
    }
}

impl<'de> Deserialize<'de> for DidDocumentSov {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Clone, Debug, PartialEq)]
        struct TempDidDocumentSov {
            #[serde(flatten)]
            did_doc: DidDocument<ExtraFieldsSov>,
        }

        let temp = TempDidDocumentSov::deserialize(deserializer)?;

        let services = temp
            .did_doc
            .service()
            .iter()
            .map(|s| ServiceSov::try_from(s.clone()))
            .collect::<Result<Vec<ServiceSov>, _>>()
            .map_err(|_| de::Error::custom("Failed to convert service"))?;

        Ok(DidDocumentSov {
            did_doc: temp.did_doc,
            services,
        })
    }
}
