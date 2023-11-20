extern crate display_as_json;

pub mod error;
pub mod extra_fields;
// TODO: Remove once migration is done
mod legacy;
pub mod service;

use std::collections::HashMap;

use did_doc::{
    did_parser::{Did, DidUrl},
    schema::{
        did_doc::{ControllerAlias, DidDocument, DidDocumentBuilder},
        service::Service,
        utils::OneOrList,
        verification_method::{VerificationMethod, VerificationMethodKind},
    },
};
use extra_fields::ExtraFieldsSov;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use service::ServiceSov;

#[derive(Clone, Debug, PartialEq)]
pub struct DidDocumentSov {
    did_doc: DidDocument<ExtraFieldsSov>,
    services: Vec<ServiceSov>,
}

impl DidDocumentSov {
    pub fn builder(id: Did) -> DidDocumentSovBuilder {
        DidDocumentSovBuilder::new(id)
    }

    pub fn id(&self) -> &Did {
        self.did_doc.id()
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

    pub fn resolved_key_agreement(&self) -> impl Iterator<Item = &VerificationMethod> {
        self.did_doc
            .key_agreement()
            .iter()
            .filter_map(|vm| match vm {
                VerificationMethodKind::Resolved(resolved) => Some(resolved),
                VerificationMethodKind::Resolvable(reference) => self.dereference_key(reference),
            })
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
        self.ddo_builder = self
            .ddo_builder
            .add_verification_method(verification_method);
        self
    }

    pub fn add_key_agreement(mut self, key_agreement: VerificationMethodKind) -> Self {
        match key_agreement {
            VerificationMethodKind::Resolved(ka) => {
                self.ddo_builder = self.ddo_builder.add_key_agreement(ka);
            }
            VerificationMethodKind::Resolvable(ka_ref) => {
                self.ddo_builder = self.ddo_builder.add_key_agreement_reference(ka_ref);
            }
        }
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
            // TODO: Remove once the transition is done
            #[serde(deserialize_with = "legacy::deserialize_legacy_or_new")]
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

impl Serialize for DidDocumentSov {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut builder: DidDocumentBuilder<ExtraFieldsSov> = self.did_doc.clone().into();

        for service_sov in &self.services {
            let service: Service<ExtraFieldsSov> = service_sov
                .clone()
                .try_into()
                .map_err(serde::ser::Error::custom)?;
            // Not very efficient, but
            // * we don't expect many services
            // * does not require allowing to remove services from existing DDO or builder
            if !self
                .did_doc
                .service()
                .iter()
                .any(|s| s.id() == service.id())
            {
                builder = builder.add_service(service);
            }
        }

        builder.build().serialize(serializer)
    }
}

impl From<DidDocumentSov> for DidDocument<ExtraFieldsSov> {
    fn from(ddo: DidDocumentSov) -> Self {
        let mut ddo_builder = DidDocument::<ExtraFieldsSov>::builder(ddo.did_doc.id().clone());
        for service in ddo.service() {
            ddo_builder = ddo_builder.add_service(service.clone().try_into().unwrap());
        }
        if let Some(controller) = ddo.did_doc.controller() {
            match controller {
                OneOrList::One(controller) => {
                    ddo_builder = ddo_builder.add_controller(controller.clone());
                }
                OneOrList::List(list) => {
                    for controller in list {
                        ddo_builder = ddo_builder.add_controller(controller.clone());
                    }
                }
            }
        }
        for vm in ddo.verification_method() {
            ddo_builder = ddo_builder.add_verification_method(vm.clone());
        }
        for ka in ddo.key_agreement() {
            match ka {
                VerificationMethodKind::Resolved(ka) => {
                    ddo_builder = ddo_builder.add_key_agreement(ka.clone());
                }
                VerificationMethodKind::Resolvable(ka_ref) => {
                    ddo_builder = ddo_builder.add_key_agreement_reference(ka_ref.clone());
                }
            }
        }
        ddo_builder.build()
    }
}

impl From<DidDocument<ExtraFieldsSov>> for DidDocumentSov {
    fn from(ddo: DidDocument<ExtraFieldsSov>) -> Self {
        let mut builder = DidDocumentSov::builder(ddo.id().clone());
        for service in ddo.service() {
            builder = builder.add_service(service.clone().try_into().unwrap());
        }
        for vm in ddo.verification_method() {
            builder = builder.add_verification_method(vm.clone());
        }
        for ka in ddo.key_agreement() {
            builder = builder.add_key_agreement(ka.clone());
        }
        // TODO: Controller
        builder.build()
    }
}

impl From<DidDocument<HashMap<String, Value>>> for DidDocumentSov {
    fn from(ddo: DidDocument<HashMap<String, Value>>) -> Self {
        let mut builder = DidDocumentSov::builder(ddo.id().clone());
        for service in ddo.service() {
            builder = builder.add_service(service.clone().try_into().unwrap());
        }
        for vm in ddo.verification_method() {
            builder = builder.add_verification_method(vm.clone());
        }
        for ka in ddo.key_agreement() {
            builder = builder.add_key_agreement(ka.clone());
        }
        builder.build()
    }
}
