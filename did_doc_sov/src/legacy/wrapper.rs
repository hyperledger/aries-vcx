use serde::{Deserialize, Serialize};
use serde_json::Value;

use did_doc::{
    did_parser::{Did, DidUrl},
    schema::{
        did_doc::{ControllerAlias, DidDocument},
        verification_method::{VerificationMethod, VerificationMethodKind},
    },
};

use crate::{extra_fields::ExtraFieldsSov, service::ServiceSov};

use super::LegacyDidDoc;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum LegacyOrNew {
    Legacy(LegacyDidDoc),
    New(DidDocument<ExtraFieldsSov>),
}

impl LegacyOrNew {
    pub fn id(&self) -> &Did {
        match self {
            LegacyOrNew::Legacy(ddo) => todo!(),
            LegacyOrNew::New(ddo) => ddo.id(),
        }
    }

    pub fn controller(&self) -> Option<&ControllerAlias> {
        match self {
            LegacyOrNew::Legacy(ddo) => todo!(),
            LegacyOrNew::New(ddo) => ddo.controller(),
        }
    }

    pub fn verification_method(&self) -> &[VerificationMethod] {
        match self {
            LegacyOrNew::Legacy(ddo) => todo!(),
            LegacyOrNew::New(ddo) => ddo.verification_method(),
        }
    }

    pub fn authentication(&self) -> &[VerificationMethodKind] {
        match self {
            LegacyOrNew::Legacy(ddo) => todo!(),
            LegacyOrNew::New(ddo) => ddo.authentication(),
        }
    }

    pub fn service(&self) -> &[ServiceSov] {
        todo!()
    }

    pub fn assertion_method(&self) -> &[VerificationMethodKind] {
        match self {
            LegacyOrNew::Legacy(ddo) => todo!(),
            LegacyOrNew::New(ddo) => ddo.assertion_method(),
        }
    }

    pub fn key_agreement(&self) -> &[VerificationMethodKind] {
        match self {
            LegacyOrNew::Legacy(ddo) => todo!(),
            LegacyOrNew::New(ddo) => ddo.key_agreement(),
        }
    }

    pub fn capability_invocation(&self) -> &[VerificationMethodKind] {
        match self {
            LegacyOrNew::Legacy(ddo) => todo!(),
            LegacyOrNew::New(ddo) => ddo.capability_invocation(),
        }
    }

    pub fn capability_delegation(&self) -> &[VerificationMethodKind] {
        match self {
            LegacyOrNew::Legacy(ddo) => todo!(),
            LegacyOrNew::New(ddo) => ddo.capability_delegation(),
        }
    }

    pub fn extra_field(&self, key: &str) -> Option<&Value> {
        match self {
            LegacyOrNew::Legacy(ddo) => todo!(),
            LegacyOrNew::New(ddo) => ddo.extra_field(key),
        }
    }

    pub fn dereference_key(&self, reference: &DidUrl) -> Option<&VerificationMethod> {
        match self {
            LegacyOrNew::Legacy(ddo) => todo!(),
            LegacyOrNew::New(ddo) => ddo.dereference_key(reference),
        }
    }
}
