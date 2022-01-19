use indy_api_types::errors::IndyResult;

use super::super::super::super::proto::cheqdid::cheqdnode::cheqd::v1::Did as ProtoDid;
use super::super::super::super::CheqdProtoBase;
use super::{ VerificationMethod, Service };

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Did {
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub context: Vec<String>,
    pub id: String,
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub controller: Vec<String>,
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub verification_method: Vec<VerificationMethod>,
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub authentication: Vec<String>,
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub assertion_method: Vec<String>,
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub capability_invocation: Vec<String>,
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub capability_delegation: Vec<String>,
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub key_agreement: Vec<String>,
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub service: Vec<Service>,
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub also_known_as: Vec<String>,
}

impl Did {
    pub fn new(
        context: Vec<String>,
        id: String,
        controller: Vec<String>,
        verification_method: Vec<VerificationMethod>,
        authentication: Vec<String>,
        assertion_method: Vec<String>,
        capability_invocation: Vec<String>,
        capability_delegation: Vec<String>,
        key_agreement: Vec<String>,
        service: Vec<Service>,
        also_known_as: Vec<String>) -> Self {
        Did {
            context,
            id,
            controller,
            verification_method,
            authentication,
            assertion_method,
            capability_invocation,
            capability_delegation,
            key_agreement,
            service,
            also_known_as
        }
    }
}

impl CheqdProtoBase for Did {
    type Proto = ProtoDid;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(
            Self::Proto {
                context: self.context.to_proto()?,
                id: self.id.clone(),
                controller: self.controller.to_proto()?,
                verification_method: self.verification_method.to_proto()?,
                authentication: self.authentication.to_proto()?,
                assertion_method: self.assertion_method.to_proto()?,
                capability_invocation: self.capability_invocation.to_proto()?,
                capability_delegation: self.capability_delegation.to_proto()?,
                key_agreement: self.key_agreement.to_proto()?,
                service: self.service.to_proto()?,
                also_known_as: self.also_known_as.to_proto()?,
            }
        )
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(
            proto.context.clone(),
            proto.id.clone(),
            proto.controller.clone(),
            Vec::<VerificationMethod>::from_proto(&proto.verification_method)?,
            proto.authentication.clone(),
            proto.assertion_method.clone(),
            proto.capability_invocation.clone(),
            proto.capability_delegation.clone(),
            proto.key_agreement.clone(),
            Vec::<Service>::from_proto(&proto.service)?,
            proto.also_known_as.clone(),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_create_did() {
        let verification_method = VerificationMethod::new(
            "id".into(),
            "type".into(),
            "controller".into(),
            HashMap::new(),
            "public_key_multibase".into()
        );

        let did_service = Service::new(
            "id".into(),
            "type".into(),
            "service_endpoint".into()
        );

        let did_data = Did::new(
            vec!("context".to_string()),
            "id".into(),
            vec!("controller".to_string()),
            vec!(verification_method),
            vec!("authentication".to_string()),
            vec!("assertion_method".to_string()),
            vec!("capability_invocation".to_string()),
            vec!("capability_delegation".to_string()),
            vec!("key_agreement".to_string()),
            vec!(did_service),
            vec!("also_known_as".to_string()),
        );

        let proto = did_data.to_proto().unwrap();
        let decoded = Did::from_proto(&proto).unwrap();

        assert_eq!(did_data, decoded);
    }
}
